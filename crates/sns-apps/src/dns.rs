//! Educational DNS messages use UDP in the simulator, with bounded iterative referrals.
use super::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Default, serde::Serialize)]
pub(super) struct Dns {
    pending: BTreeMap<Id, Pending>,
    cache: BTreeMap<(Id, String, DnsType), Cached>,
}
#[derive(Clone, serde::Serialize)]
struct Cached {
    reply: Wire,
    expires: u64,
}
#[derive(Clone, serde::Serialize)]
struct Pending {
    device: Id,
    app: Id,
    flow: Id,
    wire: Wire,
    server: Ipv4Addr,
    purpose: Purpose,
    token: u64,
}
#[derive(Clone, serde::Serialize)]
pub(super) enum Purpose {
    Mail {
        job: u64,
        resolve: super::mail::Resolve,
    },
    Terminal,
    Browser {
        url: super::web::Url,
        fields: Option<BTreeMap<String, String>>,
        generation: u64,
        resource: bool,
    },
    Connect {
        port: u16,
    },
    Command {
        line: String,
    },
    Forward {
        socket: Id,
        client: Endpoint,
        query: Wire,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Wire {
    id: u64,
    name: String,
    record_type: DnsType,
    recursive: bool,
    path: Vec<Ipv4Addr>,
    code: String,
    records: Vec<DnsRecord>,
    additional: Vec<DnsRecord>,
    referral: Option<Ipv4Addr>,
    server: Option<Ipv4Addr>,
}
impl Runtime {
    #[expect(
        clippy::too_many_arguments,
        reason = "Simulation context and actor are explicit; effects remain transactional."
    )]
    pub(super) fn dns_query(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        device: &Id,
        app: &Id,
        name: &str,
        record_type: DnsType,
        server: Option<Ipv4Addr>,
        purpose: Purpose,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        if self.dns.pending.len() >= 256 {
            return Err(diag("resource.limit_exceeded", "dns_queries"));
        }
        let name = dns_name(name)?;
        let key = (device.clone(), name.clone(), record_type);
        if let Some(cached) = self
            .dns
            .cache
            .get(&key)
            .filter(|c| c.expires > now && server.is_none())
            .cloned()
        {
            let mut reply = cached.reply;
            let ttl = ((cached.expires - now) / 1_000_000) as u32;
            for r in reply.records.iter_mut().chain(&mut reply.additional) {
                r.ttl_seconds = r.ttl_seconds.min(ttl);
            }
            fact(
                out,
                device,
                flow,
                "dns.cache.hit",
                json!({"name":name,"record_type":record_type}),
            );
            return self.dns_done(project, net, now, device, app, purpose, reply, flow, out);
        }
        let effective = net.effective_project(project);
        let server = server
            .or_else(|| {
                effective
                    .device(device)?
                    .network
                    .dns_server
                    .as_ref()?
                    .parse()
                    .ok()
            })
            .ok_or_else(|| diag("dns.no_server", name.clone()))?;
        if self.dns.pending.len() >= 256 {
            return Err(diag("resource.limit_exceeded", "dns_queries"));
        }
        self.serial += 1;
        let token = self.serial;
        let socket_id = Id::new(format!("dns-query-{token}"))?;
        let wire = Wire {
            id: token,
            name,
            record_type,
            recursive: true,
            path: vec![],
            code: "query".into(),
            records: vec![],
            additional: vec![],
            referral: None,
            server: Some(server),
        };
        let p = Pending {
            device: device.clone(),
            app: app.clone(),
            flow: flow.clone(),
            wire,
            server,
            purpose,
            token,
        };
        socket(
            out,
            device,
            flow,
            SocketCommand::Bind {
                socket: socket_id.clone(),
                owner: app.clone(),
                protocol: TransportProtocol::Udp,
                interface: None,
                address: None,
                port: 0,
            },
        );
        self.dns_send(&socket_id, &p, out)?;
        self.dns.pending.insert(socket_id, p);
        Ok(())
    }
    fn dns_send(&self, id: &Id, p: &Pending, out: &mut Output) -> Result<(), Diagnostic> {
        socket(
            out,
            &p.device,
            &p.flow,
            SocketCommand::SendTo {
                socket: id.clone(),
                destination: p.server,
                port: 53,
                data_base64: STANDARD.encode(serde_json::to_vec(&p.wire).unwrap()),
            },
        );
        out.effects.push(Effect::Timer(Timer {
            device: p.device.clone(),
            application: p.app.clone(),
            token: p.token,
            kind: TimerKind::DnsTimeout { socket: id.clone() },
            delay_us: 3_000_000,
            correlation: p.flow.clone(),
        }));
        fact(
            out,
            &p.device,
            &p.flow,
            "dns.query.sent",
            json!({"name":p.wire.name,"record_type":p.wire.record_type,"server":p.server,"socket":id}),
        );
        Ok(())
    }
    pub(super) fn dns_notice(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        n: &SocketNotice,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        if let SocketNoticeKind::Closed { .. } = &n.kind
            && let Some(p) = self.dns.pending.remove(&n.socket)
        {
            if self.state(&p.device, &p.app).is_ok() {
                push(
                    self.state(&p.device, &p.app)?,
                    Line::Error {
                        diagnostic: diag("dns.timeout", p.wire.name),
                    },
                );
                self.browser_failed(&p.device, &p.app);
            }
            return Ok(true);
        }
        let SocketNoticeKind::Datagram {
            source,
            destination,
            data_base64,
            ..
        } = &n.kind
        else {
            return Ok(false);
        };
        if !self.dns.pending.contains_key(&n.socket)
            && project
                .device(&n.device)
                .and_then(|d| d.applications.iter().find(|a| a.id == n.owner))
                .is_none_or(|a| a.kind != "dns_server")
        {
            return Ok(false);
        }
        let Ok(bytes) = STANDARD.decode(data_base64) else {
            return Ok(true);
        };
        if bytes.len() > 60_000 {
            return Ok(true);
        }
        let Ok(mut w) = serde_json::from_slice::<Wire>(&bytes) else {
            return Ok(true);
        };
        if w.path.len() > 8 || w.name.len() > 253 {
            return Ok(true);
        }
        if let Some(mut pending) = self.dns.pending.get(&n.socket).cloned() {
            if source.address != pending.server
                || source.port != 53
                || w.id != pending.wire.id
                || w.name != pending.wire.name
                || w.record_type != pending.wire.record_type
                || w.code == "query"
            {
                return Ok(true);
            }
            if let Some(server) = w.referral {
                if pending.wire.path.len() >= 8
                    || pending.wire.path.contains(&server)
                    || server == pending.server
                {
                    w.code = "loop".into();
                    w.referral = None;
                } else {
                    pending.wire.path.push(pending.server);
                    pending.server = server;
                    self.serial += 1;
                    pending.token = self.serial;
                    self.dns_send(&n.socket, &pending, out)?;
                    self.dns.pending.insert(n.socket.clone(), pending);
                    return Ok(true);
                }
            }
            self.dns.pending.remove(&n.socket);
            socket(
                out,
                &n.device,
                &n.correlation,
                SocketCommand::Close {
                    socket: n.socket.clone(),
                    force: false,
                },
            );
            if w.code == "ok"
                && let Some(ttl) = w
                    .records
                    .iter()
                    .chain(&w.additional)
                    .map(|r| r.ttl_seconds)
                    .min()
                    .filter(|t| *t > 0)
            {
                if self.dns.cache.len() >= 1024 {
                    self.dns.cache.retain(|_, c| c.expires > now);
                    if self.dns.cache.len() >= 1024 {
                        let first = self.dns.cache.keys().next().unwrap().clone();
                        self.dns.cache.remove(&first);
                    }
                }
                self.dns.cache.insert(
                    (n.device.clone(), w.name.clone(), w.record_type),
                    Cached {
                        reply: w.clone(),
                        expires: now.saturating_add(ttl as u64 * 1_000_000),
                    },
                );
            }
            fact(
                out,
                &n.device,
                &pending.flow,
                "dns.response.received",
                json!({"server":source,"response":w}),
            );
            self.dns_done(
                project,
                net,
                now,
                &pending.device,
                &pending.app,
                pending.purpose,
                w,
                &pending.flow,
                out,
            )?;
        } else {
            if w.code != "query" || !self.state(&n.device, &n.owner)?.running {
                return Ok(true);
            }
            let Some(ServiceConfig::Dns(cfg)) = project
                .device(&n.device)
                .and_then(|d| d.applications.iter().find(|a| a.id == n.owner))
                .and_then(|a| a.service.clone())
            else {
                return Ok(true);
            };
            let original = w.clone();
            let Ok(name) = dns_name(&w.name) else {
                return Ok(true);
            };
            w.name = name.clone();
            w.code = "ok".into();
            w.server = Some(destination.address);
            w.records.clear();
            w.additional.clear();
            w.referral = None;
            w.records = cfg
                .records
                .iter()
                .filter(|r| {
                    dns_name(&r.name).ok().as_ref() == Some(&name) && r.record_type == w.record_type
                })
                .cloned()
                .collect();
            for r in &mut w.records {
                r.name = dns_name(&r.name)?;
                if r.record_type != DnsType::A {
                    r.value = dns_name(&r.value)?;
                }
            }
            for r in &w.records {
                if r.record_type == DnsType::Mx {
                    w.additional.extend(
                        cfg.records
                            .iter()
                            .filter(|a| {
                                a.record_type == DnsType::A
                                    && dns_name(&a.name).ok().as_ref() == Some(&r.value)
                            })
                            .cloned(),
                    );
                }
            }
            if w.records.is_empty() {
                let delegation = cfg
                    .records
                    .iter()
                    .filter(|r| r.record_type == DnsType::Ns)
                    .filter_map(|r| dns_name(&r.name).ok().map(|name| (name, r)))
                    .filter(|(zone, _)| {
                        zone.is_empty() || name == *zone || name.ends_with(&format!(".{zone}"))
                    })
                    .max_by_key(|(zone, _)| zone.len())
                    .map(|(_, r)| r);
                let next = delegation
                    .and_then(|ns| {
                        cfg.records.iter().find(|r| {
                            r.record_type == DnsType::A
                                && dns_name(&r.name).ok() == dns_name(&ns.value).ok()
                        })
                    })
                    .and_then(|a| a.value.parse().ok())
                    .or(cfg.upstream);
                if let Some(server) = next {
                    if original.path.contains(&server) || original.path.len() >= 8 {
                        w.code = "loop".into();
                    } else if cfg.recursive && original.recursive {
                        self.dns_query_forward(n, source, original, server, out)?;
                        return Ok(true);
                    } else {
                        w.referral = Some(server);
                        w.code = "referral".into();
                    }
                } else {
                    w.code = if cfg
                        .records
                        .iter()
                        .any(|r| dns_name(&r.name).ok().as_ref() == Some(&name))
                    {
                        "nodata"
                    } else {
                        "nxdomain"
                    }
                    .into();
                }
            }
            Self::dns_reply(out, n, &w, source);
        }
        Ok(true)
    }
    fn dns_reply(out: &mut Output, n: &SocketNotice, w: &Wire, client: &Endpoint) {
        let mut w = w.clone();
        if serde_json::to_vec(&w).unwrap().len() > 60_000 {
            w.code = "too_large".into();
            w.records.clear();
            w.additional.clear();
            w.referral = None;
        }
        socket(
            out,
            &n.device,
            &n.correlation,
            SocketCommand::SendTo {
                socket: n.socket.clone(),
                destination: client.address,
                port: client.port,
                data_base64: STANDARD.encode(serde_json::to_vec(&w).unwrap()),
            },
        );
        fact(
            out,
            &n.device,
            &n.correlation,
            "dns.response.sent",
            json!({"client":client,"response":w}),
        );
    }
    fn dns_query_forward(
        &mut self,
        n: &SocketNotice,
        client: &Endpoint,
        query: Wire,
        server: Ipv4Addr,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        if self.dns.pending.len() >= 256 {
            return Err(diag("resource.limit_exceeded", "dns_queries"));
        }
        self.serial += 1;
        let token = self.serial;
        let socket_id = Id::new(format!("dns-query-{token}"))?;
        let mut wire = query.clone();
        wire.id = token;
        wire.path.push(client.address);
        let p = Pending {
            device: n.device.clone(),
            app: n.owner.clone(),
            flow: n.correlation.clone(),
            wire,
            server,
            purpose: Purpose::Forward {
                socket: n.socket.clone(),
                client: *client,
                query,
            },
            token,
        };
        socket(
            out,
            &n.device,
            &n.correlation,
            SocketCommand::Bind {
                socket: socket_id.clone(),
                owner: n.owner.clone(),
                protocol: TransportProtocol::Udp,
                interface: None,
                address: None,
                port: 0,
            },
        );
        self.dns_send(&socket_id, &p, out)?;
        self.dns.pending.insert(socket_id, p);
        Ok(())
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Simulation context and actor are explicit; effects remain transactional."
    )]
    fn dns_done(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        device: &Id,
        app: &Id,
        purpose: Purpose,
        reply: Wire,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        if let Purpose::Mail { job, resolve } = purpose {
            return self.mail_dns(
                super::mail::Context { project, net, now },
                job,
                resolve,
                &reply.code,
                &reply.records,
                out,
            );
        }
        if let Purpose::Forward {
            socket: listener,
            client,
            query,
        } = purpose
        {
            let mut reply = reply;
            reply.id = query.id;
            reply.path = query.path;
            let n = SocketNotice {
                device: device.clone(),
                owner: app.clone(),
                socket: listener,
                correlation: flow.clone(),
                kind: SocketNoticeKind::Listening,
            };
            Self::dns_reply(out, &n, &reply, &client);
            return Ok(());
        }
        if matches!(purpose, Purpose::Terminal) {
            push(
                self.state(device, app)?,
                Line::Json {
                    value: serde_json::to_value(reply).unwrap(),
                },
            );
            return Ok(());
        }
        let address = reply
            .records
            .iter()
            .find(|r| r.record_type == DnsType::A)
            .and_then(|r| r.value.parse().ok());
        let Some(address) = address.filter(|_| reply.code == "ok") else {
            push(
                self.state(device, app)?,
                Line::Error {
                    diagnostic: diag(&format!("dns.{}", reply.code), reply.name),
                },
            );
            self.browser_failed(device, app);
            return Ok(());
        };
        match purpose {
            Purpose::Browser {
                url,
                fields,
                generation,
                resource,
            } => self.web_connect(
                device, app, url, address, fields, generation, resource, flow, out,
            )?,
            Purpose::Connect { port } => {
                socket(
                    out,
                    device,
                    flow,
                    SocketCommand::Connect {
                        socket: app.clone(),
                        owner: app.clone(),
                        destination: address,
                        port,
                        local_address: None,
                        local_port: None,
                    },
                );
            }
            Purpose::Command { line } => {
                let mut p = net.effective_project(project);
                let line = line.replace("$DNS", &address.to_string());
                self.evaluate(
                    terminal::Request {
                        project: &mut p,
                        net,
                        now,
                        device,
                        application: app,
                        line: &line,
                        flow,
                    },
                    out,
                )?;
            }
            _ => {}
        }
        Ok(())
    }
    pub(super) fn dns_timeout(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        timer: &Timer,
        id: &Id,
    ) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        if let Some(p) = self
            .dns
            .pending
            .get(id)
            .filter(|p| p.token == timer.token)
            .cloned()
        {
            self.dns.pending.remove(id);
            socket(
                &mut out,
                &p.device,
                &p.flow,
                SocketCommand::Close {
                    socket: id.clone(),
                    force: false,
                },
            );
            let mut reply = p.wire;
            reply.code = "timeout".into();
            reply.referral = None;
            fact(
                &mut out,
                &p.device,
                &p.flow,
                "dns.timeout",
                json!({"server":p.server,"name":reply.name}),
            );
            self.dns_done(
                project, net, now, &p.device, &p.app, p.purpose, reply, &p.flow, &mut out,
            )?;
        }
        Ok(out)
    }
    pub(super) fn cancel_service_queries(&mut self, device: &Id, app: &Id) {
        self.dns
            .pending
            .retain(|_, p| &p.device != device || &p.app != app);
    }
    pub(super) fn cancel_browser_dns(
        &mut self,
        device: &Id,
        app: &Id,
        flow: &Id,
        out: &mut Output,
    ) {
        let ids: Vec<_> = self
            .dns
            .pending
            .iter()
            .filter(|(_, p)| {
                &p.device == device && &p.app == app && matches!(p.purpose, Purpose::Browser { .. })
            })
            .map(|(id, _)| id.clone())
            .collect();
        for id in ids {
            self.dns.pending.remove(&id);
            socket(
                out,
                device,
                flow,
                SocketCommand::Close {
                    socket: id,
                    force: false,
                },
            );
        }
    }
    pub(super) fn purge_dns(&mut self, device: &Id) {
        self.dns.cache.retain(|(d, _, _), _| d != device);
    }
}
