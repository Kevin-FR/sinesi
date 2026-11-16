//! Literal message and contact data from known JavaBeans fields and virtual files.
use super::*;
use crate::xml::Source;
use roxmltree::Node;

pub(super) fn issue_at(report: &mut ImportReport, source: &str, code: &str, detail: &str) {
    report.issues.push(ImportIssue {
        code: code.into(),
        source: source.into(),
        detail: detail.into(),
    });
}
pub(super) fn note_file(report: &mut ImportReport, source: &str, target: &str) {
    report.converted.push(FieldConversion {
        source: source.into(),
        target: target.into(),
        rule: "known_property".into(),
    });
}
pub(super) fn field(s: &Source<'_>, n: Node<'_, '_>, name: &str) -> Result<String, Diagnostic> {
    Ok(crate::network::literal(s, n, name)?
        .map(|(_, value)| value)
        .unwrap_or_default())
}
pub(super) fn mark_fields(
    s: &Source<'_>,
    n: Node<'_, '_>,
    target: &str,
    fields: &[&str],
) -> Result<(), Diagnostic> {
    for name in fields {
        if let Some(property) = s.property(n, name)? {
            s.note(property, &format!("{target}.{name}"));
        }
    }
    Ok(())
}
pub(super) fn virtual_text(files: &[VirtualFile], path: &str) -> Option<String> {
    let file = files
        .iter()
        .find(|file| file.path == path && file.media_type == "text/plain")?;
    String::from_utf8(STANDARD.decode(&file.data_base64).ok()?).ok()
}

pub(super) struct Budget<'a> {
    application: &'a str,
    remaining: usize,
    messages: usize,
    limit: usize,
}
impl<'a> Budget<'a> {
    pub(super) fn new(application: &'a str, limit: usize) -> Self {
        // Reserve space for the envelope, account keys and separators.
        Self {
            application,
            remaining: 1_048_576 - 8192,
            messages: 0,
            limit,
        }
    }
    pub(super) fn accept(&mut self, value: &impl Serialize) -> bool {
        let Ok(bytes) = serde_json::to_vec(value) else {
            return false;
        };
        if bytes.len() >= self.remaining {
            return false;
        }
        self.remaining -= bytes.len() + 1;
        true
    }
    fn message_id(&self) -> Id {
        id(format!(
            "{}-legacy-mail-{}",
            self.application, self.messages
        ))
    }
}

fn address_text(text: &str) -> Option<(String, String)> {
    let text = text.trim();
    if let Some((name, address)) = text.split_once('<') {
        let address = address.strip_suffix('>')?;
        Some((mail_address(address.trim()).ok()?, name.trim().to_owned()))
    } else {
        Some((mail_address(text).ok()?, String::new()))
    }
}

fn xml_address(s: &Source<'_>, node: Node<'_, '_>) -> Result<Option<(String, String)>, Diagnostic> {
    let node = s.view(node)?;
    if node.has_tag_name("object")
        && node.attribute("class") != Some("filius.software.email.AddressEntry")
    {
        return Ok(None);
    }
    let raw = field(s, node, "mailAddress")?;
    if !raw.is_empty() {
        let Some(address) = mail_address(raw.trim()).ok() else {
            return Ok(None);
        };
        return Ok(Some((address, field(s, node, "name")?)));
    }
    Ok(s.value(node)?.as_deref().and_then(address_text))
}

fn xml_recipients(
    s: &Source<'_>,
    node: Node<'_, '_>,
    property: &str,
) -> Result<Option<Vec<String>>, Diagnostic> {
    let Some(list) = s.property(node, property)? else {
        return Ok(Some(vec![]));
    };
    let mut recipients = Vec::new();
    for row in s.view(list)?.children().filter(Node::is_element) {
        if row.attribute("method") != Some("add") {
            return Ok(None);
        }
        let Some((address, _)) = xml_address(s, row)? else {
            return Ok(None);
        };
        recipients.push(address);
        if recipients.len() > 32 {
            return Ok(None);
        }
    }
    Ok(Some(recipients))
}

