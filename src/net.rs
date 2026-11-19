//! Two-player matches between devices, with no game server: one player's device hosts
//! the match (it runs the simulation), the other joins.
//!
//! - Local network: plain UDP. The host announces itself by broadcast; joiners can also
//!   ask by broadcast or type the host's address.
//! - Internet: the same packets go through a public MQTT broker (see `relay.rs`), and
//!   the two devices find each other by a 6-digit room code.

use crate::game::{Input, Sim};
use crate::relay::{LinkState, Relay};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub const PORT: u16 = 47777;
pub const DISC_PORT: u16 = 47778;
const MAGIC: &[u8; 4] = b"CBC2";

const T_HELLO: u8 = 1;
const T_WELCOME: u8 = 2;
const T_INPUT: u8 = 3;
const T_STATE: u8 = 4;
const T_BEACON: u8 = 5;
const T_DISCOVER: u8 = 6;
const T_BYE: u8 = 7;
const T_REMATCH: u8 = 8;

pub const TIMEOUT: f64 = 5.0;
/// Over the internet everything comes from the broker: this stands for "the other player".
pub const RELAY_PEER: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 1);

fn packet(t: u8, payload: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(5 + payload.len());
    v.extend_from_slice(MAGIC);
    v.push(t);
    v.extend_from_slice(payload);
    v
}

fn parse(buf: &[u8]) -> Option<(u8, &[u8])> {
    if buf.len() < 5 || &buf[0..4] != MAGIC {
        return None;
    }
    Some((buf[4], &buf[5..]))
}

fn broadcast_targets(port: u16) -> Vec<SocketAddr> {
    let mut v = vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), port)];
    if let Some(IpAddr::V4(ip)) = local_ip() {
        let o = ip.octets();
        v.push(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(o[0], o[1], o[2], 255)), port));
    }
    v
}

/// This device's address on the local network (no packet is sent).
pub fn local_ip() -> Option<IpAddr> {
    let probes = ["8.8.8.8:80", "192.168.0.1:80", "192.168.1.1:80", "192.168.43.1:80", "10.0.0.1:80", "172.20.10.1:80"];
    for p in probes {
        if let Ok(s) = UdpSocket::bind("0.0.0.0:0") {
            if s.connect(p).is_ok() {
                if let Ok(a) = s.local_addr() {
                    if !a.ip().is_unspecified() && !a.ip().is_loopback() {
                        return Some(a.ip());
                    }
                }
            }
        }
    }
    None
}

/// How packets travel: UDP on the local network, or the broker over the internet.
enum Wire {
    Udp { sock: UdpSocket, disc: Option<UdpSocket> },
    Relay(Relay),
}

impl Wire {
    /// Every packet waiting, with who sent it.
    fn recv_all(&mut self) -> Vec<(Vec<u8>, SocketAddr)> {
        let mut out = vec![];
        match self {
            Wire::Udp { sock, disc } => {
                let mut buf = [0u8; 1500];
                while let Ok((n, from)) = sock.recv_from(&mut buf) {
                    out.push((buf[..n].to_vec(), from));
                }
                if let Some(d) = disc {
                    while let Ok((n, from)) = d.recv_from(&mut buf) {
                        out.push((buf[..n].to_vec(), from));
                    }
                }
            }
            Wire::Relay(r) => {
                for p in r.receive() {
                    out.push((p, RELAY_PEER));
                }
            }
        }
        out
    }

    fn send(&mut self, data: &[u8], to: SocketAddr) {
        match self {
            Wire::Udp { sock, .. } => {
                let _ = sock.send_to(data, to);
            }
            Wire::Relay(r) => r.send(data),
        }
    }

    fn is_udp(&self) -> bool {
        matches!(self, Wire::Udp { .. })
    }

    fn link(&self) -> Option<LinkState> {
        match self {
            Wire::Udp { .. } => None,
            Wire::Relay(r) => Some(r.state.clone()),
        }
    }
}

