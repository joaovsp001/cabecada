//! Online play without a server of our own: both devices connect to a free public MQTT
//! broker and talk through two topics named after a 6-digit room code
//! (`cabecada/v2/<code>/h` is read by the host, `.../c` by the guest). The game packets
//! are the same ones used on the local network, just carried by the broker.
//!
//! The host connects to the first broker that answers and only then makes the room code:
//! its first digit says which broker that was (digit % 3), so the guest always goes to the
//! same broker as the host.
//!
//! Plain MQTT 3.1.1 over TCP port 1883, QoS 0, written by hand (no extra library).
//! Connecting (name lookup, TCP, CONNECT, SUBSCRIBE) happens on a helper thread so the
//! game never freezes; after that the socket is read and written without blocking.

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::time::Duration;

/// The free public brokers, tried in this order by the host.
pub const BROKERS: [&str; 3] = ["broker.emqx.io:1883", "broker.hivemq.com:1883", "test.mosquitto.org:1883"];

#[derive(Clone, Debug, PartialEq)]
pub enum LinkState {
    Connecting,
    Ready,
    Failed(String),
}

type Connected = Result<(TcpStream, usize, String), String>;

pub struct Relay {
    pending: Option<mpsc::Receiver<Connected>>,
    stream: Option<TcpStream>,
    inbuf: Vec<u8>,
    outbuf: Vec<u8>,
    guest: bool,
    outbox: String,
    pub state: LinkState,
    pub broker: String,
    /// The room code (known at once when joining; when hosting, once connected).
    pub code: String,
    last_ping: std::time::Instant,
}

/// The topic read by the host ('h') or by the guest ('c') of a room.
fn topic(code: &str, side: char) -> String {
    format!("cabecada/v2/{}/{}", code, side)
}

/// Which broker a room code is on.
fn broker_of(code: &str) -> usize {
    code.chars().next().and_then(|c| c.to_digit(10)).unwrap_or(0) as usize % BROKERS.len()
}

/// A new 6-digit room code on broker `broker`.
fn room_code_for(broker: usize) -> String {
    let firsts: Vec<u32> = (0..10).filter(|d| *d as usize % BROKERS.len() == broker).collect();
    let r = random_u32();
    let first = firsts[(r % firsts.len() as u32) as usize];
    format!("{}{:05}", first, (r >> 4) % 100_000)
}

impl Relay {
    /// Creates a room: connects to the first broker that answers and then makes the code.
    pub fn host() -> Relay {
        Relay::start(None)
    }

    /// Joins the room `code` (on the broker the code names).
    pub fn join(code: &str) -> Relay {
        Relay::start(Some(code.to_string()))
    }

    fn start(join: Option<String>) -> Relay {
        let (tx, rx) = mpsc::channel();
        let id = format!("cbc{:08x}", random_u32());
        let guest = join.is_some();
        let code = join.clone().unwrap_or_default();
        std::thread::spawn(move || {
            let order: Vec<usize> = match &join {
                Some(code) => vec![broker_of(code)],
                None => (0..BROKERS.len()).collect(),
            };
            let mut last_err = String::from("sem resposta");
            for i in order {
                let code = join.clone().unwrap_or_else(|| room_code_for(i));
                let inbox = topic(&code, if guest { 'c' } else { 'h' });
                match open(BROKERS[i], &id, &inbox) {
                    Ok(s) => {
                        let _ = tx.send(Ok((s, i, code)));
                        return;
                    }
                    Err(e) => last_err = e,
                }
            }
            let _ = tx.send(Err(last_err));
        });
        Relay {
            pending: Some(rx),
            stream: None,
            inbuf: vec![],
            outbuf: vec![],
            guest,
            outbox: String::new(),
            state: LinkState::Connecting,
            broker: String::new(),
            code,
            last_ping: std::time::Instant::now(),
        }
    }

