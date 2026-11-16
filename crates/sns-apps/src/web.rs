//! A controlled HTTP/1.1 client/server. All bytes travel through simulated TCP.
#[path = "admin.rs"]
mod admin;
use super::*;
#[derive(Clone, Debug, Default, Serialize)]
pub struct BrowserView {
    pub url: String,
    pub status: u16,
    pub html: String,
    pub loading: bool,
    pub generation: u64,
    pub images: BTreeMap<String, ImageResource>,
}
#[derive(Clone, Debug, Serialize)]
pub struct ImageResource {
    pub media_type: String,
    pub data_base64: String,
}
#[derive(Clone, Default, serde::Serialize)]
pub(super) struct Web {
    clients: BTreeMap<(Id, Id), Client>,
    servers: BTreeMap<(Id, Id), Vec<u8>>,
}
#[derive(Clone, serde::Serialize)]
struct Client {
    app: Id,
    url: Url,
    generation: u64,
    resource: bool,
    request: Vec<u8>,
    bytes: Vec<u8>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct Url {
    pub host: String,
    pub port: u16,
    pub path: String,
}
impl Url {
    pub fn parse(input: &str, base: Option<&str>) -> Result<Self, Diagnostic> {
        if input.len() > 4096
            || input
                .chars()
                .any(|c| c.is_control() || c == '\\' || c == ' ')
        {
            return Err(diag("http.invalid_url", input));
        }
        if !input.starts_with("http://") {
            if input.contains(':') || input.starts_with("//") {
                return Err(diag("http.unsupported_scheme", input));
            }
            let mut u = Self::parse(base.ok_or_else(|| diag("http.invalid_url", input))?, None)?;
            let path = input.split('#').next().unwrap_or("");
            if !path.is_empty() {
                u.path = if path.starts_with('/') {
                    path.into()
                } else {
                    format!(
                        "{}{path}",
                        u.path
                            .split('?')
                            .next()
                            .unwrap()
                            .rsplit_once('/')
                            .map(|(p, _)| p.to_string() + "/")
                            .unwrap_or("/".into())
                    )
                };
            }
            u.path = normalize_path(&u.path)?;
            return Ok(u);
        }
        let rest = &input[7..];
        let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
        if authority.contains(['@', '?', '#']) || authority.is_empty() {
            return Err(diag("http.invalid_url", input));
        }
        let (host, port) = if let Some((h, p)) = authority.split_once(':') {
            (
                h,
                p.parse::<u16>()
                    .map_err(|_| diag("http.invalid_url", input))?,
            )
        } else {
            (authority, 80)
        };
        if port == 0 {
            return Err(diag("http.invalid_url", input));
        }
        let host = dns_name(host)?;
        if host.is_empty() {
            return Err(diag("http.invalid_url", input));
        }
        let path = normalize_path(&format!("/{}", path.split('#').next().unwrap_or("")))?;
        Ok(Self { host, port, path })
    }
    pub fn full(&self) -> String {
        format!("http://{}:{}{}", self.host, self.port, self.path)
    }
}
fn normalize_path(path: &str) -> Result<String, Diagnostic> {
    let (p, q) = path
        .split_once('?')
        .map(|(p, q)| (p, Some(q)))
        .unwrap_or((path, None));
    let mut segments = vec![];
    for part in p.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                segments.pop();
            }
            s => segments.push(s),
        }
    }
    let path = format!("/{}", segments.join("/"));
    Ok(match q {
        Some(q) => format!("{path}?{q}"),
        None => path,
    })
}
fn decode(input: &str, form: bool) -> Result<String, Diagnostic> {
    let b = input.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            if i + 2 >= b.len() {
                return Err(diag("http.invalid_encoding", input));
            }
            let h = std::str::from_utf8(&b[i + 1..i + 3])
                .map_err(|_| diag("http.invalid_encoding", input))?;
            out.push(u8::from_str_radix(h, 16).map_err(|_| diag("http.invalid_encoding", input))?);
            i += 3;
        } else {
            out.push(if form && b[i] == b'+' { b' ' } else { b[i] });
            i += 1;
        }
    }
    let out = String::from_utf8(out).map_err(|_| diag("http.invalid_encoding", input))?;
    if out.chars().any(|c| c.is_control() || c == '\\') {
        return Err(diag("http.invalid_encoding", input));
    }
    Ok(out)
}
fn encode(input: &str) -> String {
    input
        .bytes()
        .map(|c| {
            if c.is_ascii_alphanumeric() || b"-._~".contains(&c) {
                (c as char).to_string()
            } else {
                format!("%{c:02X}")
            }
        })
        .collect()
}
fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
struct Message {
    first: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}
