//! Import mail identities and durable content without starting services.
use super::*;
use crate::{mail_content::*, xml::Source};
use roxmltree::Node;

const ACCOUNT_CLASS: &str = "filius.software.email.EmailKonto";
const CLIENT_FILE: &str = "/konten.txt";
const SERVER_FILE: &str = "/mailserver/konten.txt";
const SERVER_FIELDS: &[&str] = &["benutzername", "passwort", "vorname", "nachname"];
const CLIENT_FIELDS: &[&str] = &[
    "emailAdresse",
    "vorname",
    "nachname",
    "smtpserver",
    "smtpport",
    "pop3server",
    "pop3port",
    "benutzername",
    "passwort",
];

type MailImport = (Option<ServiceConfig>, Option<ApplicationData>);

pub(super) fn configuration(
    s: &Source<'_>,
    bean: Node<'_, '_>,
    software: Software,
    target: &str,
    files: &[VirtualFile],
    report: &mut ImportReport,
) -> Result<MailImport, Diagnostic> {
    if software == Software::MailServer {
        server(s, bean, target, files, report)
    } else {
        client(s, bean, target, files, report)
    }
}

fn file_location(target: &str, path: &str, index: usize) -> String {
    format!("{target}:virtual{path}:line-{}", index + 1)
}

fn xml_client(s: &Source<'_>, bean: Node<'_, '_>) -> Result<MailClientConfig, Diagnostic> {
    let port = |name, default| -> Result<u16, Diagnostic> {
        Ok(crate::network::literal(s, bean, name)?
            .map_or(default, |(_, value)| value.parse().unwrap_or(0)))
    };
    Ok(MailClientConfig {
        address: field(s, bean, "emailAdresse")?,
        display_name: format!(
            "{} {}",
            field(s, bean, "vorname")?,
            field(s, bean, "nachname")?
        )
        .trim()
        .to_owned(),
        smtp_server: field(s, bean, "smtpserver")?,
        smtp_port: port("smtpport", 25)?,
        pop3_server: field(s, bean, "pop3server")?,
        pop3_port: port("pop3port", 110)?,
        username: field(s, bean, "benutzername")?,
        password: field(s, bean, "passwort")?,
    })
}

fn file_client(line: &str) -> Option<MailClientConfig> {
    let parts: Vec<_> = line.splitn(10, ';').collect();
    let [
        pop,
        smtp,
        pop_port,
        smtp_port,
        user,
        password,
        last,
        first,
        address,
    ] = parts.as_slice()
    else {
        return None;
    };
    Some(MailClientConfig {
        address: (*address).into(),
        display_name: format!("{first} {last}").trim().into(),
        smtp_server: (*smtp).into(),
        smtp_port: smtp_port.parse().ok()?,
        pop3_server: (*pop).into(),
        pop3_port: pop_port.parse().ok()?,
        username: (*user).into(),
        password: (*password).into(),
    })
}

fn client(
    s: &Source<'_>,
    bean: Node<'_, '_>,
    target: &str,
    files: &[VirtualFile],
    report: &mut ImportReport,
) -> Result<MailImport, Diagnostic> {
    let mut configuration = None;
    if let Some(text) = virtual_text(files, CLIENT_FILE) {
        for (index, line) in text
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
        {
            let source = file_location(target, CLIENT_FILE, index);
            if let Some(config) = file_client(line).filter(|c| c.validate().is_ok()) {
                if let Some((_, previous)) = configuration.replace((config, source)) {
                    issue_at(
                        report,
                        &previous,
                        "import.unsupported_data",
                        "additional_mail_identities_preserved_in_original",
                    );
                }
            } else {
                issue_at(report, &source, "import.invalid_value", "mail_identity");
            }
        }
    }
    let from_file = configuration.as_ref().map(|(cfg, source)| {
        note_file(report, source, &format!("{target}.service"));
        cfg.clone()
    });
    let mut selected = from_file.clone();
    if let Some(list) = s.property(bean, "kontoListe")? {
        let mut complete = true;
        let mut matched = false;
        for row in s.view(list)?.children().filter(Node::is_element) {
            let node = if row.attribute("method") == Some("put") {
                row.children()
                    .find(|n| n.has_tag_name("object"))
                    .map(|n| s.view(n))
                    .transpose()?
            } else if row.attribute("method") == Some("add") {
                Some(s.view(row)?)
            } else {
                None
            };
            let source = s.location(row);
            let Some(node) = node.filter(|n| n.attribute("class") == Some(ACCOUNT_CLASS)) else {
                issue_at(report, &source, "import.unsupported_data", "mail_identity");
                complete = false;
                continue;
            };
            let cfg = xml_client(s, node)?;
            if cfg.validate().is_err() {
                issue_at(report, &source, "import.invalid_value", "mail_identity");
                complete = false;
                continue;
            }
            if selected.is_none() || (!matched && from_file.as_ref() == Some(&cfg)) {
                selected = Some(cfg);
                matched = true;
                s.note(row, &format!("{target}.service"));
                mark_fields(s, node, target, CLIENT_FIELDS)?;
            } else {
                complete = false;
                issue_at(
                    report,
                    &source,
                    "import.unsupported_data",
                    "additional_mail_identities_preserved_in_original",
                );
            }
        }
        if complete {
            s.note(list, &format!("{target}.service"));
        }
    }
    if selected.is_none() {
        crate::convert::issue(
            report,
            s,
            bean,
            "import.unsupported_data",
            "mail_identity_requires_configuration",
        );
    }
    let mut budget = Budget::new(target, 256);
    let inbox = xml_folder(
        s,
        bean,
        "empfangeneNachrichten",
        &format!("{target}.data.inbox"),
        false,
        &mut budget,
        report,
    )?;
    let sent = xml_folder(
        s,
        bean,
        "gesendeteNachrichten",
        &format!("{target}.data.sent"),
        true,
        &mut budget,
        report,
    )?;
    let contacts = contacts(
        s,
        bean,
        &format!("{target}.data.contacts"),
        &mut budget,
        report,
    )?;
    let data = (!inbox.is_empty() || !sent.is_empty() || !contacts.is_empty()).then_some(
        ApplicationData::MailClient(MailClientData {
            inbox,
            sent,
            contacts,
            next_id: Decimal(0),
        }),
    );
    Ok((selected.map(ServiceConfig::MailClient), data))
}

