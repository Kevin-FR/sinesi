//! Internal administration pages, reached only through the simulated HTTP server.
use super::*;
use sns_i18n::Language;
fn fields(body: &[u8]) -> Result<BTreeMap<String, String>, Diagnostic> {
    let text = std::str::from_utf8(body).map_err(|_| diag("http.invalid_encoding", "form"))?;
    let mut values = BTreeMap::new();
    if text.split('&').count() > 64 {
        return Err(diag("resource.limit_exceeded", "form"));
    }
    for f in text.split('&').filter(|f| !f.is_empty()) {
        let (k, v) = f.split_once('=').unwrap_or((f, ""));
        if values.insert(decode(k, true)?, decode(v, true)?).is_some() {
            return Err(diag("http.invalid_message", "duplicate_field"));
        }
    }
    Ok(values)
}
fn match_text(m: &IpMatch) -> String {
    match m {
        IpMatch::Any => "*".into(),
        IpMatch::LocalNetwork => "LAN".into(),
        IpMatch::Subnet { network, prefix } => format!("{network}/{prefix}"),
    }
}
fn parse_match(s: &str) -> Result<IpMatch, Diagnostic> {
    if s.is_empty() || s == "*" {
        return Ok(IpMatch::Any);
    }
    if s.eq_ignore_ascii_case("LAN") {
        return Ok(IpMatch::LocalNetwork);
    }
    let (a, p) = s.split_once('/').unwrap_or((s, "32"));
    let result = IpMatch::Subnet {
        network: a.parse().map_err(|_| diag("firewall.invalid_rule", s))?,
        prefix: p.parse().map_err(|_| diag("firewall.invalid_rule", s))?,
    };
    result.validate()?;
    Ok(result)
}
fn protocol(s: &str) -> Result<FilterProtocol, Diagnostic> {
    match s.to_ascii_uppercase().as_str() {
        "*" | "ANY" => Ok(FilterProtocol::Any),
        "TCP" => Ok(FilterProtocol::Tcp),
        "UDP" => Ok(FilterProtocol::Udp),
        _ => Err(diag("firewall.invalid_rule", s)),
    }
}
fn verdict(s: &str) -> Result<Verdict, Diagnostic> {
    match s.to_ascii_uppercase().as_str() {
        "ACCEPT" => Ok(Verdict::Accept),
        "DROP" => Ok(Verdict::Drop),
        _ => Err(diag("firewall.invalid_rule", s)),
    }
}
fn number(s: &str) -> Result<u16, Diagnostic> {
    s.parse::<u16>()
        .ok()
        .filter(|v| *v > 0)
        .ok_or_else(|| diag("firewall.invalid_rule", s))
}
fn parse_rule(f: &BTreeMap<String, String>) -> Result<FirewallRule, Diagnostic> {
    let get = |key: &str| f.get(key).map(String::as_str).unwrap_or("");
    let p = get("port");
    Ok(FirewallRule {
        source: parse_match(get("source"))?,
        destination: parse_match(get("destination"))?,
        protocol: protocol(get("protocol"))?,
        destination_port: if p.is_empty() || p == "*" {
            None
        } else {
            Some(number(p)?)
        },
        action: verdict(get("action"))?,
    })
}
/// A stale-form guard, not an authentication token.
fn version(host: &Device) -> String {
    serde_json::to_vec(&host.network)
        .unwrap()
        .iter()
        .fold(14695981039346656037u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(1099511628211)
        })
        .to_string()
}
fn form(
    html: &mut String,
    path: &str,
    l: Language,
    version: &str,
    controls: &[(&str, &str, String)],
    button: &str,
) {
    *html += &format!(
        "<form method=\"post\" action=\"{path}?lang={}&amp;version={version}\">",
        l.code()
    );
    for (name, label, value) in controls {
        *html += &format!(
            "<label>{}<input name=\"{name}\" value=\"{}\"></label>",
            escape(l.text(label)),
            escape(value)
        );
    }
    *html += &format!("<button>{}</button></form>", escape(button));
}
fn page_index(params: &BTreeMap<String, String>, len: usize) -> usize {
    params
        .get("page")
        .and_then(|p| p.parse::<usize>().ok())
        .unwrap_or(0)
        .min(len.saturating_sub(1) / 64)
}
fn pages(html: &mut String, path: &str, l: Language, len: usize) {
    for page in 0..len.div_ceil(64) {
        *html += &format!(
            "<a href=\"{path}?lang={}&amp;page={page}\">{} {}</a><br>",
            l.code(),
            escape(l.text("page")),
            page + 1
        );
    }
}
impl Runtime {
    pub(super) fn admin_response(
        &mut self,
        project: &Project,
        net: &Network,
        now: u64,
        n: &SocketNotice,
        request: Message,
        out: &mut Output,
    ) -> Result<Vec<u8>, Diagnostic> {
        let words: Vec<_> = request.first.split_whitespace().collect();
        if words.len() != 3 || words[2] != "HTTP/1.1" || !request.headers.contains_key("host") {
            return Ok(response(400, "text/plain", b"Bad request".to_vec()));
        }
        let host = project.device(&n.device).unwrap();
        let effective = net.effective_project(project);
        let effective_host = effective.device(&n.device).unwrap();
        let peer = net
            .transport
            .views()
            .into_iter()
            .find(|s| s.device == n.device && s.socket == n.socket)
            .and_then(|s| s.peer)
            .map(|p| p.address);
        let allowed=peer.is_some_and(|a|effective_host.interfaces.iter().filter(|i|host.network.gateway.as_ref().is_none_or(|g|g.lan==i.id)).any(|i|matches!(&i.ipv4,Some(Ipv4Config::Static{address,prefix_length}) if address.parse::<Ipv4Addr>().is_ok_and(|ip|u32::from(ip)&prefix_mask(*prefix_length)==u32::from(a)&prefix_mask(*prefix_length)))));
        if !allowed {
            return Ok(response(
                403,
                "text/plain",
                b"LAN administration only".to_vec(),
            ));
        }
        let (path, query) = words[1].split_once('?').unwrap_or((words[1], ""));
        let path = if path == "/" { "/admin" } else { path };
        let params = fields(query.as_bytes()).unwrap_or_default();
        let l = Language::ALL
            .into_iter()
            .find(|l| Some(l.code()) == params.get("lang").map(String::as_str))
            .unwrap_or_default();
        let version = version(host);
        if words[0] == "POST" {
            if params.get("version") != Some(&version) {
                return Ok(response(
                    409,
                    "text/html",
                    format!("<p>{}</p>", escape(l.text("admin_stale"))).into_bytes(),
                ));
            }
            if request
                .headers
                .get("content-type")
                .is_none_or(|v| !v.starts_with("application/x-www-form-urlencoded"))
            {
                return Ok(response(400, "text/plain", b"Invalid form".to_vec()));
            }
            let result = (|| -> Result<SecurityCommand, Diagnostic> {
                let values = fields(&request.body)?;
                if path == "/admin/nat/purge" {
                    return Ok(SecurityCommand::PurgeNat);
                }
                if path.starts_with("/admin/nat/") {
                    let mut cfg = *host
                        .network
                        .gateway
                        .clone()
                        .ok_or_else(|| diag("nat.invalid_config", host.id.as_str()))?;
                    if path == "/admin/nat/toggle" {
                        cfg.nat_enabled = !cfg.nat_enabled;
                    } else if path == "/admin/nat/add" {
                        if cfg.forwards.len() >= 128 {
                            return Err(diag("resource.limit_exceeded", "forwards"));
                        }
                        cfg.forwards.push(PortForward {
                            protocol: match values.get("protocol").map(String::as_str) {
                                Some("TCP") => TransportProtocol::Tcp,
                                Some("UDP") => TransportProtocol::Udp,
                                _ => return Err(diag("nat.invalid_config", "protocol")),
                            },
                            external_port: number(
                                values.get("external").map(String::as_str).unwrap_or(""),
                            )?,
                            internal_address: values
                                .get("address")
                                .ok_or_else(|| diag("nat.invalid_config", "address"))?
                                .parse()
                                .map_err(|_| diag("nat.invalid_config", "address"))?,
                            internal_port: number(
                                values.get("internal").map(String::as_str).unwrap_or(""),
                            )?,
                        });
                    } else if let Some(index) = path
                        .strip_prefix("/admin/nat/delete/")
                        .and_then(|s| s.parse::<usize>().ok())
                    {
                        if index >= cfg.forwards.len() {
                            return Err(diag("nat.invalid_config", "index"));
                        }
                        cfg.forwards.remove(index);
                    } else {
                        return Err(diag("http.invalid_url", path));
                    }
                    cfg.validate(host)?;
                    return Ok(SecurityCommand::ConfigureGateway { config: cfg });
                }
                let mut cfg = host
                    .network
                    .firewall
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(|| FirewallConfig {
                        enabled: false,
                        ..Default::default()
                    });
                match path {
                    "/admin/firewall/toggle" => cfg.enabled = !cfg.enabled,
                    "/admin/firewall/syn" => cfg.syn_only = !cfg.syn_only,
                    "/admin/firewall/udp" => cfg.filter_udp = !cfg.filter_udp,
                    "/admin/firewall/icmp" => cfg.drop_icmp = !cfg.drop_icmp,
                    "/admin/firewall/accept" => cfg.default_policy = Verdict::Accept,
                    "/admin/firewall/drop" => cfg.default_policy = Verdict::Drop,
                    "/admin/firewall/add" => cfg.rules.push(parse_rule(&values)?),
                    _ => {
                        let rest = path
                            .strip_prefix("/admin/firewall/rule/")
                            .ok_or_else(|| diag("http.invalid_url", path))?;
                        let (index, operation) = rest.split_once('/').unwrap_or((rest, "edit"));
                        let index = index
                            .parse::<usize>()
                            .ok()
                            .filter(|i| *i < cfg.rules.len())
                            .ok_or_else(|| diag("firewall.invalid_rule", index))?;
                        match operation {
                            "edit" => cfg.rules[index] = parse_rule(&values)?,
                            "delete" => {
                                cfg.rules.remove(index);
                            }
                            "up" if index > 0 => cfg.rules.swap(index, index - 1),
                            "down" if index + 1 < cfg.rules.len() => {
                                cfg.rules.swap(index, index + 1)
                            }
                            _ => return Err(diag("firewall.invalid_rule", operation)),
                        }
                    }
                }
                cfg.validate()?;
                Ok(SecurityCommand::ConfigureFirewall { config: cfg })
            })();
            match result {
                Ok(command) => {
                    out.effects.push(Effect::Security {
                        device: n.device.clone(),
                        command,
                        correlation: n.correlation.clone(),
                    });
                    fact(
                        out,
                        &n.device,
                        &n.correlation,
                        "admin.configuration.requested",
                        json!({"path":path}),
                    );
                    return Ok(response(
                        200,
                        "text/html",
                        format!(
                            "<h1>{}</h1><a href=\"/admin?lang={}\">{}</a>",
                            escape(l.text("admin_saved")),
                            l.code(),
                            escape(l.text("admin_home"))
                        )
                        .into_bytes(),
                    ));
                }
                Err(e) => {
                    return Ok(response(
                        400,
                        "text/html",
                        format!("<p>{}</p>", escape(&l.describe_error(&e.to_string())))
                            .into_bytes(),
                    ));
                }
            }
        }
        if words[0] != "GET" {
            return Ok(response(501, "text/plain", b"Not implemented".to_vec()));
        }
        let mut html = format!("<h1>{}</h1>", escape(l.text("admin_title")));
        for (href, key) in [
            ("/admin", "admin_home"),
            ("/admin/routes", "admin_routes"),
            ("/admin/nat", "nat_table"),
            ("/admin/firewall", "firewall"),
            ("/admin/log", "firewall_log"),
        ] {
            html += &format!(
                "<a href=\"{href}?lang={}\">{}</a><br>",
                l.code(),
                escape(l.text(key))
            );
        }
        for lang in Language::ALL {
            html += &format!(
                "<a href=\"{path}?lang={}\">{}</a> ",
                lang.code(),
                lang.label()
            );
        }
        match path {
            "/admin" | "/admin/" => {
                html += &format!(
                    "<p>{}</p><p>{}</p>",
                    escape(&host.label),
                    escape(l.text("admin_lan_hint"))
                );
            }
            "/admin/routes" => {
                html += &format!("<h2>{}</h2>", escape(l.text("admin_routes")));
                let rows = net.routing.views(effective_host, now);
                let page = page_index(&params, rows.len());
                pages(&mut html, path, l, rows.len());
                for r in rows.into_iter().skip(page * 64).take(64) {
                    html += &format!(
                        "<p>{} / {} → {} · {} · {} · {}</p>",
                        r.network,
                        r.prefix,
                        r.gateway
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| "—".into()),
                        escape(r.interface.as_ref().map(Id::as_str).unwrap_or("—")),
                        l.text(match r.origin {
                            sns_protocols::routing::Origin::Loopback => "route_loopback",
                            sns_protocols::routing::Origin::Direct => "route_direct",
                            sns_protocols::routing::Origin::Static => "route_static",
                            sns_protocols::routing::Origin::Rip => "route_rip",
                            sns_protocols::routing::Origin::Default => "route_default",
                        }),
                        r.metric
                    );
                }
            }
            "/admin/nat" => {
                html += &format!("<h2>{}</h2>", escape(l.text("nat_table")));
                if let Some(g) = &host.network.gateway {
                    html += &format!(
                        "<p>LAN: {} · WAN: {}</p>",
                        escape(g.lan.as_str()),
                        escape(g.wan.as_str())
                    );
                    form(
                        &mut html,
                        "/admin/nat/toggle",
                        l,
                        &version,
                        &[],
                        l.text(if g.nat_enabled {
                            "nat_disable"
                        } else {
                            "nat_enable"
                        }),
                    );
                    form(
                        &mut html,
                        "/admin/nat/purge",
                        l,
                        &version,
                        &[],
                        l.text("nat_purge"),
                    );
                    let mappings: Vec<_> = net
                        .nat
                        .entries
                        .values()
                        .filter(|e| e.device == host.id)
                        .collect();
                    let page = page_index(&params, mappings.len());
                    pages(&mut html, path, l, mappings.len());
                    for e in mappings.into_iter().skip(page * 64).take(64) {
                        html += &format!(
                            "<p>{}: {} → {} · {} · {} µs</p>",
                            format!("{:?}", e.protocol).to_uppercase(),
                            e.internal,
                            e.external,
                            e.remote,
                            e.expires_us.0.saturating_sub(now)
                        );
                    }
                    for (i, f) in g.forwards.iter().enumerate() {
                        html += &format!(
                            "<p>{:?} {} → {}:{}</p>",
                            f.protocol, f.external_port, f.internal_address, f.internal_port
                        );
                        form(
                            &mut html,
                            &format!("/admin/nat/delete/{i}"),
                            l,
                            &version,
                            &[],
                            l.text("delete"),
                        );
                    }
                    form(
                        &mut html,
                        "/admin/nat/add",
                        l,
                        &version,
                        &[
                            ("protocol", "protocol", "TCP".into()),
                            ("external", "nat_external_port", "8080".into()),
                            ("address", "nat_internal_address", "192.168.10.2".into()),
                            ("internal", "nat_internal_port", "80".into()),
                        ],
                        l.text("nat_add_forward"),
                    );
                } else {
                    html += &format!("<p>{}</p>", escape(l.text("nat_gateway_only")));
                }
            }
            "/admin/firewall" => {
                let cfg = host
                    .network
                    .firewall
                    .as_deref()
                    .cloned()
                    .unwrap_or_else(|| FirewallConfig {
                        enabled: false,
                        ..Default::default()
                    });
                for (p, k) in [
                    (
                        "toggle",
                        if cfg.enabled {
                            "firewall_disable"
                        } else {
                            "firewall_enable"
                        },
                    ),
                    (
                        "syn",
                        if cfg.syn_only {
                            "firewall_all_segments"
                        } else {
                            "firewall_syn_only"
                        },
                    ),
                    (
                        "udp",
                        if cfg.filter_udp {
                            "firewall_udp_disable"
                        } else {
                            "firewall_udp_enable"
                        },
                    ),
                    (
                        "icmp",
                        if cfg.drop_icmp {
                            "firewall_icmp_allow"
                        } else {
                            "firewall_icmp_drop"
                        },
                    ),
                    ("accept", "firewall_default_accept"),
                    ("drop", "firewall_default_drop"),
                ] {
                    form(
                        &mut html,
                        &format!("/admin/firewall/{p}"),
                        l,
                        &version,
                        &[],
                        l.text(k),
                    );
                }
                html += &format!(
                    "<p>{}: {:?}</p>",
                    escape(l.text("firewall_default")),
                    cfg.default_policy
                );
                let page = params
                    .get("page")
                    .and_then(|p| p.parse::<usize>().ok())
                    .unwrap_or(0)
                    .min(7);
                for page_index in 0..cfg.rules.len().div_ceil(16) {
                    html += &format!(
                        "<a href=\"/admin/firewall?lang={}&amp;page={page_index}\">{} {}</a><br>",
                        l.code(),
                        escape(l.text("page")),
                        page_index + 1
                    );
                }
                for (i, r) in cfg.rules.iter().enumerate().skip(page * 16).take(16) {
                    html += &format!("<h2>{} {}</h2>", escape(l.text("firewall_rule")), i + 1);
                    let controls = [
                        ("source", "firewall_source", match_text(&r.source)),
                        (
                            "destination",
                            "firewall_destination",
                            match_text(&r.destination),
                        ),
                        (
                            "protocol",
                            "protocol",
                            match r.protocol {
                                FilterProtocol::Any => "*",
                                FilterProtocol::Tcp => "TCP",
                                FilterProtocol::Udp => "UDP",
                            }
                            .into(),
                        ),
                        (
                            "port",
                            "port",
                            r.destination_port
                                .map(|p| p.to_string())
                                .unwrap_or("*".into()),
                        ),
                        (
                            "action",
                            "firewall_action",
                            format!("{:?}", r.action).to_uppercase(),
                        ),
                    ];
                    form(
                        &mut html,
                        &format!("/admin/firewall/rule/{i}"),
                        l,
                        &version,
                        &controls,
                        l.text("apply"),
                    );
                    for (op, key) in [
                        ("delete", "delete"),
                        ("up", "move_up"),
                        ("down", "move_down"),
                    ] {
                        if (op == "up" && i == 0) || (op == "down" && i + 1 == cfg.rules.len()) {
                            continue;
                        }
                        form(
                            &mut html,
                            &format!("/admin/firewall/rule/{i}/{op}"),
                            l,
                            &version,
                            &[],
                            l.text(key),
                        );
                    }
                }
                form(
                    &mut html,
                    "/admin/firewall/add",
                    l,
                    &version,
                    &[
                        ("source", "firewall_source", "LAN".into()),
                        ("destination", "firewall_destination", "*".into()),
                        ("protocol", "protocol", "TCP".into()),
                        ("port", "port", "80".into()),
                        ("action", "firewall_action", "ACCEPT".into()),
                    ],
                    l.text("firewall_add_rule"),
                );
            }
            "/admin/log" => {
                let rows = net
                    .firewall
                    .journal
                    .get(&host.id)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let page = page_index(&params, rows.len());
                pages(&mut html, path, l, rows.len());
                for row in rows.iter().rev().skip(page * 64).take(64) {
                    html += &format!(
                        "<p>{} µs · {} → {} · {} · {:?} · {} {:?} · {} {}</p>",
                        row.time_us.0,
                        row.source,
                        row.destination,
                        row.protocol,
                        row.action,
                        escape(l.text("firewall_rule")),
                        row.rule,
                        escape(l.text("frame")),
                        row.frame_id.0
                    );
                }
            }
            _ => return Ok(response(404, "text/plain", b"Not found".to_vec())),
        }
        Ok(response(200, "text/html; charset=utf-8", html.into_bytes()))
    }
}