/// No transfer-encoding or duplicate framing fields; incomplete frames remain buffered.
fn parse(bytes: &[u8]) -> Result<Option<Message>, Diagnostic> {
    if bytes.len() > MAX_RECEIVED_BYTES + 16384 {
        return Err(diag("http.message_too_large", "HTTP"));
    }
    let Some(end) = bytes.windows(4).position(|s| s == b"\r\n\r\n") else {
        if bytes.len() > 16384 {
            return Err(diag("http.invalid_message", "headers"));
        }
        return Ok(None);
    };
    if end > 16384 {
        return Err(diag("http.invalid_message", "headers"));
    }
    let text =
        std::str::from_utf8(&bytes[..end]).map_err(|_| diag("http.invalid_message", "headers"))?;
    let mut lines = text.split("\r\n");
    let first = lines.next().unwrap_or("").to_string();
    let mut headers = BTreeMap::new();
    for line in lines {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| diag("http.invalid_message", "header"))?;
        let name = name.to_ascii_lowercase();
        if name.is_empty()
            || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
            || headers.insert(name, value.trim().to_string()).is_some()
        {
            return Err(diag("http.invalid_message", "header"));
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err(diag("http.invalid_message", "transfer_encoding"));
    }
    if headers
        .get("content-length")
        .is_some_and(|v| v.is_empty() || !v.bytes().all(|c| c.is_ascii_digit()))
    {
        return Err(diag("http.invalid_message", "length"));
    }
    let len = headers
        .get("content-length")
        .map(|s| s.parse::<usize>())
        .transpose()
        .map_err(|_| diag("http.invalid_message", "length"))?
        .unwrap_or(0);
    if len > MAX_RECEIVED_BYTES {
        return Err(diag("http.message_too_large", "body"));
    }
    if bytes.len() < end + 4 + len {
        return Ok(None);
    }
    if bytes.len() > end + 4 + len {
        return Err(diag("http.invalid_message", "trailing_bytes"));
    }
    Ok(Some(Message {
        first,
        headers,
        body: bytes[end + 4..].to_vec(),
    }))
}
fn response(status: u16, media: &str, body: Vec<u8>) -> Vec<u8> {
    if body.len() > MAX_RECEIVED_BYTES - 16384 {
        return response(413, "text/plain", b"Content too large".to_vec());
    }
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        501 => "Not Implemented",
        413 => "Content Too Large",
        409 => "Conflict",
        _ => "Error",
    };
    let mut bytes=format!("HTTP/1.1 {status} {reason}\r\nContent-Type: {media}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",body.len()).into_bytes();
    bytes.extend(body);
    bytes
}
fn mime(path: &str) -> &'static str {
    match path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "png" => "image/png",
        "bmp" => "image/bmp",
        "gif" => "image/gif",
        "jpg" | "jpeg" => "image/jpeg",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}