    /// Sends one game packet to the other side (dropped if not connected yet).
    pub fn send(&mut self, payload: &[u8]) {
        if self.state != LinkState::Ready {
            return;
        }
        let mut body = Vec::with_capacity(payload.len() + self.outbox.len() + 2);
        put_str(&mut body, &self.outbox);
        body.extend_from_slice(payload);
        packet(&mut self.outbuf, 0x30, &body);
        // a stalled connection must not eat memory: drop the backlog (the game resends state)
        if self.outbuf.len() > 256 * 1024 {
            self.outbuf.clear();
        }
        self.flush();
    }

    /// Everything that arrived since the last call.
    pub fn receive(&mut self) -> Vec<Vec<u8>> {
        let mut got = vec![];
        if let Some(rx) = &self.pending {
            match rx.try_recv() {
                Ok(Ok((s, broker, code))) => {
                    self.stream = Some(s);
                    self.broker = BROKERS[broker].to_string();
                    self.outbox = topic(&code, if self.guest { 'h' } else { 'c' });
                    self.code = code;
                    self.state = LinkState::Ready;
                    self.pending = None;
                    self.last_ping = std::time::Instant::now();
                }
                Ok(Err(e)) => {
                    self.state = LinkState::Failed(e);
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.state = LinkState::Failed("falha ao conectar".into());
                    self.pending = None;
                }
            }
        }
        let Some(s) = self.stream.as_mut() else { return got };
        let mut buf = [0u8; 4096];
        loop {
            match s.read(&mut buf) {
                Ok(0) => {
                    self.state = LinkState::Failed("o servidor fechou a conexão".into());
                    self.stream = None;
                    return got;
                }
                Ok(n) => self.inbuf.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    self.state = LinkState::Failed(format!("conexão perdida: {}", e));
                    self.stream = None;
                    return got;
                }
            }
        }
        // complete MQTT packets
        loop {
            let Some((kind, body_start, total)) = split_packet(&self.inbuf) else { break };
            if kind >> 4 == 3 {
                // PUBLISH, QoS 0: topic, then the payload
                let body = &self.inbuf[body_start..total];
                if body.len() >= 2 {
                    let tl = u16::from_be_bytes([body[0], body[1]]) as usize;
                    if body.len() >= 2 + tl {
                        got.push(body[2 + tl..].to_vec());
                    }
                }
            }
            self.inbuf.drain(..total);
        }
        if self.last_ping.elapsed() > Duration::from_secs(20) {
            self.last_ping = std::time::Instant::now();
            self.outbuf.extend_from_slice(&[0xC0, 0x00]); // PINGREQ
        }
        self.flush();
        got
    }

    fn flush(&mut self) {
        let Some(s) = self.stream.as_mut() else { return };
        while !self.outbuf.is_empty() {
            match s.write(&self.outbuf) {
                Ok(0) => break,
                Ok(n) => {
                    self.outbuf.drain(..n);
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    self.state = LinkState::Failed(format!("conexão perdida: {}", e));
                    self.stream = None;
                    break;
                }
            }
        }
    }

    /// Says goodbye to the broker.
    pub fn close(&mut self) {
        if let Some(s) = self.stream.as_mut() {
            let _ = s.set_nonblocking(false);
            let _ = s.set_write_timeout(Some(Duration::from_millis(300)));
            let _ = s.write_all(&self.outbuf);
            let _ = s.write_all(&[0xE0, 0x00]); // DISCONNECT
        }
        self.stream = None;
        self.outbuf.clear();
    }
}

impl Drop for Relay {
    fn drop(&mut self) {
        self.close();
    }
}