fn push_account(config: &mut MailServerConfig, account: MailAccount) -> bool {
    config.accounts.push(account);
    if config.validate().is_ok() {
        true
    } else {
        config.accounts.pop();
        false
    }
}

fn server(
    s: &Source<'_>,
    bean: Node<'_, '_>,
    target: &str,
    files: &[VirtualFile],
    report: &mut ImportReport,
) -> Result<MailImport, Diagnostic> {
    let domain = crate::network::literal(s, bean, "mailDomain")?;
    let mut config = MailServerConfig {
        domain: domain
            .as_ref()
            .map_or("filius.de", |(_, value)| value)
            .trim_end_matches('.')
            .to_ascii_lowercase(),
        ..Default::default()
    };
    if config.validate().is_err() {
        crate::convert::issue(report, s, bean, "import.invalid_value", "mail_domain");
        return Ok((None, None));
    }
    if let Some((property, _)) = domain {
        s.note(property, &format!("{target}.service.domain"));
    }
    let mut budget = Budget::new(target, 128);
    let mut mailboxes = BTreeMap::new();
    if let Some(text) = virtual_text(files, SERVER_FILE) {
        for (index, line) in text
            .lines()
            .enumerate()
            .filter(|(_, line)| !line.is_empty())
        {
            let source = file_location(target, SERVER_FILE, index);
            let parts: Vec<_> = line.splitn(6, ';').collect();
            let [username, _, password, last, first, body] = parts.as_slice() else {
                issue_at(
                    report,
                    &source,
                    "import.invalid_value",
                    "mail_server_account",
                );
                continue;
            };
            let user = username.to_ascii_lowercase();
            if !push_account(
                &mut config,
                MailAccount {
                    username: (*username).into(),
                    password: (*password).into(),
                    first_name: (*first).into(),
                    last_name: (*last).into(),
                },
            ) {
                issue_at(
                    report,
                    &source,
                    "import.invalid_value",
                    "mail_server_account",
                );
                continue;
            }
            note_file(
                report,
                &source,
                &format!("{target}.service.accounts[{}]", config.accounts.len() - 1),
            );
            let inbox = file_folder(
                body,
                &source,
                &format!("{target}.data.mailboxes[{}]", config.accounts.len() - 1),
                &mut budget,
                report,
            );
            if !inbox.is_empty() {
                mailboxes.insert(user, inbox);
            }
        }
    }
    if let Some(list) = s.property(bean, "listeBenutzerkonten")? {
        if config.accounts.is_empty() {
            let mut complete = true;
            for row in s.view(list)?.children().filter(Node::is_element) {
                let node = s.view(row)?;
                if row.attribute("method") != Some("add")
                    || node.attribute("class") != Some(ACCOUNT_CLASS)
                {
                    crate::convert::issue(
                        report,
                        s,
                        row,
                        "import.unsupported_data",
                        "mail_server_account",
                    );
                    complete = false;
                    continue;
                }
                let username = field(s, node, "benutzername")?;
                let user = username.to_ascii_lowercase();
                if !push_account(
                    &mut config,
                    MailAccount {
                        username,
                        password: field(s, node, "passwort")?,
                        first_name: field(s, node, "vorname")?,
                        last_name: field(s, node, "nachname")?,
                    },
                ) {
                    crate::convert::issue(
                        report,
                        s,
                        row,
                        "import.invalid_value",
                        "mail_server_account",
                    );
                    complete = false;
                    continue;
                }
                let index = config.accounts.len() - 1;
                s.note(row, &format!("{target}.service.accounts[{index}]"));
                mark_fields(s, node, target, SERVER_FIELDS)?;
                let inbox = xml_folder(
                    s,
                    node,
                    "nachrichten",
                    &format!("{target}.data.mailboxes[{index}]"),
                    false,
                    &mut budget,
                    report,
                )?;
                if !inbox.is_empty() {
                    mailboxes.insert(user, inbox);
                }
            }
            if complete {
                s.note(list, &format!("{target}.service.accounts"));
            }
        } else {
            crate::convert::issue(
                report,
                s,
                list,
                "import.unsupported_data",
                "mail_server_xml_snapshot_preserved_in_original",
            );
        }
    }
    let data = (!mailboxes.is_empty())
        .then_some(ApplicationData::MailServer(MailServerData { mailboxes }));
    Ok((Some(ServiceConfig::MailServer(config)), data))
}
