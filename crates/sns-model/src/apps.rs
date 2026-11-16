//! Portable application and virtual filesystem commands.
use crate::Id;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Software {
    Terminal,
    FileExplorer,
    TextEditor,
    ImageViewer,
    Firewall,
    DnsServer,
    TcpServer,
    TcpClient,
    WebServer,
    WebBrowser,
    MailServer,
    MailClient,
    P2p,
}
impl Software {
    pub const ALL: [Self; 13] = [
        Self::Terminal,
        Self::FileExplorer,
        Self::TextEditor,
        Self::ImageViewer,
        Self::Firewall,
        Self::DnsServer,
        Self::TcpServer,
        Self::TcpClient,
        Self::WebServer,
        Self::WebBrowser,
        Self::MailServer,
        Self::MailClient,
        Self::P2p,
    ];
    pub fn id(self) -> &'static str {
        match self {
            Self::Terminal => "terminal",
            Self::FileExplorer => "file_explorer",
            Self::TextEditor => "text_editor",
            Self::ImageViewer => "image_viewer",
            Self::Firewall => "firewall",
            Self::DnsServer => "dns_server",
            Self::TcpServer => "tcp_server",
            Self::TcpClient => "tcp_client",
            Self::WebServer => "web_server",
            Self::WebBrowser => "web_browser",
            Self::MailServer => "mail_server",
            Self::MailClient => "mail_client",
            Self::P2p => "p2p",
        }
    }
    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|s| s.id() == id)
    }
    pub fn available(self) -> bool {
        matches!(
            self,
            Self::Terminal
                | Self::FileExplorer
                | Self::TextEditor
                | Self::ImageViewer
                | Self::TcpServer
                | Self::TcpClient
                | Self::Firewall
                | Self::DnsServer
                | Self::WebServer
                | Self::WebBrowser
                | Self::MailServer
                | Self::MailClient
                | Self::P2p
        )
    }
    pub fn milestone(self) -> u8 {
        match self {
            Self::Firewall => 5,
            Self::DnsServer | Self::WebServer | Self::WebBrowser => 4,
            Self::MailServer | Self::MailClient | Self::P2p => 6,
            _ => 2,
        }
    }
    pub fn label_key(self) -> &'static str {
        if self == Self::DnsServer {
            "dns_software"
        } else {
            self.id()
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum FileCommand {
    Mkdir {
        path: String,
    },
    Write {
        path: String,
        media_type: String,
        data_base64: String,
        overwrite: bool,
    },
    Copy {
        source: String,
        destination: String,
    },
    Move {
        source: String,
        destination: String,
    },
    Remove {
        path: String,
        recursive: bool,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum AppCommand {
    P2p {
        application: Id,
        command: crate::P2pCommand,
    },
    Mail {
        application: Id,
        command: crate::MailCommand,
    },
    ConfigureService {
        application: Id,
        config: crate::ServiceConfig,
    },
    Navigate {
        application: Id,
        url: String,
        #[serde(default)]
        fields: Option<std::collections::BTreeMap<String, String>>,
    },
    PurgeDns {
        application: Id,
    },
    Install {
        application: Id,
        software: Software,
    },
    Uninstall {
        application: Id,
    },
    OpenView {
        application: Id,
    },
    CloseView {
        application: Id,
    },
    StartServer {
        application: Id,
        port: u16,
    },
    Stop {
        application: Id,
    },
    Connect {
        application: Id,
        target: String,
        port: u16,
    },
    Send {
        application: Id,
        text: String,
    },
    ClearReceived {
        application: Id,
    },
    File {
        application: Id,
        command: FileCommand,
    },
    Terminal {
        application: Id,
        line: String,
    },
    Interrupt {
        application: Id,
    },
}