/// Name lookup, TCP connection, MQTT CONNECT and SUBSCRIBE (blocking, on the helper thread).
fn open(broker: &str, client_id: &str, inbox: &str) -> Result<TcpStream, String> {
    let addrs: Vec<_> = broker.to_socket_addrs().map_err(|_| "servidor não encontrado (sem internet?)".to_string())?.collect();
    let mut last = String::from("servidor não encontrado");
    for a in addrs {
        match TcpStream::connect_timeout(&a, Duration::from_secs(4)) {
            Ok(mut s) => {
                let _ = s.set_nodelay(true);
                s.set_read_timeout(Some(Duration::from_secs(5))).map_err(|e| e.to_string())?;
                // CONNECT: protocol "MQTT" level 4, clean session, keep-alive 60 s
                let mut body = vec![];
                put_str(&mut body, "MQTT");
                body.extend_from_slice(&[4, 0x02, 0, 60]);
                put_str(&mut body, client_id);
                let mut out = vec![];
                packet(&mut out, 0x10, &body);
                // SUBSCRIBE (packet id 1) to our inbox, QoS 0
                let mut sb = vec![0, 1];
                put_str(&mut sb, inbox);
                sb.push(0);
                packet(&mut out, 0x82, &sb);
                s.write_all(&out).map_err(|e| e.to_string())?;
                // wait for CONNACK (accepted) and SUBACK
                let mut buf = vec![];
                let mut tmp = [0u8; 256];
                let (mut connack, mut suback) = (false, false);
                let start = std::time::Instant::now();
                while !(connack && suback) && start.elapsed() < Duration::from_secs(5) {
                    let n = s.read(&mut tmp).map_err(|_| "o servidor não respondeu".to_string())?;
                    if n == 0 {
                        return Err("o servidor fechou a conexão".into());
                    }
                    buf.extend_from_slice(&tmp[..n]);
                    while let Some((kind, body_start, total)) = split_packet(&buf) {
                        match kind >> 4 {
                            2 => {
                                if buf.get(body_start + 1) != Some(&0) {
                                    return Err("o servidor recusou".into());
                                }
                                connack = true;
                            }
                            9 => suback = true,
                            _ => {}
                        }
                        buf.drain(..total);
                    }
                }
                if !(connack && suback) {
                    return Err("o servidor não respondeu".into());
                }
                s.set_read_timeout(None).map_err(|e| e.to_string())?;
                s.set_nonblocking(true).map_err(|e| e.to_string())?;
                return Ok(s);
            }
            Err(_) => last = "não foi possível conectar (a rede pode estar bloqueando)".into(),
        }
    }
    Err(last)
}

fn put_str(out: &mut Vec<u8>, s: &str) {
    out.extend_from_slice(&(s.len() as u16).to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

/// Appends one MQTT packet: fixed header byte, remaining length, body.
fn packet(out: &mut Vec<u8>, first: u8, body: &[u8]) {
    out.push(first);
    let mut len = body.len();
    loop {
        let mut b = (len % 128) as u8;
        len /= 128;
        if len > 0 {
            b |= 0x80;
        }
        out.push(b);
        if len == 0 {
            break;
        }
    }
    out.extend_from_slice(body);
}

/// If `buf` starts with a whole packet: (first byte, where the body starts, packet size).
fn split_packet(buf: &[u8]) -> Option<(u8, usize, usize)> {
    if buf.len() < 2 {
        return None;
    }
    let mut len = 0usize;
    let mut mult = 1usize;
    let mut i = 1;
    loop {
        let b = *buf.get(i)?;
        len += (b & 0x7f) as usize * mult;
        i += 1;
        if b & 0x80 == 0 {
            break;
        }
        mult *= 128;
        if i > 4 {
            return None;
        }
    }
    if buf.len() < i + len {
        return None;
    }
    Some((buf[0], i, i + len))
}

fn random_u32() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(1);
    let mut x = (t as u32) ^ ((t >> 32) as u32) ^ 0x9e37_79b9 ^ COUNTER.fetch_add(0x6d2b_79f5, Ordering::Relaxed);
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_name_their_broker() {
        for b in 0..BROKERS.len() {
            for _ in 0..50 {
                let c = room_code_for(b);
                assert_eq!(c.len(), 6, "{}", c);
                assert!(c.chars().all(|ch| ch.is_ascii_digit()), "{}", c);
                assert_eq!(broker_of(&c), b, "{}", c);
            }
        }
    }
}