struct Decoded {
    from: String,
    sender_name: String,
    to: Vec<String>,
    cc: Vec<String>,
    bcc: Vec<String>,
    subject: String,
    body: String,
    dated: bool,
}
fn xml_message(s: &Source<'_>, node: Node<'_, '_>) -> Result<Option<Decoded>, Diagnostic> {
    if node.attribute("class") != Some("filius.software.email.Email") {
        return Ok(None);
    }
    // JavaBeans may emit an initial getter and a final assignment to its alias.
    let sender = node
        .children()
        .rfind(|n| n.attribute("property") == Some("absender"));
    let Some(sender) = sender else {
        return Ok(None);
    };
    let Some((from, sender_name)) = xml_address(s, sender)? else {
        return Ok(None);
    };
    let (Some(to), Some(cc), Some(bcc)) = (
        xml_recipients(s, node, "empfaenger")?,
        xml_recipients(s, node, "cc")?,
        xml_recipients(s, node, "bcc")?,
    ) else {
        return Ok(None);
    };
    Ok(Some(Decoded {
        from,
        sender_name,
        to,
        cc,
        bcc,
        subject: field(s, node, "betreff")?,
        body: field(s, node, "text")?,
        dated: !field(s, node, "dateReceived")?.is_empty(),
    }))
}
fn file_recipients(text: &str) -> Option<Vec<String>> {
    text.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(33)
        .map(|value| address_text(value).map(|(address, _)| address))
        .collect()
}
fn file_message(row: &str) -> Option<Decoded> {
    let mut values = row.split('$');
    let (from, sender_name) = address_text(values.next()?)?;
    let to = file_recipients(values.next()?)?;
    let cc = file_recipients(values.next()?)?;
    let bcc = file_recipients(values.next()?)?;
    let dated = !values.next()?.is_empty();
    let subject = values.next()?.to_owned();
    let body = values
        .next()?
        .replace('|', "\n")
        .replace("&&00036&&", "$")
        .replace("&&00059&&", ";")
        .replace("&&00124&&", "|")
        .replace("&&00035&&", "#");
    if values.next().is_some() {
        return None;
    }
    Some(Decoded {
        from,
        sender_name,
        to,
        cc,
        bcc,
        subject,
        body,
        dated,
    })
}

fn accept_message(
    decoded: Option<Decoded>,
    private: bool,
    budget: &mut Budget<'_>,
    source: &str,
    report: &mut ImportReport,
) -> Option<(MailMessage, bool)> {
    let Some(decoded) = decoded else {
        issue_at(report, source, "import.unsupported_data", "mail_message");
        return None;
    };
    let redacted = !private && !decoded.bcc.is_empty();
    let message = MailMessage {
        id: budget.message_id(),
        from: decoded.from,
        sender_name: decoded.sender_name,
        to: decoded.to,
        cc: decoded.cc,
        bcc: if private { decoded.bcc } else { vec![] },
        subject: decoded.subject,
        body: decoded.body,
        relay_path: vec![],
        received_us: Decimal(0),
    };
    if message.validate().is_err() {
        issue_at(report, source, "import.invalid_value", "mail_message");
        return None;
    }
    if budget.messages >= budget.limit || !budget.accept(&message) {
        issue_at(
            report,
            source,
            "import.unsupported_data",
            "mail_storage_limit",
        );
        return None;
    }
    budget.messages += 1;
    if redacted {
        issue_at(
            report,
            source,
            "import.unsupported_data",
            "mail_bcc_preserved_in_original",
        );
    }
    if decoded.dated {
        issue_at(
            report,
            source,
            "import.unsupported_data",
            "mail_wall_clock_date",
        );
    }
    Some((message, redacted))
}

