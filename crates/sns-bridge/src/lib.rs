//! Explicit real-socket adapter. The simulation never opens host sockets itself.
mod driver;
pub use driver::{BridgeView, Driver};
use sns_model::*;
use std::{
    collections::VecDeque,
    io::{Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::mpsc,
    time::{Duration, Instant},
};
const HELLO: &[u8] = b"SNS-BRIDGE/1\n";
#[derive(Clone, Debug)]
pub enum BridgeEvent {
    Listening(SocketAddr),
    Connected,
    Packet(BridgePacket),
    Disconnected(String),
}
enum Request {
    Packet(BridgePacket),
    Stop,
}
pub struct Bridge {
    sender: mpsc::SyncSender<Request>,
    receiver: mpsc::Receiver<BridgeEvent>,
    terminated: std::cell::Cell<bool>,
}
impl Bridge {
    /// Creating this adapter is the sole activation point; loading a project cannot call it.
    pub fn start(config: ModemConfig) -> Result<Self, Diagnostic> {
        config.validate()?;
        let (sender, requests) = mpsc::sync_channel(32);
        let (events, receiver) = mpsc::sync_channel(32);
        std::thread::Builder::new()
            .name("sinesi-modem".into())
            .spawn(move || {
                if let Err(code) = serve(config, &requests, &events) {
                    let _ = events.try_send(BridgeEvent::Disconnected(code.into()));
                }
            })
            .map_err(|_| Diagnostic::new("bridge.worker_failed", "modem"))?;
        Ok(Self {
            sender,
            receiver,
            terminated: std::cell::Cell::new(false),
        })
    }
    pub fn send(&self, packet: BridgePacket) -> Result<(), Diagnostic> {
        if packet.version != 1
            || serde_json::to_vec(&packet).map_or(true, |v| v.len() > BRIDGE_MAX_PACKET)
        {
            return Err(Diagnostic::new("bridge.invalid_message", "packet"));
        }
        self.sender
            .try_send(Request::Packet(packet))
            .map_err(|_| Diagnostic::new("bridge.queue_full", "modem"))
    }
    pub fn poll(&self) -> Vec<BridgeEvent> {
        let mut events = Vec::new();
        for _ in 0..64 {
            match self.receiver.try_recv() {
                Ok(e) => {
                    if matches!(e, BridgeEvent::Disconnected(_)) {
                        self.terminated.set(true);
                    }
                    events.push(e);
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    if !self.terminated.replace(true) {
                        events.push(BridgeEvent::Disconnected("bridge.disconnected".into()));
                    }
                    break;
                }
            }
        }
        events
    }
}
impl Drop for Bridge {
    fn drop(&mut self) {
        let _ = self.sender.try_send(Request::Stop);
    }
}
fn stopped(rx: &mpsc::Receiver<Request>) -> bool {
    matches!(
        rx.try_recv(),
        Ok(Request::Stop) | Err(mpsc::TryRecvError::Disconnected)
    )
}
fn serve(
    config: ModemConfig,
    rx: &mpsc::Receiver<Request>,
    tx: &mpsc::SyncSender<BridgeEvent>,
) -> Result<(), &'static str> {
    let endpoint = config
        .endpoint
        .parse::<SocketAddr>()
        .map_err(|_| "bridge.invalid_endpoint")?;
    let mut stream = match config.role {
        ModemRole::Connect => TcpStream::connect_timeout(&endpoint, Duration::from_millis(500))
            .map_err(|_| "bridge.endpoint_unavailable")?,
        ModemRole::Listen => {
            let listener =
                TcpListener::bind(endpoint).map_err(|_| "bridge.endpoint_unavailable")?;
            listener
                .set_nonblocking(true)
                .map_err(|_| "bridge.io_error")?;
            tx.try_send(BridgeEvent::Listening(
                listener.local_addr().map_err(|_| "bridge.io_error")?,
            ))
            .map_err(|_| "bridge.queue_full")?;
            loop {
                if stopped(rx) {
                    return Err("bridge.disconnected");
                }
                match listener.accept() {
                    Ok((s, _)) => break s,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => return Err("bridge.io_error"),
                }
            }
        }
    };
    stream
        .set_nonblocking(true)
        .map_err(|_| "bridge.io_error")?;
    let _ = stream.set_nodelay(true);
    let mut output = VecDeque::from([HELLO.to_vec()]);
    let mut offset = 0;
    let mut input = Vec::new();
    let mut greeting = false;
    let started = Instant::now();
    let mut sequence = 0;
    loop {
        for _ in 0..32 {
            match rx.try_recv() {
                Ok(Request::Stop) | Err(mpsc::TryRecvError::Disconnected) => {
                    return Err("bridge.disconnected");
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Ok(Request::Packet(packet)) => {
                    if !greeting {
                        return Err("bridge.not_connected");
                    }
                    if output.len() >= 32 {
                        return Err("bridge.queue_full");
                    }
                    let bytes =
                        serde_json::to_vec(&packet).map_err(|_| "bridge.invalid_message")?;
                    if bytes.len() > BRIDGE_MAX_PACKET {
                        return Err("bridge.message_too_large");
                    }
                    let mut framed = (bytes.len() as u32).to_be_bytes().to_vec();
                    framed.extend(bytes);
                    output.push_back(framed);
                }
            }
        }
        for _ in 0..16 {
            let Some(bytes) = output.front() else { break };
            match stream.write(&bytes[offset..]) {
                Ok(0) => return Err("bridge.disconnected"),
                Ok(n) => {
                    offset += n;
                    if offset == bytes.len() {
                        output.pop_front();
                        offset = 0;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => return Err("bridge.disconnected"),
            }
        }
        let mut buffer = [0u8; 8192];
        for _ in 0..16 {
            match stream.read(&mut buffer) {
                Ok(0) => return Err("bridge.disconnected"),
                Ok(n) => {
                    input.extend_from_slice(&buffer[..n]);
                    if !greeting {
                        if let Some(i) = input.iter().position(|b| *b == b'\n') {
                            if &input[..=i] != HELLO {
                                return Err("bridge.incompatible_version");
                            }
                            input.drain(..=i);
                            greeting = true;
                            tx.try_send(BridgeEvent::Connected)
                                .map_err(|_| "bridge.queue_full")?;
                        } else if input.len() > 64 {
                            return Err("bridge.incompatible_version");
                        }
                    }
                    if greeting {
                        for _ in 0..32 {
                            if input.len() < 4 {
                                break;
                            }
                            let len = u32::from_be_bytes(input[..4].try_into().unwrap()) as usize;
                            if len == 0 || len > BRIDGE_MAX_PACKET {
                                return Err("bridge.message_too_large");
                            }
                            if input.len() < len + 4 {
                                break;
                            }
                            let packet: BridgePacket = serde_json::from_slice(&input[4..len + 4])
                                .map_err(|_| "bridge.invalid_message")?;
                            if packet.version != 1
                                || packet.sequence.0 != sequence + 1
                                || !(1..=64).contains(&packet.budget)
                            {
                                return Err("bridge.invalid_message");
                            }
                            sequence = packet.sequence.0;
                            input.drain(..len + 4);
                            tx.try_send(BridgeEvent::Packet(packet))
                                .map_err(|_| "bridge.queue_full")?;
                        }
                    }
                    if input.len() > BRIDGE_MAX_PACKET + 4 {
                        return Err("bridge.message_too_large");
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => return Err("bridge.disconnected"),
            }
        }
        if !greeting && started.elapsed() > Duration::from_secs(2) {
            return Err("bridge.handshake_timeout");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