impl Runtime {
    pub(super) fn browser_failed(&mut self, device: &Id, app: &Id) {
        if let Ok(s) = self.state(device, app) {
            s.browser.loading = false;
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Simulation context and actor are explicit; effects remain transactional."
    )]
    pub(super) fn navigate(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        device: &Id,
        app: &Id,
        input: &str,
        fields: Option<BTreeMap<String, String>>,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let base = self.state(device, app)?.browser.url.clone();
        let url = Url::parse(input, (!base.is_empty()).then_some(base.as_str()))?;
        if fields
            .as_ref()
            .is_some_and(|f| f.len() > 64 || f.iter().any(|(k, v)| k.len() > 256 || v.len() > 4096))
        {
            return Err(diag("resource.limit_exceeded", "form"));
        }
        self.web_cancel(device, app, flow, out);
        self.serial += 1;
        let generation = self.serial;
        self.state(device, app)?.browser = BrowserView {
            url: url.full(),
            loading: true,
            generation,
            ..Default::default()
        };
        self.web_fetch(
            project, net, now, device, app, url, fields, generation, false, flow, out,
        )
    }
    fn web_cancel(&mut self, device: &Id, app: &Id, flow: &Id, out: &mut Output) {
        let keys: Vec<_> = self
            .web
            .clients
            .iter()
            .filter(|((d, _), c)| d == device && &c.app == app)
            .map(|(k, _)| k.clone())
            .collect();
        for (d, s) in keys {
            self.web.clients.remove(&(d.clone(), s.clone()));
            socket(
                out,
                &d,
                flow,
                SocketCommand::Close {
                    socket: s,
                    force: true,
                },
            );
        }
        // Cancel name lookups as well, so late replies cannot load old resources.
        self.cancel_browser_dns(device, app, flow, out);
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Simulation context and actor are explicit; effects remain transactional."
    )]
    pub(super) fn web_fetch(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        device: &Id,
        app: &Id,
        url: Url,
        fields: Option<BTreeMap<String, String>>,
        generation: u64,
        resource: bool,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        if let Ok(address) = url.host.parse() {
            self.web_connect(
                device, app, url, address, fields, generation, resource, flow, out,
            )
        } else {
            let name = url.host.clone();
            self.dns_query(
                project,
                net,
                now,
                device,
                app,
                &name,
                DnsType::A,
                None,
                dns::Purpose::Browser {
                    url,
                    fields,
                    generation,
                    resource,
                },
                flow,
                out,
            )
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Simulation context and actor are explicit; effects remain transactional."
    )]
    pub(super) fn web_connect(
        &mut self,
        device: &Id,
        app: &Id,
        url: Url,
        address: Ipv4Addr,
        fields: Option<BTreeMap<String, String>>,
        generation: u64,
        resource: bool,
        flow: &Id,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        if self.state(device, app)?.browser.generation != generation {
            return Ok(());
        }
        if self.web.clients.len() >= 256 {
            return Err(diag("resource.limit_exceeded", "http_connections"));
        }
        let body = fields
            .as_ref()
            .map(|f| {
                f.iter()
                    .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
                    .collect::<Vec<_>>()
                    .join("&")
            })
            .unwrap_or_default();
        let request=format!("{} {} HTTP/1.1\r\nHost: {}:{}\r\nContent-Length: {}\r\nContent-Type: application/x-www-form-urlencoded\r\nConnection: close\r\n\r\n{body}",if fields.is_some(){"POST"}else{"GET"},url.path,url.host,url.port,body.len()).into_bytes();
        self.serial += 1;
        let s = Id::new(format!("http-client-{}", self.serial))?;
        self.web.clients.insert(
            (device.clone(), s.clone()),
            Client {
                app: app.clone(),
                url: url.clone(),
                generation,
                resource,
                request,
                bytes: vec![],
            },
        );
        socket(
            out,
            device,
            flow,
            SocketCommand::Connect {
                socket: s.clone(),
                owner: app.clone(),
                destination: address,
                port: url.port,
                local_address: None,
                local_port: None,
            },
        );
        out.effects.push(Effect::Timer(Timer {
            device: device.clone(),
            application: app.clone(),
            token: generation,
            kind: TimerKind::WebTimeout { socket: s },
            delay_us: 15_000_000,
            correlation: flow.clone(),
        }));
        Ok(())
    }
    pub(super) fn web_notice(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        n: &SocketNotice,
        out: &mut Output,
    ) -> Result<bool, Diagnostic> {
        let key = (n.device.clone(), n.socket.clone());
        if matches!(n.kind, SocketNoticeKind::Closed { .. }) {
            self.web.servers.remove(&key);
        }
        if let Some(mut client) = self.web.clients.remove(&key) {
            match &n.kind {
                SocketNoticeKind::Connected => {
                    socket(
                        out,
                        &n.device,
                        &n.correlation,
                        SocketCommand::Send {
                            socket: n.socket.clone(),
                            data_base64: STANDARD.encode(&client.request),
                        },
                    );
                    fact(
                        out,
                        &n.device,
                        &n.correlation,
                        "http.request.sent",
                        json!({"url":client.url.full(),"resource":client.resource,"request_base64":STANDARD.encode(&client.request)}),
                    );
                }
                SocketNoticeKind::Data { data_base64 } => {
                    client.bytes.extend(
                        STANDARD
                            .decode(data_base64)
                            .map_err(|_| diag("validation.invalid_base64", "HTTP"))?,
                    );
                    match parse(&client.bytes) {
                        Ok(Some(response)) => {
                            if let Err(e) =
                                self.web_complete(project, net, now, n, &client, response, out)
                            {
                                push(
                                    self.state(&n.device, &client.app)?,
                                    Line::Error { diagnostic: e },
                                );
                                self.browser_failed(&n.device, &client.app);
                            }
                            socket(
                                out,
                                &n.device,
                                &n.correlation,
                                SocketCommand::Close {
                                    socket: n.socket.clone(),
                                    force: false,
                                },
                            );
                            return Ok(true);
                        }
                        Err(e) => {
                            push(
                                self.state(&n.device, &client.app)?,
                                Line::Error { diagnostic: e },
                            );
                            self.browser_failed(&n.device, &client.app);
                            socket(
                                out,
                                &n.device,
                                &n.correlation,
                                SocketCommand::Close {
                                    socket: n.socket.clone(),
                                    force: true,
                                },
                            );
                            return Ok(true);
                        }
                        _ => {}
                    }
                }
                SocketNoticeKind::Closed { reason } => {
                    self.browser_failed(&n.device, &client.app);
                    push(
                        self.state(&n.device, &client.app)?,
                        Line::Error {
                            diagnostic: diag("http.connection_failed", reason),
                        },
                    );
                    return Ok(true);
                }
                SocketNoticeKind::PeerClosed => {
                    self.browser_failed(&n.device, &client.app);
                    push(
                        self.state(&n.device, &client.app)?,
                        Line::Error {
                            diagnostic: diag("http.truncated_response", client.url.full()),
                        },
                    );
                    socket(
                        out,
                        &n.device,
                        &n.correlation,
                        SocketCommand::Close {
                            socket: n.socket.clone(),
                            force: false,
                        },
                    );
                    return Ok(true);
                }
                _ => {}
            }
            self.web.clients.insert(key, client);
            return Ok(true);
        }
        let metadata = project
            .device(&n.device)
            .and_then(|d| d.applications.iter().find(|a| a.id == n.owner));
        if metadata.is_some_and(|a| a.kind == "web_browser") {
            if matches!(n.kind, SocketNoticeKind::PeerClosed) {
                socket(
                    out,
                    &n.device,
                    &n.correlation,
                    SocketCommand::Close {
                        socket: n.socket.clone(),
                        force: false,
                    },
                );
            }
            return Ok(true);
        }
        let Some(ServiceConfig::Http(cfg)) = metadata.and_then(|a| a.service.as_ref()) else {
            return Ok(false);
        };
        if !self.state(&n.device, &n.owner)?.running {
            return Ok(true);
        }
        match &n.kind {
            SocketNoticeKind::Accepted => {
                if self.web.servers.len() >= 256 {
                    socket(
                        out,
                        &n.device,
                        &n.correlation,
                        SocketCommand::Close {
                            socket: n.socket.clone(),
                            force: true,
                        },
                    );
                } else {
                    self.web.servers.insert(key, vec![]);
                    out.effects.push(Effect::Timer(Timer {
                        device: n.device.clone(),
                        application: n.owner.clone(),
                        token: 0,
                        kind: TimerKind::WebServerTimeout {
                            socket: n.socket.clone(),
                        },
                        delay_us: 15_000_000,
                        correlation: n.correlation.clone(),
                    }));
                }
            }
            SocketNoticeKind::Closed { .. } => {
                self.web.servers.remove(&key);
            }
            SocketNoticeKind::Data { data_base64 } => {
                let buffer = self.web.servers.entry(key.clone()).or_default();
                buffer.extend(
                    STANDARD
                        .decode(data_base64)
                        .map_err(|_| diag("validation.invalid_base64", "HTTP"))?,
                );
                let result = parse(buffer);
                let response = match result {
                    Ok(None) => return Ok(true),
                    Ok(Some(request)) => {
                        if cfg.administration
                            && request
                                .first
                                .split_whitespace()
                                .nth(1)
                                .is_some_and(|p| p.starts_with("/admin") || p == "/")
                        {
                            self.admin_response(project, net, now, n, request, out)?
                        } else {
                            self.serve(&n.device, cfg, request)
                        }
                    }
                    Err(_) => response(400, "text/plain", b"Bad request".to_vec()),
                };
                self.web.servers.remove(&key);
                fact(
                    out,
                    &n.device,
                    &n.correlation,
                    "http.response.sent",
                    json!({"socket":n.socket,"bytes":response.len(),"response_base64":STANDARD.encode(&response)}),
                );
                socket(
                    out,
                    &n.device,
                    &n.correlation,
                    SocketCommand::Send {
                        socket: n.socket.clone(),
                        data_base64: STANDARD.encode(response),
                    },
                );
                socket(
                    out,
                    &n.device,
                    &n.correlation,
                    SocketCommand::Close {
                        socket: n.socket.clone(),
                        force: false,
                    },
                );
            }
            SocketNoticeKind::PeerClosed => {
                self.web.servers.remove(&key);
                socket(
                    out,
                    &n.device,
                    &n.correlation,
                    SocketCommand::Close {
                        socket: n.socket.clone(),
                        force: false,
                    },
                );
            }
            _ => {}
        }
        Ok(true)
    }
    fn serve(&self, device: &Id, cfg: &HttpConfig, request: Message) -> Vec<u8> {
        let words: Vec<_> = request.first.split(' ').collect();
        if words.len() != 3 || words[2] != "HTTP/1.1" || !request.headers.contains_key("host") {
            return response(400, "text/plain", b"Bad request".to_vec());
        }
        let method = words[0];
        if method != "GET" && method != "POST" {
            return response(501, "text/plain", b"Not implemented".to_vec());
        }
        let Ok(path) = decode(words[1].split('?').next().unwrap_or("/"), false) else {
            return response(400, "text/plain", b"Invalid path".to_vec());
        };
        if !path.starts_with('/') || path.split('/').any(|s| s == ".." || s == ".") {
            return response(403, "text/plain", b"Forbidden".to_vec());
        }
        if method == "POST" {
            if cfg.form_path.as_deref() != Some(&path) {
                return response(404, "text/plain", b"Not found".to_vec());
            }
            if request
                .headers
                .get("content-type")
                .is_none_or(|v| !v.starts_with("application/x-www-form-urlencoded"))
            {
                return response(400, "text/plain", b"Unsupported form".to_vec());
            }
            let Ok(body) = std::str::from_utf8(&request.body) else {
                return response(400, "text/plain", b"Invalid form".to_vec());
            };
            let mut html = String::from("<html><body><h1>POST</h1>");
            if body.split('&').count() > 64 {
                return response(400, "text/plain", b"Too many fields".to_vec());
            }
            for field in body.split('&').filter(|f| !f.is_empty()) {
                let (k, v) = field.split_once('=').unwrap_or((field, ""));
                let (Ok(k), Ok(v)) = (decode(k, true), decode(v, true)) else {
                    return response(400, "text/plain", b"Invalid form".to_vec());
                };
                html += &format!("<p>{}: {}</p>", escape(&k), escape(&v));
            }
            html += "</body></html>";
            return response(200, "text/html; charset=utf-8", html.into_bytes());
        }
        let host = request.headers["host"].split(':').next().unwrap_or("");
        let host = dns_name(host).unwrap_or_default();
        let root = cfg
            .virtual_hosts
            .iter()
            .find(|(name, _)| dns_name(name).ok().as_ref() == Some(&host))
            .map(|(_, r)| r)
            .unwrap_or(&cfg.root);
        let path = if path == "/" {
            format!("{}/index.html", root.trim_end_matches('/'))
        } else {
            format!("{}{path}", root.trim_end_matches('/'))
        };
        match self.files(device).and_then(|fs| fs.read(&path).ok()) {
            Some(bytes) => response(200, mime(&path), bytes.to_vec()),
            None => response(404, "text/plain", b"Not found".to_vec()),
        }
    }
    #[expect(
        clippy::too_many_arguments,
        reason = "Simulation context and actor are explicit; effects remain transactional."
    )]
    fn web_complete(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        n: &SocketNotice,
        client: &Client,
        response: Message,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        if self.state(&n.device, &client.app)?.browser.generation != client.generation {
            return Ok(());
        }
        if !response.first.starts_with("HTTP/1.1 ") {
            return Err(diag("http.invalid_message", "status"));
        }
        let status = response
            .first
            .split(' ')
            .nth(1)
            .and_then(|s| s.parse::<u16>().ok())
            .filter(|s| (100..=599).contains(s))
            .ok_or_else(|| diag("http.invalid_message", "status"))?;
        let media = response
            .headers
            .get("content-type")
            .cloned()
            .unwrap_or_default();
        fact(
            out,
            &n.device,
            &n.correlation,
            "http.response.received",
            json!({"url":client.url.full(),"status":status,"media_type":media,"resource":client.resource,"bytes":response.body.len()}),
        );
        if client.resource {
            if status == 200
                && matches!(
                    media.as_str(),
                    "image/png" | "image/bmp" | "image/gif" | "image/jpeg"
                )
            {
                self.state(&n.device, &client.app)?.browser.images.insert(
                    client.url.full(),
                    ImageResource {
                        media_type: media,
                        data_base64: STANDARD.encode(response.body),
                    },
                );
            }
            return Ok(());
        }
        let html =
            String::from_utf8(response.body).map_err(|_| diag("http.invalid_encoding", "page"))?;
        let parsed = if media.starts_with("text/html") {
            Some(
                sns_html::parse_network(&html)
                    .map_err(|e| diag("html.invalid_document", e.to_string()))?,
            )
        } else {
            None
        };
        {
            let b = &mut self.state(&n.device, &client.app)?.browser;
            b.status = status;
            b.html = if parsed.is_some() {
                html
            } else {
                format!("<p>{}</p>", escape(&html))
            };
            b.loading = false;
        }
        if let Some(doc) = parsed {
            let mut urls = BTreeSet::new();
            for block in doc.blocks {
                if let sns_html::Block::Image { source, .. } = block {
                    let Ok(url) = Url::parse(&source, Some(&client.url.full())) else {
                        continue;
                    };
                    if urls.len() < 32
                        && urls.insert(url.full())
                        && let Err(e) = self.web_fetch(
                            project,
                            net,
                            now,
                            &n.device,
                            &client.app,
                            url,
                            None,
                            client.generation,
                            true,
                            &n.correlation,
                            out,
                        )
                    {
                        push(
                            self.state(&n.device, &client.app)?,
                            Line::Error { diagnostic: e },
                        );
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn cancel_web(&mut self, device: &Id, app: &Id) {
        self.web
            .clients
            .retain(|(d, _), c| d != device || &c.app != app);
        // Accepted server connections belong to the server owner; only remove that owner's sockets below.
    }
    pub(super) fn web_server_timeout(&mut self, t: &Timer, id: &Id) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        if self
            .web
            .servers
            .remove(&(t.device.clone(), id.clone()))
            .is_some()
        {
            socket(
                &mut out,
                &t.device,
                &t.correlation,
                SocketCommand::Close {
                    socket: id.clone(),
                    force: true,
                },
            );
        }
        Ok(out)
    }
    pub(super) fn web_timeout(&mut self, t: &Timer, id: &Id) -> Result<Output, Diagnostic> {
        let mut out = Output::default();
        if let Some(c) = self.web.clients.remove(&(t.device.clone(), id.clone())) {
            self.browser_failed(&t.device, &c.app);
            push(
                self.state(&t.device, &c.app)?,
                Line::Error {
                    diagnostic: diag("http.timeout", c.url.full()),
                },
            );
            socket(
                &mut out,
                &t.device,
                &t.correlation,
                SocketCommand::Close {
                    socket: id.clone(),
                    force: true,
                },
            );
            fact(
                &mut out,
                &t.device,
                &t.correlation,
                "http.timeout",
                json!({"url":c.url.full()}),
            );
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn strict_framing_and_offline_urls() {
        assert!(
            parse(b"POST / HTTP/1.1\r\nHost: a\r\nContent-Length: 0\r\nContent-Length: 2\r\n\r\n")
                .is_err()
        );
        assert!(parse(b"GET / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n").is_err());
        for u in [
            "https://example.com/",
            "file:///etc/passwd",
            "javascript:x",
            "http://a@b/",
        ] {
            assert!(Url::parse(u, None).is_err(), "{u}");
        }
        assert_eq!(
            Url::parse("../b", Some("http://a/x/y")).unwrap().full(),
            "http://a:80/b"
        );
    }
}