/// Over the internet, at most this many state or input packets per second (the public
/// broker is shared; 30 is plenty for the picture, and button changes still go at once).
const RELAY_RATE: f64 = 30.0;

pub struct Host {
    wire: Wire,
    pub port: u16,
    pub peer: Option<SocketAddr>,
    last_seen: f64,
    pub remote: Input,
    pub remote_pkind: u8,
    pub remote_look: u8,
    remote_seq: u32,
    seq: u32,
    beacon_t: f64,
    pub name: String,
    pending_ev: u32,                        // sound/effect events not sent yet
    last_send: Option<std::time::Instant>, // last state sent (online rooms send fewer)
}

impl Host {
    /// A match on the local network (UDP).
    pub fn new(port: u16, name: &str) -> std::io::Result<Host> {
        let sock = UdpSocket::bind(("0.0.0.0", port))?;
        sock.set_nonblocking(true)?;
        let _ = sock.set_broadcast(true);
        let port = sock.local_addr()?.port();
        Ok(Host::with_wire(Wire::Udp { sock, disc: None }, port, name))
    }

    /// A room on the internet; its 6-digit code is ready once the broker answers.
    pub fn new_online() -> Host {
        Host::with_wire(Wire::Relay(Relay::host()), 0, "")
    }

    fn with_wire(wire: Wire, port: u16, name: &str) -> Host {
        Host {
            wire,
            port,
            peer: None,
            last_seen: 0.0,
            remote: Input::default(),
            remote_pkind: 0,
            remote_look: 1,
            remote_seq: 0,
            seq: 0,
            beacon_t: -10.0,
            name: name.to_string(),
            pending_ev: 0,
            last_send: None,
        }
    }

    /// The code of an online room (empty until the broker answered, or on the local network).
    pub fn room_code(&self) -> String {
        match &self.wire {
            Wire::Relay(r) => r.code.clone(),
            Wire::Udp { .. } => String::new(),
        }
    }

    /// The broker connection, for online rooms (None on the local network).
    pub fn link(&self) -> Option<LinkState> {
        self.wire.link()
    }

    pub fn connected(&self, now: f64) -> bool {
        self.peer.is_some() && now - self.last_seen < TIMEOUT
    }

    /// Read everything waiting. Returns true when a new player joined.
    pub fn poll(&mut self, now: f64) -> bool {
        let mut joined = false;
        for (data, from) in self.wire.recv_all() {
            let Some((t, pl)) = parse(&data) else { continue };
            match t {
                T_HELLO => {
                    if self.peer.is_none() || self.peer == Some(from) || now - self.last_seen > TIMEOUT {
                        if self.peer != Some(from) {
                            joined = true;
                            self.remote_seq = 0;
                        }
                        self.peer = Some(from);
                        self.last_seen = now;
                        if let Some(k) = pl.first() {
                            self.remote_pkind = *k % crate::game::POWER_KINDS;
                        }
                        if let Some(l) = pl.get(1) {
                            self.remote_look = *l % crate::game::CHARS.len() as u8;
                        }
                        let welcome = packet(T_WELCOME, &[]);
                        self.wire.send(&welcome, from);
                    }
                }
                T_INPUT if Some(from) == self.peer && pl.len() >= 5 => {
                    let seq = u32::from_le_bytes([pl[0], pl[1], pl[2], pl[3]]);
                    if seq >= self.remote_seq {
                        self.remote_seq = seq;
                        self.remote = Input::from_bits(pl[4]);
                    }
                    self.last_seen = now;
                }
                T_DISCOVER if self.wire.is_udp() => {
                    let b = self.beacon_packet();
                    self.wire.send(&b, from);
                }
                T_BYE if Some(from) == self.peer => {
                    self.peer = None;
                    self.remote = Input::default();
                }
                _ => {}
            }
        }
        if self.wire.is_udp() && self.peer.is_none() && now - self.beacon_t > 0.5 {
            self.beacon_t = now;
            let pk = self.beacon_packet();
            for t in broadcast_targets(DISC_PORT) {
                self.wire.send(&pk, t);
            }
        }
        joined
    }

