//! Student-level reading of terminal results. The simulated terminal keeps its
//! structured output; this only chooses how a known shape is presented.
use serde_json::Value;
use sns_apps::Line;
use sns_i18n::Language;

fn s(value: &Value, key: &str) -> Option<String> {
    match value.get(key)? {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}
fn milliseconds(micros: &str) -> String {
    micros
        .parse::<u64>()
        .map_or_else(|_| micros.to_owned(), |us| us.div_ceil(1000).to_string())
}
fn format(language: Language, key: &str, params: &[(&str, &str)]) -> String {
    language.format(key, params).unwrap_or_else(|e| e)
}

/// A readable rendering, or `None` to keep the regular presentation.
/// `command` is the first word of the latest prompt, which produced this line.
pub(super) fn readable(language: Language, command: Option<&str>, line: &Line) -> Option<String> {
    match line {
        Line::Message { key, params } => {
            let get = |name: &str| params.get(name).map(String::as_str).unwrap_or("—");
            match key.as_str() {
                "ping_reply" => Some(format(
                    language,
                    "term_ping_reply",
                    &[("source", get("source")), ("ms", &milliseconds(get("rtt")))],
                )),
                "ping_timeout" => Some(language.text("term_ping_timeout").to_owned()),
                "ping_summary" => Some(format(
                    language,
                    if get("lost") == "0" {
                        "term_ping_summary_ok"
                    } else {
                        "term_ping_summary_lost"
                    },
                    &[
                        ("sent", get("sent")),
                        ("received", get("received")),
                        ("lost", get("lost")),
                    ],
                )),
                _ => None,
            }
        }
        Line::Json { value } => match command? {
            "ipconfig" => ipconfig(language, value),
            "arp" => arp(language, value),
            "route" => route(language, value),
            "ls" | "dir" => list(language, value),
            _ => None,
        },
        _ => None,
    }
}

fn ipconfig(language: Language, value: &Value) -> Option<String> {
    let mut lines = Vec::new();
    for interface in value.get("interfaces")?.as_array()? {
        let id = s(interface, "id").unwrap_or_default();
        let mac = s(interface, "mac").unwrap_or_default();
        match interface
            .get("ipv4")
            .filter(|v| v.get("address").is_some_and(|a| !a.is_null()))
        {
            Some(ipv4) => {
                let mode = language.text(if s(ipv4, "mode").as_deref() == Some("dhcp") {
                    "term_mode_dhcp"
                } else {
                    "term_mode_static"
                });
                lines.push(format(
                    language,
                    "term_ip_card",
                    &[
                        ("id", &id),
                        ("address", &s(ipv4, "address")?),
                        ("prefix", &s(ipv4, "prefix_length")?),
                        ("mode", mode),
                        ("mac", &mac),
                    ],
                ));
            }
            None => lines.push(format(language, "term_ip_none", &[("id", &id)])),
        }
    }
    let network = value.get("network")?;
    lines.push(match s(network, "default_gateway") {
        Some(gateway) => format(language, "term_gateway", &[("gateway", &gateway)]),
        None => language.text("term_gateway_none").to_owned(),
    });
    lines.push(match s(network, "dns_server") {
        Some(dns) => format(language, "term_dns", &[("dns", &dns)]),
        None => language.text("term_dns_none").to_owned(),
    });
    Some(lines.join("\n"))
}

fn arp(language: Language, value: &Value) -> Option<String> {
    let rows = value.as_array()?;
    if rows.is_empty() {
        return Some(language.text("term_arp_empty").to_owned());
    }
    let mut lines = Vec::new();
    for row in rows {
        lines.push(format(
            language,
            "term_arp_row",
            &[
                ("address", &s(row, "address")?),
                ("mac", &s(row, "mac")?),
                ("interface", &s(row, "interface")?),
            ],
        ));
    }
    Some(lines.join("\n"))
}

fn route(language: Language, value: &Value) -> Option<String> {
    let mut lines = Vec::new();
    for row in value.as_array()? {
        let origin = s(row, "origin")?;
        if origin == "loopback" {
            continue;
        }
        let network = s(row, "network")?;
        let prefix = s(row, "prefix")?;
        let gateway = s(row, "gateway");
        let interface = s(row, "interface").unwrap_or_default();
        lines.push(match gateway {
            Some(gateway) if prefix == "0" => {
                format(language, "term_route_default", &[("gateway", &gateway)])
            }
            Some(gateway) => format(
                language,
                "term_route_via",
                &[
                    ("network", &network),
                    ("prefix", &prefix),
                    ("gateway", &gateway),
                ],
            ),
            None => format(
                language,
                "term_route_direct",
                &[
                    ("network", &network),
                    ("prefix", &prefix),
                    ("interface", &interface),
                ],
            ),
        });
    }
    if lines.is_empty() {
        lines.push(language.text("term_route_empty").to_owned());
    }
    Some(lines.join("\n"))
}

fn list(language: Language, value: &Value) -> Option<String> {
    let entries = value.as_array()?;
    if entries.is_empty() {
        return Some(language.text("term_ls_empty").to_owned());
    }
    let mut lines = Vec::new();
    for entry in entries {
        let path = s(entry, "path")?;
        let name = path
            .rsplit('/')
            .find(|p| !p.is_empty())
            .unwrap_or(&path)
            .to_owned();
        lines.push(if entry.get("directory")?.as_bool()? {
            format(language, "term_ls_dir", &[("name", &name)])
        } else {
            format(
                language,
                "term_ls_file",
                &[("name", &name), ("bytes", &s(entry, "bytes")?)],
            )
        });
    }
    Some(lines.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn json(command: &str, value: Value) -> Option<String> {
        readable(Language::Fr, Some(command), &Line::Json { value })
    }

    #[test]
    fn routes_hide_loopback_and_name_the_default_gateway() {
        let text = json(
            "route",
            json!([
                {"network":"127.0.0.0","prefix":8,"gateway":null,"interface":"lo","origin":"loopback","metric":0,"expires_us":null},
                {"network":"192.168.10.0","prefix":24,"gateway":null,"interface":"a-p0","origin":"direct","metric":0,"expires_us":null},
                {"network":"192.168.30.0","prefix":24,"gateway":"192.168.10.9","interface":"a-p0","origin":"static","metric":1,"expires_us":null},
                {"network":"0.0.0.0","prefix":0,"gateway":"192.168.10.1","interface":"a-p0","origin":"default","metric":0,"expires_us":null}
            ]),
        )
        .unwrap();
        assert_eq!(
            text,
            "Réseau 192.168.10.0/24 : relié directement (carte a-p0)\n\
             Réseau 192.168.30.0/24 : en passant par 192.168.10.9\n\
             Tous les autres réseaux : en passant par la passerelle 192.168.10.1"
        );
        assert_eq!(
            json("route", json!([])).unwrap(),
            Language::Fr.text("term_route_empty")
        );
    }

    #[test]
    fn listings_and_unknown_shapes() {
        let text = json(
            "ls",
            json!([
                {"path":"/docs","directory":true,"media_type":"inode/directory","bytes":0},
                {"path":"/notes.txt","directory":false,"media_type":"text/plain","bytes":12}
            ]),
        )
        .unwrap();
        assert_eq!(text, "[dossier] docs\nnotes.txt (12 octets)");
        // netstat and unexpected shapes keep the regular structured presentation.
        assert!(json("netstat", json!([])).is_none());
        assert!(json("ipconfig", json!({"unexpected": true})).is_none());
        assert!(readable(Language::Fr, None, &Line::Json { value: json!([]) }).is_none());
    }

    #[test]
    fn ping_messages_use_milliseconds_and_a_summary() {
        let message = |key: &str, params: &[(&str, &str)]| Line::Message {
            key: key.into(),
            params: params
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
        };
        assert_eq!(
            readable(
                Language::Fr,
                Some("ping"),
                &message("ping_reply", &[("source", "192.168.1.3"), ("rtt", "2400")])
            )
            .unwrap(),
            "Réponse de 192.168.1.3 en 3 ms : ça marche !"
        );
        assert_eq!(
            readable(
                Language::Fr,
                Some("ping"),
                &message(
                    "ping_summary",
                    &[("sent", "4"), ("received", "3"), ("lost", "1")]
                )
            )
            .unwrap(),
            "Bilan : 3 réponse(s) sur 4 envois, 1 perdu(s)."
        );
    }
}
