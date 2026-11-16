//! Adapter orchestration shared by the native application and two-process qualification.
use super::*;
use sns_engine::Engine;
use std::collections::BTreeMap;
#[derive(Clone, Debug, serde::Serialize)]
pub struct BridgeView {
    pub state: String,
    pub endpoint: String,
    pub detail: Option<String>,
}
struct Session {
    id: Id,
    transport: Option<Bridge>,
    connected: bool,
    view: BridgeView,
}
#[derive(Default)]
pub struct Driver {
    sessions: BTreeMap<Id, Session>,
    serial: u64,
}
impl Driver {
    pub fn views(&self) -> BTreeMap<Id, BridgeView> {
        self.sessions
            .iter()
            .map(|(d, s)| (d.clone(), s.view.clone()))
            .collect()
    }
    pub fn active(&self) -> bool {
        self.sessions.values().any(|s| s.transport.is_some())
    }
    fn issue(&mut self, e: &mut Engine, d: &Id, c: BridgeCommand) -> Result<(), Diagnostic> {
        let command_id = loop {
            self.serial = self
                .serial
                .checked_add(1)
                .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "bridge_command"))?;
            let id = Id::new(format!("bridge-adapter-{}", self.serial))?;
            if !e.command_id_taken(&id) {
                break id;
            }
        };
        e.command(CommandEnvelope {
            api_version: 1,
            command_id,
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::Bridge {
                device: d.clone(),
                command: c,
            },
        })
    }
    pub fn start(&mut self, e: &mut Engine, d: &Id, config: ModemConfig) -> Result<(), Diagnostic> {
        config.validate()?;
        if !e.project().device(d).is_some_and(|m| {
            m.kind == DeviceKind::Modem
                && m.interfaces.len() == 1
                && m.interfaces[0].medium == Medium::Ethernet
        }) {
            return Err(Diagnostic::new("bridge.invalid_config", d.as_str()));
        }
        self.stop(e, d);
        if let Some(session) = e.network().bridge_session(d).cloned() {
            self.issue(
                e,
                d,
                BridgeCommand::Disconnect {
                    session,
                    reason: "bridge.disconnected".into(),
                },
            )?;
        }
        self.serial = self
            .serial
            .checked_add(1)
            .ok_or_else(|| Diagnostic::new("simulation.sequence_overflow", "bridge_session"))?;
        let id = Id::new(format!(
            "bridge-session-{}-{}",
            e.run_id().as_str(),
            self.serial
        ))?;
        let transport = Bridge::start(config.clone())?;
        self.sessions.insert(
            d.clone(),
            Session {
                id,
                transport: Some(transport),
                connected: false,
                view: BridgeView {
                    state: "bridge.connecting".into(),
                    endpoint: config.endpoint,
                    detail: None,
                },
            },
        );
        Ok(())
    }
    pub fn stop(&mut self, e: &mut Engine, d: &Id) {
        if let Some(mut s) = self.sessions.remove(d) {
            self.end(e, d, &mut s, "bridge.disconnected");
            self.sessions.insert(d.clone(), s);
        }
    }
    pub fn stop_all(&mut self, e: &mut Engine) {
        let keys = self.sessions.keys().cloned().collect::<Vec<_>>();
        for d in keys {
            self.stop(e, &d);
        }
    }
    fn end(&mut self, e: &mut Engine, d: &Id, s: &mut Session, reason: &str) {
        s.transport = None;
        if s.connected {
            let _ = self.issue(
                e,
                d,
                BridgeCommand::Disconnect {
                    session: s.id.clone(),
                    reason: reason.into(),
                },
            );
        }
        s.connected = false;
        s.view.state = "bridge.disconnected".into();
        s.view.detail = Some(reason.into());
    }
    pub fn poll(&mut self, e: &mut Engine) -> bool {
        let keys = self.sessions.keys().cloned().collect::<Vec<_>>();
        let mut changed = false;
        for d in keys {
            let mut s = self.sessions.remove(&d).unwrap();
            let events = s.transport.as_ref().map(Bridge::poll).unwrap_or_default();
            for event in events {
                changed = true;
                match event {
                    BridgeEvent::Listening(endpoint) => {
                        s.view.state = "bridge.listening".into();
                        s.view.endpoint = endpoint.to_string();
                    }
                    BridgeEvent::Connected => {
                        match self.issue(
                            e,
                            &d,
                            BridgeCommand::Activate {
                                session: s.id.clone(),
                            },
                        ) {
                            Ok(()) => {
                                s.connected = true;
                                s.view.state = "bridge.connected".into();
                                s.view.detail = None;
                            }
                            Err(err) => self.end(e, &d, &mut s, &err.code),
                        }
                    }
                    BridgeEvent::Disconnected(code) => self.end(e, &d, &mut s, &code),
                    BridgeEvent::Packet(packet) => {
                        if let Err(err) = self.issue(
                            e,
                            &d,
                            BridgeCommand::Input {
                                session: s.id.clone(),
                                packet,
                            },
                        ) {
                            self.end(e, &d, &mut s, &err.code);
                        }
                    }
                }
            }
            self.sessions.insert(d, s);
        }
        for (d, packet) in e.take_bridge_output() {
            if let Some(mut s) = self.sessions.remove(&d) {
                if let Some(t) = s.transport.as_ref()
                    && let Err(err) = t.send(packet)
                {
                    changed = true;
                    self.end(e, &d, &mut s, &err.code);
                }
                self.sessions.insert(d, s);
            }
        }
        changed
    }
}