    fn beacon_packet(&self) -> Vec<u8> {
        let mut pl = self.port.to_le_bytes().to_vec();
        pl.extend_from_slice(self.name.as_bytes());
        packet(T_BEACON, &pl)
    }

    pub fn send_state(&mut self, sim: &Sim) {
        let Some(peer) = self.peer else { return };
        self.pending_ev |= sim.events;
        if !self.wire.is_udp() {
            // online: fewer packets; the events of the skipped frames go with the next one
            if self.last_send.map(|t| t.elapsed().as_secs_f64() < 1.0 / RELAY_RATE).unwrap_or(false) {
                return;
            }
            self.last_send = Some(std::time::Instant::now());
        }
        self.seq += 1;
        let mut pl = self.seq.to_le_bytes().to_vec();
        if sim.events == self.pending_ev {
            sim.to_bytes(&mut pl);
        } else {
            let mut s = sim.clone();
            s.events = self.pending_ev;
            s.to_bytes(&mut pl);
        }
        self.pending_ev = 0;
        let pk = packet(T_STATE, &pl);
        self.wire.send(&pk, peer);
    }

    pub fn send_rematch(&mut self) {
        if let Some(peer) = self.peer {
            let pk = packet(T_REMATCH, &[]);
            self.wire.send(&pk, peer);
        }
    }

    pub fn bye(&mut self) {
        if let Some(peer) = self.peer.take() {
            for _ in 0..3 {
                let pk = packet(T_BYE, &[]);
                self.wire.send(&pk, peer);
            }
        }
        if let Wire::Relay(r) = &mut self.wire {
            r.close();
        }
    }
}

pub struct Found {
    pub addr: SocketAddr,
    pub name: String,
    pub seen: f64,
}

pub struct Client {
    wire: Wire,
    pub host: Option<SocketAddr>,
    pub welcomed: bool,
    pub found: Vec<Found>,
    pub latest: Option<Sim>,
    pub events: u32,
    state_seq: u32,
    seq: u32,
    hello_t: f64,
    discover_t: f64,
    pub last_seen: f64,
    pub host_left: bool,
    pub pkind: u8,
    pub look: u8,
    last_bits: u8,                          // last input sent
    last_send: Option<std::time::Instant>, // when (online rooms send fewer)
}

impl Client {
    /// Joining on the local network (UDP).
    pub fn new() -> std::io::Result<Client> {
        let sock = UdpSocket::bind("0.0.0.0:0")?;
        sock.set_nonblocking(true)?;
        let _ = sock.set_broadcast(true);
        let disc = UdpSocket::bind(("0.0.0.0", DISC_PORT)).ok().map(|s| {
            let _ = s.set_nonblocking(true);
            s
        });
        Ok(Client::with_wire(Wire::Udp { sock, disc }))
    }

    /// Joining an internet room by its 6-digit code.
    pub fn new_online(code: &str) -> Client {
        let mut cl = Client::with_wire(Wire::Relay(Relay::join(code)));
        cl.host = Some(RELAY_PEER);
        cl
    }

    fn with_wire(wire: Wire) -> Client {
        Client {
            wire,
            host: None,
            welcomed: false,
            found: vec![],
            latest: None,
            events: 0,
            state_seq: 0,
            seq: 0,
            hello_t: -10.0,
            discover_t: -10.0,
            last_seen: 0.0,
            host_left: false,
            pkind: 0,
            look: 0,
            last_bits: 0,
            last_send: None,
        }
    }

    /// The broker connection, for online rooms (None on the local network).
    pub fn link(&self) -> Option<LinkState> {
        self.wire.link()
    }

