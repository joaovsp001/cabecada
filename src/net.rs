//! Local-network play with no server: one player's device hosts the match
//! (it runs the simulation), the other joins over Wi-Fi or a phone hotspot.
//! Plain UDP. The host announces itself by broadcast; joiners can also ask
//! by broadcast or type the host's address.

use crate::game::{Input, Sim};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

pub const PORT: u16 = 47777;
pub const DISC_PORT: u16 = 47778;
const MAGIC: &[u8; 4] = b"CBC1";

const T_HELLO: u8 = 1;
const T_WELCOME: u8 = 2;
const T_INPUT: u8 = 3;
const T_STATE: u8 = 4;
const T_BEACON: u8 = 5;
const T_DISCOVER: u8 = 6;
const T_BYE: u8 = 7;
const T_REMATCH: u8 = 8;

pub const TIMEOUT: f64 = 5.0;

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

pub struct Host {
    sock: UdpSocket,
    pub port: u16,
    pub peer: Option<SocketAddr>,
    last_seen: f64,
    pub remote: Input,
    pub remote_pkind: u8,
    remote_seq: u32,
    seq: u32,
    beacon_t: f64,
    pub name: String,
}

impl Host {
    pub fn new(port: u16, name: &str) -> std::io::Result<Host> {
        let sock = UdpSocket::bind(("0.0.0.0", port))?;
        sock.set_nonblocking(true)?;
        let _ = sock.set_broadcast(true);
        let port = sock.local_addr()?.port();
        Ok(Host { sock, port, peer: None, last_seen: 0.0, remote: Input::default(), remote_pkind: 0, remote_seq: 0, seq: 0, beacon_t: -10.0, name: name.to_string() })
    }

    pub fn connected(&self, now: f64) -> bool {
        self.peer.is_some() && now - self.last_seen < TIMEOUT
    }

    /// Read everything waiting. Returns true when a new player joined.
    pub fn poll(&mut self, now: f64) -> bool {
        let mut joined = false;
        let mut buf = [0u8; 1500];
        loop {
            match self.sock.recv_from(&mut buf) {
                Ok((n, from)) => {
                    let Some((t, pl)) = parse(&buf[..n]) else { continue };
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
                                let _ = self.sock.send_to(&packet(T_WELCOME, &[]), from);
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
                        T_DISCOVER => {
                            let _ = self.sock.send_to(&self.beacon_packet(), from);
                        }
                        T_BYE if Some(from) == self.peer => {
                            self.peer = None;
                            self.remote = Input::default();
                        }
                        _ => {}
                    }
                }
                Err(_) => break,
            }
        }
        if self.peer.is_none() && now - self.beacon_t > 0.5 {
            self.beacon_t = now;
            let pk = self.beacon_packet();
            for t in broadcast_targets(DISC_PORT) {
                let _ = self.sock.send_to(&pk, t);
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
        if let Some(peer) = self.peer {
            self.seq += 1;
            let mut pl = self.seq.to_le_bytes().to_vec();
            sim.to_bytes(&mut pl);
            let _ = self.sock.send_to(&packet(T_STATE, &pl), peer);
        }
    }

    pub fn send_rematch(&mut self) {
        if let Some(peer) = self.peer {
            let _ = self.sock.send_to(&packet(T_REMATCH, &[]), peer);
        }
    }

    pub fn bye(&mut self) {
        if let Some(peer) = self.peer.take() {
            for _ in 0..3 {
                let _ = self.sock.send_to(&packet(T_BYE, &[]), peer);
            }
        }
    }
}

pub struct Found {
    pub addr: SocketAddr,
    pub name: String,
    pub seen: f64,
}

pub struct Client {
    sock: UdpSocket,
    disc: Option<UdpSocket>,
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
}

impl Client {
    pub fn new() -> std::io::Result<Client> {
        let sock = UdpSocket::bind("0.0.0.0:0")?;
        sock.set_nonblocking(true)?;
        let _ = sock.set_broadcast(true);
        let disc = UdpSocket::bind(("0.0.0.0", DISC_PORT)).ok().map(|s| {
            let _ = s.set_nonblocking(true);
            s
        });
        Ok(Client {
            sock,
            disc,
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
        })
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
        let mut buf = [0u8; 1500];
        let mut packets: Vec<(Vec<u8>, SocketAddr)> = vec![];
        loop {
            match self.sock.recv_from(&mut buf) {
                Ok((n, from)) => packets.push((buf[..n].to_vec(), from)),
                Err(_) => break,
            }
        }
        if let Some(d) = &self.disc {
            loop {
                match d.recv_from(&mut buf) {
                    Ok((n, from)) => packets.push((buf[..n].to_vec(), from)),
                    Err(_) => break,
                }
            }
        }
        for (data, from) in packets {
            let Some((t, pl)) = parse(&data) else { continue };
            match t {
                T_BEACON if pl.len() >= 2 => {
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
                let mut pl = vec![self.pkind];
                pl.extend_from_slice(b"jogador");
                let _ = self.sock.send_to(&packet(T_HELLO, &pl), h);
            }
        } else if now - self.discover_t > 1.0 {
            self.discover_t = now;
            let pk = packet(T_DISCOVER, &[]);
            for t in broadcast_targets(PORT) {
                let _ = self.sock.send_to(&pk, t);
            }
        }
        rematch
    }

    pub fn send_input(&mut self, inp: Input) {
        if let Some(h) = self.host {
            self.seq += 1;
            let mut pl = self.seq.to_le_bytes().to_vec();
            pl.push(inp.bits());
            let _ = self.sock.send_to(&packet(T_INPUT, &pl), h);
        }
    }

    pub fn bye(&mut self) {
        if let Some(h) = self.host.take() {
            for _ in 0..3 {
                let _ = self.sock.send_to(&packet(T_BYE, &[]), h);
            }
        }
        self.welcomed = false;
    }
}
