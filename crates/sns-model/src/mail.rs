//! Portable mail configuration and durable application data, with finite quotas.
use crate::*;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailAccount {
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailServerConfig {
    pub domain: String,
    pub smtp_port: u16,
    pub pop3_port: u16,
    pub accounts: Vec<MailAccount>,
}
impl Default for MailServerConfig {
    fn default() -> Self {
        Self {
            domain: "ecole.test".into(),
            smtp_port: 25,
            pop3_port: 110,
            accounts: vec![],
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailClientConfig {
    pub address: String,
    pub display_name: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub pop3_server: String,
    pub pop3_port: u16,
    pub username: String,
    pub password: String,
}
impl Default for MailClientConfig {
    fn default() -> Self {
        Self {
            address: "alice@ecole.test".into(),
            display_name: "Alice".into(),
            smtp_server: "mail.ecole.test".into(),
            smtp_port: 25,
            pop3_server: "mail.ecole.test".into(),
            pop3_port: 110,
            username: "alice".into(),
            password: "classe".into(),
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailDraft {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailMessage {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub sender_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relay_path: Vec<String>,
    pub id: Id,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body: String,
    pub received_us: Decimal,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contact {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub street: String,
    pub house_number: String,
    pub postal_code: String,
    pub city: String,
    pub email: String,
    pub phone: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailServerData {
    pub mailboxes: BTreeMap<String, Vec<MailMessage>>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MailClientData {
    pub next_id: Decimal,
    pub inbox: Vec<MailMessage>,
    pub sent: Vec<MailMessage>,
    pub contacts: Vec<Contact>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "application",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ApplicationData {
    MailServer(MailServerData),
    MailClient(MailClientData),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailFolder {
    Inbox,
    Sent,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum MailCommand {
    Start,
    Send { draft: MailDraft },
    Receive,
    Reply { message: Id },
    Delete { folder: MailFolder, message: Id },
    UpsertContact { contact: Contact },
    DeleteContact { id: String },
}
pub fn advanced_profile(project: &mut Project) {
    if project.schema_version < 7 {
        project.schema_version = 7;
        project.semantic_profile = "sns-educational-applications-v1".into();
    }
}
fn bad() -> Diagnostic {
    Diagnostic::new("mail.invalid_config", "mail")
}
pub fn mail_address(value: &str) -> Result<String, Diagnostic> {
    let (local, domain) = value.split_once('@').ok_or_else(bad)?;
    if !username_valid(local) || domain.is_empty() {
        return Err(bad());
    }
    let domain = dns_name(domain)?;
    if domain.is_empty() {
        return Err(bad());
    }
    Ok(format!("{}@{domain}", local.to_ascii_lowercase()))
}
pub fn username_valid(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 64
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}
fn single(s: &str, max: usize) -> bool {
    s.len() <= max && !s.chars().any(char::is_control)
}
impl MailDraft {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if self.to.len() + self.cc.len() + self.bcc.len() == 0
            || self.to.len() + self.cc.len() + self.bcc.len() > 32
            || !single(&self.subject, 512)
            || self.body.len() > 32768
            || self.body.contains('\0')
        {
            return Err(bad());
        }
        for a in self.to.iter().chain(&self.cc).chain(&self.bcc) {
            mail_address(a)?;
        }
        Ok(())
    }
}
impl MailMessage {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        mail_address(&self.from)?;
        if !single(&self.sender_name, 256) {
            return Err(bad());
        }
        if self.relay_path.len() > 8 {
            return Err(bad());
        }
        for d in &self.relay_path {
            dns_name(d)?;
        }
        MailDraft {
            to: if self.to.is_empty() && self.cc.is_empty() && self.bcc.is_empty() {
                vec![self.from.clone()]
            } else {
                self.to.clone()
            },
            cc: self.cc.clone(),
            bcc: self.bcc.clone(),
            subject: self.subject.clone(),
            body: self.body.clone(),
        }
        .validate()
    }
}
impl MailServerConfig {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        if dns_name(&self.domain)?.is_empty()
            || self.smtp_port == 0
            || self.pop3_port == 0
            || self.smtp_port == self.pop3_port
            || self.accounts.len() > 64
        {
            return Err(bad());
        }
        let mut seen = BTreeSet::new();
        for a in &self.accounts {
            if !username_valid(&a.username)
                || !seen.insert(a.username.to_ascii_lowercase())
                || a.password.is_empty()
                || !single(&a.password, 128)
                || !single(&a.first_name, 256)
                || !single(&a.last_name, 256)
            {
                return Err(bad());
            }
        }
        Ok(())
    }
}
impl MailClientConfig {
    pub fn validate(&self) -> Result<(), Diagnostic> {
        mail_address(&self.address)?;
        if self.smtp_port == 0
            || self.pop3_port == 0
            || !username_valid(&self.username)
            || self.password.is_empty()
            || !single(&self.password, 128)
            || !single(&self.display_name, 256)
        {
            return Err(bad());
        }
        for host in [&self.smtp_server, &self.pop3_server] {
            if dns_name(host)?.is_empty() {
                return Err(bad());
            }
        }
        Ok(())
    }
}
impl ApplicationData {
    pub fn validate(&self, kind: &str) -> Result<(), Diagnostic> {
        if serde_json::to_vec(self).map_err(|_| bad())?.len() > 1_048_576 {
            return Err(Diagnostic::new("resource.limit_exceeded", "mail_storage"));
        }
        match self {
            Self::MailServer(s) => {
                if kind != "mail_server"
                    || s.mailboxes.len() > 64
                    || s.mailboxes.values().map(Vec::len).sum::<usize>() > 128
                {
                    return Err(bad());
                }
                for (k, v) in &s.mailboxes {
                    if !username_valid(k) {
                        return Err(bad());
                    }
                    let mut ids = BTreeSet::new();
                    for m in v {
                        m.validate()?;
                        if !m.bcc.is_empty() || !ids.insert(&m.id) {
                            return Err(bad());
                        }
                    }
                }
            }
            Self::MailClient(c) => {
                if kind != "mail_client"
                    || c.inbox.len() + c.sent.len() > 256
                    || c.contacts.len() > 128
                {
                    return Err(bad());
                }
                for folder in [&c.inbox, &c.sent] {
                    let mut ids = BTreeSet::new();
                    for m in folder {
                        m.validate()?;
                        if !ids.insert(&m.id) {
                            return Err(bad());
                        }
                    }
                }
                let mut ids = BTreeSet::new();
                for v in &c.contacts {
                    if v.id.is_empty()
                        || !ids.insert(&v.id)
                        || [
                            &v.id,
                            &v.first_name,
                            &v.last_name,
                            &v.street,
                            &v.house_number,
                            &v.postal_code,
                            &v.city,
                            &v.phone,
                        ]
                        .iter()
                        .any(|v| !single(v, 256))
                    {
                        return Err(bad());
                    }
                    mail_address(&v.email)?;
                }
            }
        }
        Ok(())
    }
}
