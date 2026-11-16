use super::*;
/// The pedagogical help inventory. Developer diagnostics are not commands.
pub const COMMAND_HELP: &str = "help · cd · pwd · ls / dir · touch · mkdir · cat / type · cp / copy · mv / move · rm / del · ipconfig · netstat · route · arp · arpsend · ping · traceroute · host · nslookup · dnscache · tcpdump · exit · cls / reset";
pub(crate) struct Request<'a> {
    pub project: &'a mut Project,
    pub net: &'a Network,
    pub now: u64,
    pub device: &'a Id,
    pub application: &'a Id,
    pub line: &'a str,
    pub flow: &'a Id,
}
/// Quoting is lexical only. No expansion, shell, redirection or host process exists.
pub fn parse_words(line: &str) -> Result<Vec<String>, Diagnostic> {
    if line.len() > 4096 {
        return Err(diag("resource.limit_exceeded", "terminal_input"));
    }
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut started = false;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if let Some(q) = quote {
            if c == q {
                quote = None;
            } else if c == '\\' && chars.peek().is_some_and(|next| *next == q || *next == '\\') {
                word.push(chars.next().unwrap());
            } else {
                word.push(c);
            }
            started = true;
        } else {
            match c {
                '\'' | '"' => {
                    quote = Some(c);
                    started = true;
                }
                c if c.is_whitespace() => {
                    if started {
                        words.push(std::mem::take(&mut word));
                        started = false;
                    }
                }
                _ => {
                    word.push(c);
                    started = true;
                }
            }
        }
    }
    if quote.is_some() {
        return Err(diag("terminal.unclosed_quote", "terminal_input"));
    }
    if started {
        words.push(word);
    }
    Ok(words)
}
pub fn usage(command: &str) -> Option<&'static str> {
    Some(match command {
        "help" => "help [COMMAND]",
        "cd" => "cd PATH",
        "pwd" => "pwd",
        "ls" | "dir" => "ls [PATH] / dir [PATH]",
        "touch" => "touch PATH",
        "mkdir" => "mkdir PATH",
        "cat" | "type" => "cat PATH / type PATH",
        "cp" | "copy" => "cp SOURCE DESTINATION / copy SOURCE DESTINATION",
        "mv" | "move" => "mv SOURCE DESTINATION / move SOURCE DESTINATION",
        "rm" | "del" => "rm [-r] PATH / del [-r] PATH",
        "ipconfig" => "ipconfig",
        "host" => "host [-t A|MX|NS] NAME [DNS_IP]",
        "nslookup" => "nslookup [-type=A|MX|NS] NAME [DNS_IP]",
        "dnscache" => "dnscache -d",
        "netstat" => "netstat",
        "route" => "route",
        "traceroute" => "traceroute [-m MAX_HOPS] IP_OR_NAME",
        "arpsend" => "arpsend SOURCE_IP TARGET_IP",
        "arp" => "arp [-a [IP] / -d [IP]]",
        "ping" => "ping [-c COUNT / -n COUNT] [-b] IP_OR_NAME",
        "tcpdump" => "tcpdump",
        "exit" => "exit",
        "cls" | "reset" => "cls / reset",
        _ => return None,
    })
}
fn arity(args: &[String], min: usize, max: usize, cmd: &str) -> Result<(), Diagnostic> {
    if args.len() < min || args.len() > max {
        Err(diag("terminal.syntax", usage(cmd).unwrap_or(cmd)))
    } else {
        Ok(())
    }
}
impl Runtime {
    pub(crate) fn evaluate(
        &mut self,
        request: Request<'_>,
        out: &mut Output,
    ) -> Result<(), Diagnostic> {
        let Request {
            project,
            net,
            now,
            device,
            application,
            line,
            flow,
        } = request;
        let words = parse_words(line)?;
        let Some(command) = words.first().map(String::as_str) else {
            return Ok(());
        };
        let args = &words[1..];
        if usage(command).is_none() {
            return Err(diag("terminal.unknown_command", command));
        }
        if self.state(device, application)?.job.is_some()
            && !matches!(command, "cls" | "reset" | "exit")
        {
            return Err(diag("terminal.busy", application.as_str()));
        }
        let cwd = self.state(device, application)?.cwd.clone();
        let mut output = None;
        match command {
            "help" => {
                arity(args, 0, 1, command)?;
                if let Some(cmd) = args.first()
                    && usage(cmd).is_none()
                {
                    return Err(diag("terminal.unknown_command", cmd));
                }
                output = Some(Line::Help {
                    command: args.first().cloned(),
                });
            }
            "pwd" => {
                arity(args, 0, 0, command)?;
                output = Some(Line::Text { value: cwd });
            }
            "cd" => {
                arity(args, 1, 1, command)?;
                let path = sns_vfs::resolve(&cwd, &args[0])?;
                if !self.files(device).unwrap().is_directory(&path) {
                    return Err(diag("fs.not_directory", path));
                }
                self.state(device, application)?.cwd = path;
            }
            "ls" | "dir" => {
                arity(args, 0, 1, command)?;
                let path = sns_vfs::resolve(&cwd, args.first().map(String::as_str).unwrap_or("."))?;
                output = Some(Line::Json {
                    value: serde_json::to_value(self.files(device).unwrap().list(&path)?).unwrap(),
                });
            }
            "cat" | "type" => {
                arity(args, 1, 1, command)?;
                let path = sns_vfs::resolve(&cwd, &args[0])?;
                output = Some(Line::Text {
                    value: self.files(device).unwrap().read_text(&path)?.to_owned(),
                });
            }
            "touch" | "mkdir" => {
                arity(args, 1, 1, command)?;
                let path = sns_vfs::resolve(&cwd, &args[0])?;
                let op = if command == "mkdir" {
                    FileCommand::Mkdir { path }
                } else {
                    FileCommand::Write {
                        path,
                        media_type: "text/plain".into(),
                        data_base64: String::new(),
                        overwrite: false,
                    }
                };
                self.file(project, device, &op)?;
            }
            "cp" | "copy" | "mv" | "move" => {
                arity(args, 2, 2, command)?;
                let source = sns_vfs::resolve(&cwd, &args[0])?;
                let destination = sns_vfs::resolve(&cwd, &args[1])?;
                let op = if matches!(command, "cp" | "copy") {
                    FileCommand::Copy {
                        source,
                        destination,
                    }
                } else {
                    FileCommand::Move {
                        source,
                        destination,
                    }
                };
                self.file(project, device, &op)?;
            }
            "rm" | "del" => {
                arity(args, 1, 2, command)?;
                let recursive = args.first().is_some_and(|s| s == "-r");
                if (recursive && args.len() != 2) || (!recursive && args.len() != 1) {
                    return Err(diag("terminal.syntax", usage(command).unwrap()));
                }
                let path = sns_vfs::resolve(&cwd, &args[usize::from(recursive)])?;
                self.file(project, device, &FileCommand::Remove { path, recursive })?;
            }
            "route" => {
                arity(args, 0, 0, command)?;
                output = Some(Line::Json {
                    value: serde_json::to_value(
                        net.routing.views(project.device(device).unwrap(), now),
                    )
                    .unwrap(),
                });
            }
            "arpsend" => {
                arity(args, 2, 2, command)?;
                let source = args[0]
                    .parse()
                    .map_err(|_| diag("validation.invalid_ipv4", &args[0]))?;
                let target = args[1]
                    .parse()
                    .map_err(|_| diag("validation.invalid_ipv4", &args[1]))?;
                // Validate before scheduling: a terminal error must not fail the enclosing tick.
                let host = project.device(device).unwrap();
                sns_protocols::arp_injection_route(host, source, target)?;
                network(
                    out,
                    flow,
                    Action::ArpSend {
                        device: device.clone(),
                        source,
                        target,
                    },
                );
            }
            "traceroute" => {
                let (target, max) = match args {
                    [target] => (target, 30),
                    [flag, hops, target] if flag == "-m" => (
                        target,
                        hops.parse::<u8>()
                            .ok()
                            .filter(|n| (1..=64).contains(n))
                            .ok_or_else(|| diag("terminal.syntax", usage(command).unwrap()))?,
                    ),
                    _ => return Err(diag("terminal.syntax", usage(command).unwrap())),
                };
                if needs_dns(project, net, device, target) {
                    return self.dns_query(
                        project,
                        net,
                        now,
                        device,
                        application,
                        target,
                        DnsType::A,
                        None,
                        dns::Purpose::Command {
                            line: format!("traceroute -m {max} $DNS"),
                        },
                        flow,
                        out,
                    );
                }
                let target = resolve_name(&net.effective_project(project), device, target)?;
                if target.is_broadcast() || target.is_multicast() || project.devices.iter().flat_map(|d|&d.interfaces).any(|i|matches!(&i.ipv4,Some(Ipv4Config::Static{address,prefix_length}) if *prefix_length<31 && address.parse().ok().is_some_and(|a|sns_protocols::subnet(a,*prefix_length).is_ok_and(|(_,b)|b==target)))) {return Err(diag("validation.invalid_broadcast",target.to_string()));}
                self.serial = self
                    .serial
                    .checked_add(1)
                    .ok_or_else(|| diag("simulation.sequence_overflow", "terminal_jobs"))?;
                let token = self.serial;
                self.state(device, application)?.job = Some(Job::Trace {
                    token,
                    target,
                    max,
                    done: false,
                });
                out.effects.push(Effect::Timer(Timer {
                    device: device.clone(),
                    application: application.clone(),
                    token,
                    kind: TimerKind::TraceSend { ttl: 1 },
                    delay_us: 0,
                    correlation: flow.clone(),
                }));
            }
            "host" | "nslookup" => {
                let mut args = args;
                let mut kind = DnsType::A;
                let type_name = if args.first().is_some_and(|a| a == "-t") {
                    if args.len() < 3 {
                        return Err(diag("terminal.syntax", usage(command).unwrap()));
                    }
                    let k = &args[1];
                    args = &args[2..];
                    Some(k.as_str())
                } else if args.first().is_some_and(|a| a.starts_with("-type=")) {
                    let k = args[0].trim_start_matches("-type=");
                    args = &args[1..];
                    Some(k)
                } else {
                    None
                };
                if let Some(k) = type_name {
                    kind = match k.to_ascii_uppercase().as_str() {
                        "A" => DnsType::A,
                        "MX" => DnsType::Mx,
                        "NS" => DnsType::Ns,
                        _ => return Err(diag("terminal.syntax", usage(command).unwrap())),
                    };
                }
                arity(args, 1, 2, command)?;
                let server = args
                    .get(1)
                    .map(|s| {
                        s.parse::<Ipv4Addr>()
                            .map_err(|_| diag("validation.invalid_ipv4", s))
                    })
                    .transpose()?;
                self.dns_query(
                    project,
                    net,
                    now,
                    device,
                    application,
                    &args[0],
                    kind,
                    server,
                    dns::Purpose::Terminal,
                    flow,
                    out,
                )?;
            }
            "dnscache" => {
                if args != ["-d"] {
                    return Err(diag("terminal.syntax", usage(command).unwrap()));
                }
                self.purge_dns(device);
                fact(out, device, flow, "dns.cache.purged", json!({}));
            }
            "ipconfig" => {
                arity(args, 0, 0, command)?;
                let effective = net.effective_project(project);
                let d = effective.device(device).unwrap();
                output = Some(Line::Json {
                    value: json!({"device":d.id,"interfaces":d.interfaces,"network":d.network,"configured_interfaces":project.device(device).unwrap().interfaces}),
                });
            }
            "netstat" => {
                arity(args, 0, 0, command)?;
                output = Some(Line::Json {
                    value: serde_json::to_value(
                        net.transport
                            .views()
                            .into_iter()
                            .filter(|s| &s.device == device)
                            .collect::<Vec<_>>(),
                    )
                    .unwrap(),
                });
            }
            "arp" => {
                arity(args, 0, 2, command)?;
                let (delete, address) = match args {
                    [] => (false, None),
                    [flag] if flag == "-a" => (false, None),
                    [flag] if flag == "-d" => (true, None),
                    [flag, address] if flag == "-a" || flag == "-d" => (
                        flag == "-d",
                        Some(
                            address
                                .parse::<Ipv4Addr>()
                                .map_err(|_| diag("validation.invalid_ipv4", address))?,
                        ),
                    ),
                    _ => return Err(diag("terminal.syntax", usage(command).unwrap())),
                };
                if delete {
                    network(
                        out,
                        flow,
                        if let Some(address) = address {
                            Action::PurgeArpAddress {
                                device: device.clone(),
                                address,
                            }
                        } else {
                            Action::PurgeArp {
                                device: device.clone(),
                            }
                        },
                    );
                } else {
                    let rows:Vec<_>=net.arp.iter().filter(|((port,ip),_)|project.device_for_interface(port).is_some_and(|d|&d.id==device)&&address.is_none_or(|a|a==*ip)).map(|((port,ip),entry)|json!({"interface":port,"address":ip,"mac":entry.mac,"expires_us":entry.expires_us})).collect();
                    output = Some(Line::Json { value: json!(rows) });
                }
            }
            "ping" => {
                let mut target = None;
                let mut count = 4_u16;
                let mut broadcast = false;
                let mut index = 0;
                while index < args.len() {
                    match args[index].as_str() {
                        "-b" => broadcast = true,
                        "-c" | "-n" => {
                            index += 1;
                            count = args
                                .get(index)
                                .and_then(|s| s.parse().ok())
                                .filter(|n| (1..=1024).contains(n))
                                .ok_or_else(|| diag("terminal.syntax", usage(command).unwrap()))?;
                        }
                        s if s.starts_with('-') => {
                            return Err(diag("terminal.syntax", usage(command).unwrap()));
                        }
                        _ => {
                            if target.replace(args[index].as_str()).is_some() {
                                return Err(diag("terminal.syntax", usage(command).unwrap()));
                            }
                        }
                    }
                    index += 1;
                }
                let name =
                    target.ok_or_else(|| diag("terminal.syntax", usage(command).unwrap()))?;
                if needs_dns(project, net, device, name) {
                    return self.dns_query(
                        project,
                        net,
                        now,
                        device,
                        application,
                        name,
                        DnsType::A,
                        None,
                        dns::Purpose::Command {
                            line: format!(
                                "ping -c {count} {} $DNS",
                                if broadcast { "-b" } else { "" }
                            ),
                        },
                        flow,
                        out,
                    );
                }
                let effective = net.effective_project(project);
                let target = resolve_name(&effective, device, name)?;
                let host = project.device(device).unwrap();
                let is_broadcast = target.is_broadcast()
                    || host.interfaces.iter().any(|i| match &i.ipv4 {
                        Some(Ipv4Config::Static {
                            address,
                            prefix_length,
                        }) if *prefix_length < 31 => address.parse().ok().is_some_and(|a| {
                            sns_protocols::subnet(a, *prefix_length).is_ok_and(|(_, b)| b == target)
                        }),
                        _ => false,
                    });
                if is_broadcast && !broadcast {
                    return Err(diag("ipv4.broadcast_requires_opt_in", "-b"));
                }
                self.serial = self
                    .serial
                    .checked_add(1)
                    .ok_or_else(|| diag("simulation.sequence_overflow", "application_jobs"))?;
                let token = self.serial;
                self.state(device, application)?.job = Some(Job::Ping {
                    token,
                    target,
                    count,
                    sent: 0,
                    received: 0,
                    replied: BTreeSet::new(),
                    broadcast: is_broadcast,
                });
                out.effects.push(Effect::Timer(Timer {
                    device: device.clone(),
                    application: application.clone(),
                    token,
                    kind: TimerKind::PingSend { index: 0 },
                    delay_us: 0,
                    correlation: flow.clone(),
                }));
            }
            "tcpdump" => {
                arity(args, 0, 0, command)?;
                self.state(device, application)?.job = Some(Job::Capture);
            }
            "exit" => {
                arity(args, 0, 0, command)?;
                self.state(device, application)?.visible = false;
            }
            "cls" | "reset" => {
                arity(args, 0, 0, command)?;
                let s = self.state(device, application)?;
                s.lines.clear();
                s.omitted = 0;
                push(s, message("terminal_welcome", &[]));
            }
            _ => unreachable!("usage and dispatch share the same allowlist"),
        }
        if let Some(line) = output {
            push(self.state(device, application)?, line);
        }
        let _ = now;
        Ok(())
    }
}
