//! Portable simulated socket commands. These never represent operating-system sockets.
use crate::Id;
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportProtocol {
    Tcp,
    Udp,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum SocketCommand {
    Bind {
        socket: Id,
        owner: Id,
        protocol: TransportProtocol,
        #[serde(default)]
        interface: Option<Id>,
        address: Option<Ipv4Addr>,
        port: u16,
    },
    Connect {
        socket: Id,
        owner: Id,
        destination: Ipv4Addr,
        port: u16,
        local_address: Option<Ipv4Addr>,
        local_port: Option<u16>,
    },
    Send {
        socket: Id,
        data_base64: String,
    },
    SendTo {
        socket: Id,
        destination: Ipv4Addr,
        port: u16,
        data_base64: String,
    },
    Close {
        socket: Id,
        force: bool,
    },
    CloseOwner {
        owner: Id,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Endpoint {
    pub address: Ipv4Addr,
    pub port: u16,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TcpFlags {
    pub syn: bool,
    pub ack: bool,
    pub fin: bool,
    pub rst: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TcpSegment {
    pub source_port: u16,
    pub destination_port: u16,
    pub sequence: u32,
    pub acknowledgment: u32,
    pub flags: TcpFlags,
    pub window: u16,
    pub data_base64: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "protocol", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransportPacket {
    Tcp(TcpSegment),
    Udp {
        source_port: u16,
        destination_port: u16,
        length: u16,
        data_base64: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TcpState {
    SynSent,
    SynReceived,
    Established,
    #[serde(rename = "FIN_WAIT_1")]
    FinWait1,
    #[serde(rename = "FIN_WAIT_2")]
    FinWait2,
    CloseWait,
    LastAck,
    Closing,
    TimeWait,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SocketView {
    pub device: Id,
    pub socket: Id,
    pub owner: Id,
    pub protocol: TransportProtocol,
    pub interface: Option<Id>,
    pub local: Endpoint,
    pub peer: Option<Endpoint>,
    pub state: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum SocketNoticeKind {
    Listening,
    Connected,
    Accepted,
    PeerClosed,
    Closed {
        reason: String,
    },
    Data {
        data_base64: String,
    },
    Datagram {
        source: Endpoint,
        destination: Endpoint,
        data_base64: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct SocketNotice {
    pub device: Id,
    pub socket: Id,
    pub owner: Id,
    pub correlation: Id,
    pub kind: SocketNoticeKind,
}
