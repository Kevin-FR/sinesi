//! Pure bridge input validation and Ethernet injection. Host sockets live in the adapter.
use super::*;
impl Network {
    pub fn bridge_session(&self, device: &Id) -> Option<&Id> {
        self.bridge_sessions.get(device).map(|s| &s.0)
    }
    pub(super) fn bridge_command(
        &mut self,
        c: &mut Context<'_>,
        id: &Id,
        cmd: &BridgeCommand,
    ) -> Result<(), Diagnostic> {
        let d = c
            .project
            .device(id)
            .filter(|d| {
                d.kind == DeviceKind::Modem
                    && d.interfaces.len() == 1
                    && d.interfaces[0].medium == Medium::Ethernet
            })
            .ok_or_else(|| Diagnostic::new("bridge.invalid_config", id.as_str()))?;
        match cmd {
            BridgeCommand::Activate { session } => {
                if self.bridge_sessions.contains_key(id) {
                    return Err(Diagnostic::new("bridge.already_connected", id.as_str()));
                }
                self.bridge_sessions
                    .insert(id.clone(), (session.clone(), 0, 0));
                c.fact("bridge.connected", id, json!({"session":session}));
            }
            BridgeCommand::Disconnect { session, reason } => {
                if reason.len() > 256 {
                    return Err(Diagnostic::new("bridge.invalid_message", "reason"));
                }
                if self
                    .bridge_sessions
                    .get(id)
                    .is_some_and(|s| &s.0 == session)
                {
                    self.bridge_sessions.remove(id);
                    self.bridge_output.retain(|(d, _)| d != id);
                    c.fact(
                        "bridge.disconnected",
                        id,
                        json!({"session":session,"reason":reason}),
                    );
                }
            }
            BridgeCommand::Input { session, packet } => {
                let Some(state) = self.bridge_sessions.get(id) else {
                    return Err(Diagnostic::new("bridge.not_connected", id.as_str()));
                };
                if &state.0 != session
                    || packet.sequence.0 != state.1 + 1
                    || packet.version != 1
                    || !(1..=64).contains(&packet.budget)
                    || serde_json::to_vec(packet).map_or(true, |v| v.len() > BRIDGE_MAX_PACKET)
                {
                    return Err(Diagnostic::new("bridge.invalid_message", id.as_str()));
                }
                let mut frame: Frame = serde_json::from_value(packet.frame.clone())
                    .map_err(|_| Diagnostic::new("bridge.invalid_message", "frame"))?;
                validate_frame(&frame)?;
                self.serial = self
                    .serial
                    .checked_add(1)
                    .ok_or_else(|| Diagnostic::new("resource.limit_exceeded", "bridge_serial"))?;
                frame.frame_id = Decimal(self.serial);
                frame.remaining_hops = packet.budget - 1;
                self.bridge_sessions.get_mut(id).unwrap().1 = packet.sequence.0;
                c.fact(
                    "bridge.input",
                    id,
                    json!({"session":session,"packet":packet}),
                );
                self.send(c, &d.interfaces[0].id, frame);
            }
        }
        Ok(())
    }
}
fn mac(s: &str) -> bool {
    s.len() == 17
        && s.split(':').count() == 6
        && s.split(':')
            .all(|p| p.len() == 2 && u8::from_str_radix(p, 16).is_ok())
}
fn validate_frame(f: &Frame) -> Result<(), Diagnostic> {
    let bad = || Diagnostic::new("bridge.invalid_message", "frame");
    if !mac(&f.source_mac)
        || !mac(&f.destination_mac)
        || !matches!(
            (&f.payload, f.ether_type),
            (Payload::Arp { .. }, 0x0806)
                | (Payload::Ipv4 { .. } | Payload::Ipv4Transport { .. }, 0x0800)
        )
    {
        return Err(bad());
    }
    fn payload(p: &Payload, depth: u8) -> bool {
        if depth > 4 {
            return false;
        }
        match p {
            Payload::Arp {
                operation,
                sender_mac,
                target_mac,
                ..
            } => matches!(operation, 1 | 2) && mac(sender_mac) && mac(target_mac),
            Payload::Ipv4 { ttl, icmp, .. } => {
                *ttl > 0
                    && matches!(icmp.message_type, 0 | 3 | 8 | 11)
                    && icmp.quoted.as_ref().is_none_or(|q| payload(q, depth + 1))
            }
            Payload::Ipv4Transport { ttl, packet, .. } => {
                *ttl > 0
                    && match packet {
                        TransportPacket::Tcp(t) => {
                            t.source_port != 0
                                && t.destination_port != 0
                                && t.data_base64.len() <= 87384
                                && base64::Engine::decode(
                                    &base64::engine::general_purpose::STANDARD,
                                    &t.data_base64,
                                )
                                .is_ok_and(|b| b.len() <= transport::MSS)
                        }
                        TransportPacket::Udp {
                            source_port,
                            destination_port,
                            length,
                            data_base64,
                        } => {
                            *source_port != 0
                                && *destination_port != 0
                                && base64::Engine::decode(
                                    &base64::engine::general_purpose::STANDARD,
                                    data_base64,
                                )
                                .is_ok_and(|b| b.len() + 8 == *length as usize)
                        }
                    }
            }
        }
    }
    if !payload(&f.payload, 0) {
        return Err(bad());
    }
    Ok(())
}
