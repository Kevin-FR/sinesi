//! Bounded Gnutella-like overlay over the simulator's TCP sockets. No host I/O.
use super::*;
use serde::Deserialize;
use sha2::{Digest, Sha256};
const MAX_WIRE: usize = 400_000;
const MAX_FILE: usize = 262_144;
#[derive(Clone, Debug, Default, Serialize)]
pub struct P2pView {
    pub neighbors: Vec<Endpoint>,
    pub discovered: Vec<Endpoint>,
    pub results: Vec<P2pResult>,
    pub searching: bool,
    pub downloading: bool,
}
#[derive(Clone, Default, serde::Serialize)]
pub(super) struct P2p {
    nodes: BTreeMap<(Id, Id), Node>,
}
#[derive(Clone, serde::Serialize)]
struct Node {
    endpoint: Endpoint,
    config: P2pConfig,
    connections: BTreeMap<Id, Connection>,
    seen: BTreeSet<Id>,
    query: Option<Id>,
    search_token: u64,
}
#[derive(Clone, serde::Serialize)]
struct Connection {
    peer: Option<Endpoint>,
    expected: Option<Endpoint>,
    buffer: Vec<u8>,
    mode: Mode,
    flow: Id,
}
#[derive(Clone, serde::Serialize)]
enum Mode {
    Pending,
    Neighbor,
    Download {
        result: P2pResult,
        destination: String,
        token: Id,
    },
    Serving,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "message", rename_all = "snake_case", deny_unknown_fields)]