    pub fn connect(&mut self, addr: SocketAddr, now: f64) {
        self.host = Some(addr);
        self.welcomed = false;
        self.latest = None;
        self.state_seq = 0;
        self.hello_t = -10.0;
        self.last_seen = now;
        self.host_left = false;
    }

    pub fn connected(&self, now: f64) -> bool {
        self.welcomed && now - self.last_seen < TIMEOUT
    }

    fn note_found(&mut self, addr: SocketAddr, name: String, now: f64) {
        if let Some(f) = self.found.iter_mut().find(|f| f.addr == addr) {
            f.seen = now;
            f.name = name;
        } else {
            self.found.push(Found { addr, name, seen: now });
        }
    }

    /// Returns true when the host started a rematch.
    pub fn poll(&mut self, now: f64) -> bool {
        let mut rematch = false;
        for (data, from) in self.wire.recv_all() {
            let Some((t, pl)) = parse(&data) else { continue };
            match t {
                T_BEACON if pl.len() >= 2 && self.wire.is_udp() => {
                    let port = u16::from_le_bytes([pl[0], pl[1]]);
                    let name = String::from_utf8_lossy(&pl[2..]).to_string();
                    self.note_found(SocketAddr::new(from.ip(), port), name, now);
                }
                T_WELCOME if Some(from) == self.host => {
                    self.welcomed = true;
                    self.last_seen = now;
                }
                T_STATE if Some(from) == self.host && pl.len() > 4 => {
                    let seq = u32::from_le_bytes([pl[0], pl[1], pl[2], pl[3]]);
                    self.welcomed = true;
                    self.last_seen = now;
                    if seq > self.state_seq || seq + 1000 < self.state_seq {
                        self.state_seq = seq;
                        if let Some(s) = Sim::from_bytes(&pl[4..]) {
                            self.events |= s.events;
                            self.latest = Some(s);
                        }
                    }
                }
                T_REMATCH if Some(from) == self.host => {
                    rematch = true;
                }
                T_BYE if Some(from) == self.host => {
                    self.host_left = true;
                    self.welcomed = false;
                }
                _ => {}
            }
        }
        self.found.retain(|f| now - f.seen < 4.0);
        if let Some(h) = self.host {
            if !self.welcomed && now - self.hello_t > 0.3 {
                self.hello_t = now;
                let mut pl = vec![self.pkind, self.look];
                pl.extend_from_slice(b"jogador");
                let pk = packet(T_HELLO, &pl);
                self.wire.send(&pk, h);
            }
        } else if self.wire.is_udp() && now - self.discover_t > 1.0 {
            self.discover_t = now;
            let pk = packet(T_DISCOVER, &[]);
            for t in broadcast_targets(PORT) {
                self.wire.send(&pk, t);
            }
        }
        rematch
    }

    pub fn send_input(&mut self, inp: Input) {
        if let Some(h) = self.host {
            if !self.wire.is_udp() {
                // online: a change goes at once, otherwise a reminder 30 times a second
                let same = inp.bits() == self.last_bits;
                if same && self.last_send.map(|t| t.elapsed().as_secs_f64() < 1.0 / RELAY_RATE).unwrap_or(false) {
                    return;
                }
                self.last_bits = inp.bits();
                self.last_send = Some(std::time::Instant::now());
            }
            self.seq += 1;
            let mut pl = self.seq.to_le_bytes().to_vec();
            pl.push(inp.bits());
            let pk = packet(T_INPUT, &pl);
            self.wire.send(&pk, h);
        }
    }

    pub fn bye(&mut self) {
        if let Some(h) = self.host.take() {
            for _ in 0..3 {
                let pk = packet(T_BYE, &[]);
                self.wire.send(&pk, h);
            }
        }
        self.welcomed = false;
        if let Wire::Relay(r) = &mut self.wire {
            r.close();
        }
    }
}