pub(super) fn xml_folder(
    s: &Source<'_>,
    bean: Node<'_, '_>,
    name: &str,
    target: &str,
    private: bool,
    budget: &mut Budget<'_>,
    report: &mut ImportReport,
) -> Result<Vec<MailMessage>, Diagnostic> {
    let Some(list) = s.property(bean, name)? else {
        return Ok(vec![]);
    };
    let mut messages = Vec::new();
    let mut complete = true;
    for row in s.view(list)?.children().filter(Node::is_element) {
        let node = s.view(row)?;
        let decoded = if row.attribute("method") == Some("add") {
            xml_message(s, node)?
        } else {
            None
        };
        let Some((message, redacted)) =
            accept_message(decoded, private, budget, &s.location(row), report)
        else {
            complete = false;
            continue;
        };
        let dest = format!("{target}[{}]", messages.len());
        s.note(row, &dest);
        mark_fields(s, node, &dest, &["betreff", "text"])?;
        let senders = node
            .children()
            .filter(|n| n.attribute("property") == Some("absender"));
        if let Some(last) = senders.clone().next_back() {
            let selected = s.view(last)?;
            for property in senders {
                if s.view(property)? == selected {
                    s.note(property, &format!("{dest}.from"));
                    mark_fields(s, property, &dest, &["mailAddress", "name"])?;
                }
            }
        }
        for (name, field) in [("empfaenger", "to"), ("cc", "cc"), ("bcc", "bcc")] {
            if field == "bcc" && redacted {
                continue;
            }
            if let Some(list) = s.property(node, name)? {
                let dest = format!("{dest}.{field}");
                s.note(list, &dest);
                for (index, row) in s
                    .view(list)?
                    .children()
                    .filter(Node::is_element)
                    .enumerate()
                {
                    mark_fields(s, row, &format!("{dest}[{index}]"), &["mailAddress"])?;
                }
            }
        }
        messages.push(message);
    }
    if complete {
        s.note(list, target);
    }
    Ok(messages)
}

pub(super) fn file_folder(
    text: &str,
    source: &str,
    target: &str,
    budget: &mut Budget<'_>,
    report: &mut ImportReport,
) -> Vec<MailMessage> {
    if text.is_empty() {
        return vec![];
    }
    let Some(rows) = text.strip_prefix('#') else {
        issue_at(
            report,
            source,
            "import.unsupported_data",
            "mail_file_messages",
        );
        return vec![];
    };
    let mut messages = Vec::new();
    for (index, row) in rows.split('#').enumerate() {
        let source = format!("{source}:message-{}", index + 1);
        if let Some((message, _)) =
            accept_message(file_message(row), false, budget, &source, report)
        {
            note_file(report, &source, &format!("{target}[{}]", messages.len()));
            messages.push(message);
        }
    }
    messages
}

pub(super) fn contacts(
    s: &Source<'_>,
    bean: Node<'_, '_>,
    target: &str,
    budget: &mut Budget<'_>,
    report: &mut ImportReport,
) -> Result<Vec<Contact>, Diagnostic> {
    let Some(list) = s.property(bean, "adressbuch")? else {
        return Ok(vec![]);
    };
    let mut contacts = Vec::new();
    let mut complete = true;
    for (index, row) in s
        .view(list)?
        .children()
        .filter(Node::is_element)
        .enumerate()
    {
        let contact = s.view(row)?;
        let source = s.location(row);
        if row.attribute("method") != Some("add")
            || contact.attribute("class") != Some("filius.software.email.Kontakt")
        {
            issue_at(report, &source, "import.unsupported_data", "mail_contact");
            complete = false;
            continue;
        }
        let entry = Contact {
            id: format!("legacy-contact-{index}"),
            first_name: field(s, contact, "vorname")?,
            last_name: field(s, contact, "name")?,
            street: field(s, contact, "strasse")?,
            house_number: field(s, contact, "hausnr")?,
            postal_code: field(s, contact, "plz")?,
            city: field(s, contact, "wohnort")?,
            email: field(s, contact, "email")?,
            phone: field(s, contact, "telefon")?,
        };
        let probe = ApplicationData::MailClient(MailClientData {
            contacts: vec![entry.clone()],
            ..Default::default()
        });
        if contacts.len() >= 128 || probe.validate("mail_client").is_err() || !budget.accept(&entry)
        {
            issue_at(report, &source, "import.unsupported_data", "mail_contact");
            complete = false;
            continue;
        }
        let dest = format!("{target}[{}]", contacts.len());
        s.note(row, &dest);
        mark_fields(
            s,
            contact,
            &dest,
            &[
                "vorname", "name", "strasse", "hausnr", "plz", "wohnort", "email", "telefon",
            ],
        )?;
        contacts.push(entry);
    }
    if complete {
        s.note(list, target);
    }
    Ok(contacts)
}