enum Wire {
    Hello {
        version: u8,
        endpoint: Endpoint,
    },
    Ping {
        id: Id,
        ttl: u8,
    },
    Pong {
        id: Id,
        ttl: u8,
        endpoint: Endpoint,
    },
    Query {
        id: Id,
        ttl: u8,
        text: String,
    },
    QueryHit {
        id: Id,
        ttl: u8,
        query: Id,
        result: P2pResult,
    },
    Get {
        token: Id,
        path: String,
        sha256: String,
    },
    File {
        token: Id,
        data_base64: String,
    },
    Error {
        code: String,
    },
}
fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn valid_endpoint(e: Endpoint) -> bool {
    e.port != 0
        && !e.address.is_unspecified()
        && !e.address.is_multicast()
        && !e.address.is_broadcast()
}
fn emit(out: &mut Output, d: &Id, a: &Id, f: &Id, s: &Id, w: &Wire) {
    let mut bytes = serde_json::to_vec(w).expect("bounded wire DTO");
    bytes.push(b'\n');
    socket(
        out,
        d,
        f,
        SocketCommand::Send {
            socket: s.clone(),
            data_base64: STANDARD.encode(bytes),
        },
    );
    fact(
        out,
        d,
        f,
        "p2p.message.sent",
        json!({"application":a,"socket":s,"message":w}),
    );
}
fn close(out: &mut Output, d: &Id, f: &Id, s: &Id) {
    socket(
        out,
        d,
        f,
        SocketCommand::Close {
            socket: s.clone(),
            force: true,
        },
    );
}
fn flood(node: &Node, out: &mut Output, d: &Id, a: &Id, f: &Id, except: Option<&Id>, w: &Wire) {
    for (s, c) in &node.connections {
        if matches!(c.mode, Mode::Neighbor) && except != Some(s) {
            emit(out, d, a, f, s, w);
        }
    }
}
impl Runtime {
    pub(super) fn p2p_cancel(&mut self, d: &Id, a: &Id) {
        self.p2p.nodes.remove(&(d.clone(), a.clone()));
        if let Ok(s) = self.state(d, a) {
            s.p2p = P2pView::default();
        }
    }
    fn p2p_id(&mut self, prefix: &str) -> Id {
        self.serial += 1;
        Id::new(format!("p2p-{prefix}-{}", self.serial)).unwrap()
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Application command transaction context"
    )]
    pub(super) fn p2p_command(
        &mut self,
        p: &Project,
        net: &Network,
        d: &Id,
        a: &Id,
        cmd: &P2pCommand,
        f: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let metadata = p
            .device(d)
            .and_then(|h| h.applications.iter().find(|m| &m.id == a))
            .filter(|m| m.kind == "p2p")
            .ok_or_else(|| diag("application.invalid_operation", a.as_str()))?;
        let key = (d.clone(), a.clone());
        match cmd {
            P2pCommand::Join { peer } => {
                let cfg = match &metadata.service {
                    Some(ServiceConfig::P2p(c)) => c.clone(),
                    _ => P2pConfig::default(),
                };
                cfg.validate()?;
                if !self.p2p.nodes.contains_key(&key) {
                    let effective = net.effective_project(p);
                    let host = effective.device(d).unwrap();
                    let address = host
                        .interfaces
                        .iter()
                        .find_map(|i| match &i.ipv4 {
                            Some(Ipv4Config::Static { address, .. }) => {
                                address.parse::<Ipv4Addr>().ok()
                            }
                            _ => None,
                        })
                        .ok_or_else(|| diag("routing.no_route", d.as_str()))?;
                    let endpoint = Endpoint {
                        address,
                        port: cfg.port,
                    };
                    let listener = self.p2p_id("listener");
                    socket(
                        out,
                        d,
                        f,
                        SocketCommand::Bind {
                            socket: listener,
                            owner: a.clone(),
                            protocol: TransportProtocol::Tcp,
                            interface: None,
                            address: Some(address),
                            port: cfg.port,
                        },
                    );
                    self.p2p.nodes.insert(
                        key.clone(),
                        Node {
                            endpoint,
                            config: cfg.clone(),
                            connections: BTreeMap::new(),
                            seen: BTreeSet::new(),
                            query: None,
                            search_token: 0,
                        },
                    );
                    self.state(d, a)?.running = true;
                    fact(
                        out,
                        d,
                        f,
                        "p2p.joined",
                        json!({"application":a,"endpoint":endpoint}),
                    );
                }
                if let Some(target) = peer.or(cfg.bootstrap) {
                    let node = &self.p2p.nodes[&key];
                    if !valid_endpoint(target) || target == node.endpoint {
                        return Err(diag("p2p.invalid_peer", target.address.to_string()));
                    }
                    if node
                        .connections
                        .values()
                        .any(|c| c.peer == Some(target) || c.expected == Some(target))
                    {
                        return Ok(());
                    }
                    if node
                        .connections
                        .values()
                        .filter(|c| matches!(c.mode, Mode::Neighbor | Mode::Pending))
                        .count()
                        >= node.config.max_neighbors as usize
                    {
                        return Err(diag("p2p.neighbor_limit", a.as_str()));
                    }
                    let sid = self.p2p_id("peer");
                    self.p2p.nodes.get_mut(&key).unwrap().connections.insert(
                        sid.clone(),
                        Connection {
                            peer: None,
                            expected: Some(target),
                            buffer: vec![],
                            mode: Mode::Pending,
                            flow: f.clone(),
                        },
                    );
                    socket(
                        out,
                        d,
                        f,
                        SocketCommand::Connect {
                            socket: sid.clone(),
                            owner: a.clone(),
                            destination: target.address,
                            port: target.port,
                            local_address: None,
                            local_port: None,
                        },
                    );
                    out.effects.push(Effect::Timer(Timer {
                        device: d.clone(),
                        application: a.clone(),
                        token: 0,
                        kind: TimerKind::P2pSession { socket: sid },
                        delay_us: 30_000_000,
                        correlation: f.clone(),
                    }));
                }
            }
            P2pCommand::Reset => {
                self.p2p_cancel(d, a);
                self.state(d, a)?.running = false;
                socket(out, d, f, SocketCommand::CloseOwner { owner: a.clone() });
                fact(out, d, f, "p2p.reset", json!({"application":a}));
            }
            P2pCommand::CancelSearch | P2pCommand::ClearResults => {
                if let Some(n) = self.p2p.nodes.get_mut(&key) {
                    n.query = None;
                    n.search_token += 1;
                }
                let s = self.state(d, a)?;
                s.p2p.searching = false;
                if matches!(cmd, P2pCommand::ClearResults) {
                    s.p2p.results.clear();
                }
                fact(out, d, f, "p2p.search.cancelled", json!({"application":a}));
            }
            P2pCommand::Discover { ttl } | P2pCommand::Search { ttl, .. } => {
                if !(1..=16).contains(ttl) {
                    return Err(diag("p2p.invalid_ttl", ttl.to_string()));
                }
                let endpoint = self
                    .p2p
                    .nodes
                    .get(&key)
                    .ok_or_else(|| diag("p2p.not_joined", a.as_str()))?
                    .endpoint;
                let message_id =
                    self.p2p_id(&format!("message-{}-{}", endpoint.address, endpoint.port));
                let n = self
                    .p2p
                    .nodes
                    .get_mut(&key)
                    .ok_or_else(|| diag("p2p.not_joined", a.as_str()))?;
                if n.seen.len() >= 4096 {
                    return Err(diag("resource.limit_exceeded", "p2p_seen"));
                }
                n.seen.insert(message_id.clone());
                let wire = if let P2pCommand::Search { text, .. } = cmd {
                    if text.is_empty() || text.len() > 256 || text.chars().any(char::is_control) {
                        return Err(diag("p2p.invalid_query", text));
                    }
                    n.query = Some(message_id.clone());
                    n.search_token += 1;
                    out.effects.push(Effect::Timer(Timer {
                        device: d.clone(),
                        application: a.clone(),
                        token: n.search_token,
                        kind: TimerKind::P2pSearch,
                        delay_us: 30_000_000,
                        correlation: f.clone(),
                    }));
                    Wire::Query {
                        id: message_id,
                        ttl: *ttl,
                        text: text.clone(),
                    }
                } else {
                    Wire::Ping {
                        id: message_id,
                        ttl: *ttl,
                    }
                };
                flood(n, out, d, a, f, None, &wire);
                if matches!(cmd, P2pCommand::Search { .. }) {
                    let s = self.state(d, a)?;
                    s.p2p.results.clear();
                    s.p2p.searching = true;
                }
            }
            P2pCommand::Download {
                result,
                destination,
            } => {
                let s = self.state(d, a)?;
                if s.p2p.downloading {
                    return Err(diag("p2p.download_busy", a.as_str()));
                }
                let result = s
                    .p2p
                    .results
                    .iter()
                    .find(|r| &r.id == result)
                    .cloned()
                    .ok_or_else(|| diag("p2p.invalid_result", result.as_str()))?;
                let destination = sns_vfs::resolve("/", destination)?;
                if self.hosts[d].files.exists(&destination) {
                    return Err(diag("fs.already_exists", destination));
                }
                let sid = self.p2p_id("download");
                let token = self.p2p_id("transfer");
                let n = self
                    .p2p
                    .nodes
                    .get_mut(&key)
                    .ok_or_else(|| diag("p2p.not_joined", a.as_str()))?;
                n.connections.insert(
                    sid.clone(),
                    Connection {
                        peer: None,
                        expected: Some(result.source),
                        buffer: vec![],
                        mode: Mode::Download {
                            result: result.clone(),
                            destination,
                            token,
                        },
                        flow: f.clone(),
                    },
                );
                self.state(d, a)?.p2p.downloading = true;
                socket(
                    out,
                    d,
                    f,
                    SocketCommand::Connect {
                        socket: sid.clone(),
                        owner: a.clone(),
                        destination: result.source.address,
                        port: result.source.port,
                        local_address: None,
                        local_port: None,
                    },
                );
                out.effects.push(Effect::Timer(Timer {
                    device: d.clone(),
                    application: a.clone(),
                    token: 0,
                    kind: TimerKind::P2pSession { socket: sid },
                    delay_us: 30_000_000,
                    correlation: f.clone(),
                }));
            }
        }
        self.p2p_view(d, a)?;
        Ok(())
    }
    fn p2p_view(&mut self, d: &Id, a: &Id) -> Result<(), Diagnostic> {
        let peers = self
            .p2p
            .nodes
            .get(&(d.clone(), a.clone()))
            .map(|n| {
                n.connections
                    .values()
                    .filter(|c| matches!(c.mode, Mode::Neighbor))
                    .filter_map(|c| c.peer)
                    .collect::<BTreeSet<_>>()
                    .into_iter()
                    .collect()
            })
            .unwrap_or_default();
        self.state(d, a)?.p2p.neighbors = peers;
        Ok(())
    }
    pub(super) fn p2p_timer(&mut self, t: &Timer) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        let key = (t.device.clone(), t.application.clone());
        if !self.p2p.nodes.contains_key(&key) {
            return Ok(out);
        }
        if let Some(n) = self.p2p.nodes.get_mut(&key) {
            match &t.kind {
                TimerKind::P2pSession { socket }
                    if n.connections
                        .get(socket)
                        .is_some_and(|c| !matches!(c.mode, Mode::Neighbor)) =>
                {
                    n.connections.remove(socket);
                    close(&mut out, &t.device, &t.correlation, socket);
                    self.state(&t.device, &t.application)?.p2p.downloading = false;
                    fact(
                        &mut out,
                        &t.device,
                        &t.correlation,
                        "p2p.session.timeout",
                        json!({"socket":socket}),
                    );
                }
                TimerKind::P2pSearch if n.search_token == t.token => {
                    n.query = None;
                    self.state(&t.device, &t.application)?.p2p.searching = false;
                }
                _ => {}
            }
        }
        self.p2p_view(&t.device, &t.application)?;
        Ok(out)
    }
    pub(super) fn p2p_notice(
        &mut self,
        p: &Project,
        n: &SocketNotice,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        if !p.device(&n.device).is_some_and(|d| {
            d.applications
                .iter()
                .any(|a| a.id == n.owner && a.kind == "p2p")
        }) {
            return Ok(false);
        }
        let key = (n.device.clone(), n.owner.clone());
        let Some(mut node) = self.p2p.nodes.remove(&key) else {
            return Ok(true);
        };
        let sid = &n.socket;
        match &n.kind {
            SocketNoticeKind::Accepted => {
                if node.connections.len() >= node.config.max_neighbors as usize + 4 {
                    close(out, &n.device, &n.correlation, sid);
                } else {
                    node.connections.insert(
                        sid.clone(),
                        Connection {
                            peer: None,
                            expected: None,
                            buffer: vec![],
                            mode: Mode::Pending,
                            flow: n.correlation.clone(),
                        },
                    );
                    out.effects.push(Effect::Timer(Timer {
                        device: n.device.clone(),
                        application: n.owner.clone(),
                        token: 0,
                        kind: TimerKind::P2pSession {
                            socket: sid.clone(),
                        },
                        delay_us: 30_000_000,
                        correlation: n.correlation.clone(),
                    }));
                }
            }
            SocketNoticeKind::Connected => {
                if let Some(c) = node.connections.get(sid) {
                    match &c.mode {
                        Mode::Pending => emit(
                            out,
                            &n.device,
                            &n.owner,
                            &c.flow,
                            sid,
                            &Wire::Hello {
                                version: 1,
                                endpoint: node.endpoint,
                            },
                        ),
                        Mode::Download { result, token, .. } => emit(
                            out,
                            &n.device,
                            &n.owner,
                            &c.flow,
                            sid,
                            &Wire::Get {
                                token: token.clone(),
                                path: result.path.clone(),
                                sha256: result.sha256.clone(),
                            },
                        ),
                        _ => {}
                    }
                }
            }
            SocketNoticeKind::Closed { .. } | SocketNoticeKind::PeerClosed => {
                if let Some(c) = node.connections.remove(sid)
                    && matches!(c.mode, Mode::Download { .. })
                {
                    self.state(&n.device, &n.owner)?.p2p.downloading = false;
                }
                if matches!(n.kind, SocketNoticeKind::PeerClosed) {
                    close(out, &n.device, &n.correlation, sid);
                }
            }
            SocketNoticeKind::Data { data_base64 } => {
                if let Some(mut conn) = node.connections.remove(sid) {
                    let bytes = STANDARD.decode(data_base64).unwrap_or_default();
                    conn.buffer.extend(bytes);
                    let mut keep = conn.buffer.len() <= MAX_WIRE;
                    let mut count = 0;
                    while keep {
                        let Some(pos) = conn.buffer.iter().position(|b| *b == b'\n') else {
                            break;
                        };
                        count += 1;
                        if count > 1024 {
                            keep = false;
                            break;
                        }
                        let line = conn.buffer.drain(..=pos).collect::<Vec<_>>();
                        let wire = serde_json::from_slice::<Wire>(&line);
                        match wire {
                            Ok(w) => {
                                fact(
                                    out,
                                    &n.device,
                                    &conn.flow,
                                    "p2p.message.received",
                                    json!({"socket":sid,"message":w}),
                                );
                                match self.p2p_wire(&mut node, &mut conn, n, w, out) {
                                    Ok(k) => keep = k,
                                    Err(e) => {
                                        push(
                                            self.state(&n.device, &n.owner)?,
                                            Line::Error { diagnostic: e },
                                        );
                                        keep = false;
                                    }
                                }
                            }
                            Err(_) => {
                                push(
                                    self.state(&n.device, &n.owner)?,
                                    Line::Error {
                                        diagnostic: diag("p2p.invalid_message", sid.as_str()),
                                    },
                                );
                                keep = false;
                            }
                        }
                    }
                    if keep {
                        node.connections.insert(sid.clone(), conn);
                    } else {
                        if matches!(conn.mode, Mode::Download { .. }) {
                            self.state(&n.device, &n.owner)?.p2p.downloading = false;
                        }
                        socket(
                            out,
                            &n.device,
                            &conn.flow,
                            SocketCommand::Close {
                                socket: sid.clone(),
                                force: !matches!(conn.mode, Mode::Serving),
                            },
                        );
                    }
                }
            }
            _ => {}
        }
        self.p2p.nodes.insert(key, node);
        self.p2p_view(&n.device, &n.owner)?;
        Ok(true)
    }
    fn p2p_wire(
        &mut self,
        node: &mut Node,
        c: &mut Connection,
        n: &SocketNotice,
        w: Wire,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        let d = &n.device;
        let a = &n.owner;
        let sid = &n.socket;
        let f = &c.flow;
        match w {
            Wire::Hello { version, endpoint } => {
                if version != 1
                    || !valid_endpoint(endpoint)
                    || endpoint == node.endpoint
                    || !matches!(c.mode, Mode::Pending)
                    || c.expected.is_some_and(|e| e != endpoint)
                {
                    return Err(diag("p2p.invalid_peer", sid.as_str()));
                }
                if node
                    .connections
                    .values()
                    .filter(|c| matches!(c.mode, Mode::Neighbor | Mode::Pending))
                    .count()
                    >= node.config.max_neighbors as usize
                    || node
                        .connections
                        .values()
                        .any(|c| matches!(c.mode, Mode::Neighbor) && c.peer == Some(endpoint))
                {
                    emit(
                        out,
                        d,
                        a,
                        f,
                        sid,
                        &Wire::Error {
                            code: "p2p.neighbor_limit".into(),
                        },
                    );
                    return Ok(false);
                }
                if c.expected.is_none() {
                    emit(
                        out,
                        d,
                        a,
                        f,
                        sid,
                        &Wire::Hello {
                            version: 1,
                            endpoint: node.endpoint,
                        },
                    );
                }
                c.peer = Some(endpoint);
                c.mode = Mode::Neighbor;
            }
            Wire::Get {
                token,
                path,
                sha256,
            } => {
                if !matches!(c.mode, Mode::Pending) {
                    return Err(diag("p2p.invalid_message", sid.as_str()));
                }
                c.mode = Mode::Serving;
                if !shared(&node.config.shared_directory, &path) {
                    return Err(diag("fs.invalid_path", path));
                }
                let data = self.hosts[d].files.read(&path)?;
                if data.len() > MAX_FILE || hash(data) != sha256 {
                    return Err(diag("p2p.file_changed", path));
                }
                emit(
                    out,
                    d,
                    a,
                    f,
                    sid,
                    &Wire::File {
                        token,
                        data_base64: STANDARD.encode(data),
                    },
                );
                // Graceful close is queued after the bytes; forced close would discard them.
                return Ok(false);
            }
            Wire::File { token, data_base64 } => {
                let Mode::Download {
                    result,
                    destination,
                    token: expected,
                } = &c.mode
                else {
                    return Err(diag("p2p.invalid_message", sid.as_str()));
                };
                let bytes = STANDARD
                    .decode(data_base64)
                    .map_err(|_| diag("p2p.invalid_message", sid.as_str()))?;
                if token != *expected
                    || bytes.len() > MAX_FILE
                    || bytes.len() != result.bytes as usize
                    || hash(&bytes) != result.sha256
                {
                    return Err(diag("p2p.file_changed", &result.path));
                }
                let files = &mut self.hosts.get_mut(d).unwrap().files;
                files.write(destination, &bytes, &result.media_type, false)?;
                out.effects.push(Effect::Files {
                    device: d.clone(),
                    files: files.to_files(),
                });
                self.state(d, a)?.p2p.downloading = false;
                fact(
                    out,
                    d,
                    f,
                    "p2p.download.completed",
                    json!({"application":a,"source":result.source,"path":destination,"sha256":hash(&bytes),"bytes":bytes.len()}),
                );
                return Ok(false);
            }
            Wire::Error { code } => {
                push(
                    self.state(d, a)?,
                    Line::Error {
                        diagnostic: diag("p2p.remote_error", code),
                    },
                );
                return Ok(false);
            }
            wire => {
                if !matches!(c.mode, Mode::Neighbor) {
                    return Err(diag("p2p.invalid_message", sid.as_str()));
                }
                let (id, ttl) = match &wire {
                    Wire::Ping { id, ttl }
                    | Wire::Pong { id, ttl, .. }
                    | Wire::Query { id, ttl, .. }
                    | Wire::QueryHit { id, ttl, .. } => (id, *ttl),
                    _ => unreachable!(),
                };
                if !(1..=16).contains(&ttl) {
                    return Err(diag("p2p.invalid_ttl", ttl.to_string()));
                }
                if node.seen.contains(id) {
                    fact(out, d, f, "p2p.message.duplicate", json!({"id":id}));
                    return Ok(true);
                }
                if node.seen.len() >= 4096 {
                    return Err(diag("resource.limit_exceeded", "p2p_seen"));
                }
                node.seen.insert(id.clone());
                let forward = match wire {
                    Wire::Ping { id, ttl } => {
                        let reply = Wire::Pong {
                            id: self.p2p_id(&format!(
                                "pong-{}-{}",
                                node.endpoint.address, node.endpoint.port
                            )),
                            ttl: 16,
                            endpoint: node.endpoint,
                        };
                        emit(out, d, a, f, sid, &reply);
                        flood(node, out, d, a, f, Some(sid), &reply);
                        Wire::Ping { id, ttl: ttl - 1 }
                    }
                    Wire::Pong { id, ttl, endpoint } => {
                        if !valid_endpoint(endpoint) {
                            return Err(diag("p2p.invalid_peer", sid.as_str()));
                        }
                        let view = &mut self.state(d, a)?.p2p;
                        if endpoint != node.endpoint
                            && !view.discovered.contains(&endpoint)
                            && view.discovered.len() < 128
                        {
                            view.discovered.push(endpoint);
                            view.discovered.sort();
                        }
                        Wire::Pong {
                            id,
                            ttl: ttl - 1,
                            endpoint,
                        }
                    }
                    Wire::Query { id, ttl, text } => {
                        if text.is_empty() || text.len() > 256 {
                            return Err(diag("p2p.invalid_query", text));
                        }
                        let files = self.hosts[d]
                            .files
                            .to_files()
                            .into_iter()
                            .filter(|e| shared(&node.config.shared_directory, &e.path))
                            .filter_map(|e| {
                                let bytes = self.hosts[d].files.read(&e.path).ok()?.len();
                                Some(Entry {
                                    path: e.path,
                                    directory: false,
                                    media_type: e.media_type,
                                    bytes,
                                })
                            })
                            .collect::<Vec<_>>();
                        for e in files
                            .into_iter()
                            .filter(|e| {
                                !e.directory
                                    && e.bytes <= MAX_FILE
                                    && e.path.to_lowercase().contains(&text.to_lowercase())
                            })
                            .take(16)
                        {
                            let result = P2pResult {
                                id: self.p2p_id(&format!(
                                    "result-{}-{}",
                                    node.endpoint.address, node.endpoint.port
                                )),
                                source: node.endpoint,
                                sha256: hash(self.hosts[d].files.read(&e.path)?),
                                path: e.path,
                                media_type: e.media_type,
                                bytes: e.bytes as u32,
                            };
                            let reply = Wire::QueryHit {
                                id: self.p2p_id(&format!(
                                    "hit-{}-{}",
                                    node.endpoint.address, node.endpoint.port
                                )),
                                ttl: 16,
                                query: id.clone(),
                                result,
                            };
                            emit(out, d, a, f, sid, &reply);
                            flood(node, out, d, a, f, Some(sid), &reply);
                        }
                        Wire::Query {
                            id,
                            ttl: ttl - 1,
                            text,
                        }
                    }
                    Wire::QueryHit {
                        id,
                        ttl,
                        query,
                        result,
                    } => {
                        if !valid_endpoint(result.source)
                            || !sns_model::virtual_path_valid(&result.path)
                            || result.bytes as usize > MAX_FILE
                            || result.sha256.len() != 64
                            || !result.sha256.bytes().all(|b| b.is_ascii_hexdigit())
                            || result.media_type.len() > 256
                        {
                            return Err(diag("p2p.invalid_result", result.id.as_str()));
                        }
                        if node.query.as_ref() == Some(&query) {
                            let view = &mut self.state(d, a)?.p2p;
                            if view.results.len() < 128
                                && !view
                                    .results
                                    .iter()
                                    .any(|r| r.source == result.source && r.path == result.path)
                            {
                                view.results.push(result.clone());
                            }
                        }
                        Wire::QueryHit {
                            id,
                            ttl: ttl - 1,
                            query,
                            result,
                        }
                    }
                    _ => unreachable!(),
                };
                if ttl > 1 {
                    flood(node, out, d, a, f, Some(sid), &forward);
                } else {
                    fact(out, d, f, "p2p.message.ttl_expired", json!({"socket":sid}));
                }
            }
        }
        Ok(true)
    }
}
fn shared(root: &str, path: &str) -> bool {
    sns_model::virtual_path_valid(path) && (root == "/" || path.starts_with(&format!("{root}/")))
}
