//! Small, bounded HTML prototype. This module has no loader, script engine or OS I/O.
use std::collections::BTreeMap;

const MAX_BYTES: usize = 65_536;
const MAX_NODES: usize = 2_048;
const MAX_DEPTH: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Text {
        text: String,
        heading: bool,
    },
    Link {
        text: String,
        target: String,
    },
    Image {
        alt: String,
        source: String,
    },
    Form {
        action: String,
        fields: Vec<Field>,
        submit: String,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub label: String,
    pub value: String,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    pub blocks: Vec<Block>,
}
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("html.{code}: {detail}")]
pub struct Error {
    pub code: &'static str,
    pub detail: String,
}
fn error(code: &'static str, detail: impl Into<String>) -> Error {
    Error {
        code,
        detail: detail.into(),
    }
}

#[derive(Default)]
struct Node {
    tag: String,
    attrs: BTreeMap<String, String>,
    text: String,
    children: Vec<Node>,
}
impl Node {
    fn text(&self) -> String {
        if self.tag.is_empty() {
            return self.text.clone();
        }
        self.children
            .iter()
            .map(Self::text)
            .collect::<Vec<_>>()
            .join(" ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

pub fn parse(source: &str) -> Result<Document, Error> {
    if source.len() > MAX_BYTES {
        return Err(error("limit_exceeded", "document bytes"));
    }
    let mut stack = vec![Node {
        tag: "root".into(),
        ..Default::default()
    }];
    let mut remaining = source;
    let mut count = 0;
    while !remaining.is_empty() {
        count += 1;
        if count > MAX_NODES {
            return Err(error("limit_exceeded", "nodes"));
        }
        if remaining.starts_with('<') {
            let end = tag_end(remaining)?;
            let token = &remaining[1..end];
            remaining = &remaining[end + 1..];
            if token.starts_with(['!', '?']) {
                return Err(error("unsupported_markup", token));
            }
            if let Some(close) = token.strip_prefix('/') {
                let close = close.trim().to_ascii_lowercase();
                if stack.len() <= 1 || stack.last().is_none_or(|node| node.tag != close) {
                    return Err(error("unbalanced_tag", close));
                }
                let node = stack
                    .pop()
                    .ok_or_else(|| error("unbalanced_tag", "stack"))?;
                stack
                    .last_mut()
                    .ok_or_else(|| error("unbalanced_tag", "root"))?
                    .children
                    .push(node);
            } else {
                let (tag, attrs, self_closing) = opening(token)?;
                let void = matches!(tag.as_str(), "input" | "img" | "br");
                let node = Node {
                    tag,
                    attrs,
                    ..Default::default()
                };
                if void || self_closing {
                    stack
                        .last_mut()
                        .ok_or_else(|| error("invalid_document", "root"))?
                        .children
                        .push(node);
                } else {
                    if stack.len() >= MAX_DEPTH {
                        return Err(error("limit_exceeded", "depth"));
                    }
                    stack.push(node);
                }
            }
        } else {
            let end = remaining.find('<').unwrap_or(remaining.len());
            let text = decode_entities(&remaining[..end])?;
            remaining = &remaining[end..];
            if !text.trim().is_empty() {
                stack
                    .last_mut()
                    .ok_or_else(|| error("invalid_document", "root"))?
                    .children
                    .push(Node {
                        text,
                        ..Default::default()
                    });
            }
        }
    }
    if stack.len() != 1 {
        return Err(error("unbalanced_tag", "missing closing tag"));
    }
    let mut document = Document::default();
    for node in &stack[0].children {
        emit(node, &mut document.blocks)?;
    }
    Ok(document)
}

fn tag_end(source: &str) -> Result<usize, Error> {
    let mut quote = None;
    for (index, ch) in source.char_indices().skip(1) {
        match (quote, ch) {
            (Some(q), c) if q == c => quote = None,
            (None, '\'' | '"') => quote = Some(ch),
            (None, '>') => return Ok(index),
            _ => {}
        }
    }
    Err(error("invalid_tag", "unterminated tag or attribute"))
}

fn opening(token: &str) -> Result<(String, BTreeMap<String, String>, bool), Error> {
    let self_closing = token.trim_end().ends_with('/');
    let token = if self_closing {
        token.trim_end().trim_end_matches('/')
    } else {
        token
    };
    let split = token.find(char::is_whitespace).unwrap_or(token.len());
    let tag = token[..split].to_ascii_lowercase();
    if !matches!(
        tag.as_str(),
        "html"
            | "body"
            | "main"
            | "h1"
            | "h2"
            | "p"
            | "br"
            | "a"
            | "img"
            | "form"
            | "input"
            | "label"
            | "button"
            | "strong"
            | "em"
            | "b"
            | "i"
    ) {
        return Err(error("unsupported_tag", tag));
    }
    let mut attrs = BTreeMap::new();
    let mut rest = token[split..].trim();
    while !rest.is_empty() {
        let key_end = rest
            .find(|c: char| c == '=' || c.is_whitespace())
            .unwrap_or(rest.len());
        let key = rest[..key_end].to_ascii_lowercase();
        rest = rest[key_end..].trim_start();
        rest = rest
            .strip_prefix('=')
            .ok_or_else(|| error("invalid_attribute", &key))?
            .trim_start();
        let quote = rest
            .chars()
            .next()
            .ok_or_else(|| error("invalid_attribute", &key))?;
        if !matches!(quote, '\'' | '"') {
            return Err(error(
                "invalid_attribute",
                "attribute values must be quoted",
            ));
        }
        rest = &rest[1..];
        let end = rest
            .find(quote)
            .ok_or_else(|| error("invalid_attribute", &key))?;
        let value = decode_entities(&rest[..end])?;
        rest = rest[end + 1..].trim_start();
        let allowed = match tag.as_str() {
            "a" => key == "href",
            "img" => matches!(key.as_str(), "src" | "alt"),
            "form" => matches!(key.as_str(), "method" | "action"),
            "input" => matches!(key.as_str(), "name" | "value" | "type" | "id"),
            "label" => key == "for",
            _ => false,
        };
        if !allowed || value.len() > 2_048 || attrs.insert(key.clone(), value.clone()).is_some() {
            return Err(error("unsupported_attribute", key));
        }
        if matches!(key.as_str(), "href" | "src" | "action")
            && !sns_model::virtual_path_valid(&value)
        {
            return Err(error("external_resource_denied", value));
        }
    }
    Ok((tag, attrs, self_closing))
}

fn required<'a>(node: &'a Node, key: &str) -> Result<&'a str, Error> {
    node.attrs
        .get(key)
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| error("missing_attribute", format!("{}.{key}", node.tag)))
}
fn emit(node: &Node, blocks: &mut Vec<Block>) -> Result<(), Error> {
    match node.tag.as_str() {
        "h1" | "h2" => blocks.push(Block::Text {
            text: node.text(),
            heading: true,
        }),
        "a" => blocks.push(Block::Link {
            text: node.text(),
            target: required(node, "href")?.into(),
        }),
        "img" => blocks.push(Block::Image {
            alt: required(node, "alt")?.into(),
            source: required(node, "src")?.into(),
        }),
        "form" => {
            if !required(node, "method")?.eq_ignore_ascii_case("post") {
                return Err(error("unsupported_method", "only POST forms in J0"));
            }
            let mut fields = Vec::new();
            let mut submit = "Envoyer".to_owned();
            form_fields(node, None, &mut fields, &mut submit)?;
            if fields.len() > 32 {
                return Err(error("limit_exceeded", "form fields"));
            }
            let mut names = std::collections::BTreeSet::new();
            if fields.iter().any(|field| !names.insert(&field.name)) {
                return Err(error("invalid_form", "duplicate field"));
            }
            blocks.push(Block::Form {
                action: required(node, "action")?.into(),
                fields,
                submit,
            });
        }
        "input" | "button" => return Err(error("invalid_form", "control outside form")),
        "" => blocks.push(Block::Text {
            text: node.text(),
            heading: false,
        }),
        _ => {
            for child in &node.children {
                emit(child, blocks)?;
            }
        }
    }
    Ok(())
}
fn form_fields(
    node: &Node,
    label: Option<&str>,
    fields: &mut Vec<Field>,
    submit: &mut String,
) -> Result<(), Error> {
    if node.children.iter().any(|child| child.tag == "form") {
        return Err(error("invalid_form", "nested form"));
    }
    match node.tag.as_str() {
        "input" => {
            if node.attrs.get("type").is_some_and(|kind| kind != "text") {
                return Err(error("unsupported_input", "only text in J0"));
            }
            let name = required(node, "name")?.to_owned();
            fields.push(Field {
                label: label.filter(|s| !s.is_empty()).unwrap_or(&name).into(),
                name,
                value: node.attrs.get("value").cloned().unwrap_or_default(),
            });
        }
        "button" => *submit = node.text(),
        "form" if label.is_some() => return Err(error("invalid_form", "nested form")),
        "label" => {
            let text = node.text();
            for child in &node.children {
                form_fields(child, Some(&text), fields, submit)?;
            }
        }
        "a" | "img" => return Err(error("unsupported_form_content", &node.tag)),
        _ => {
            for child in &node.children {
                form_fields(child, label, fields, submit)?;
            }
        }
    }
    Ok(())
}

fn decode_entities(source: &str) -> Result<String, Error> {
    let mut result = String::new();
    let mut rest = source;
    while let Some(start) = rest.find('&') {
        result.push_str(&rest[..start]);
        rest = &rest[start + 1..];
        let end = rest
            .find(';')
            .ok_or_else(|| error("invalid_entity", "missing semicolon"))?;
        let entity = &rest[..end];
        let ch = match entity {
            "amp" => '&',
            "lt" => '<',
            "gt" => '>',
            "quot" => '"',
            "apos" => '\'',
            "nbsp" => ' ',
            _ => {
                let number = if let Some(hex) = entity.strip_prefix("#x") {
                    u32::from_str_radix(hex, 16).ok()
                } else {
                    entity.strip_prefix('#').and_then(|v| v.parse().ok())
                };
                number
                    .and_then(char::from_u32)
                    .filter(|c| !c.is_control())
                    .ok_or_else(|| error("invalid_entity", entity))?
            }
        };
        result.push(ch);
        rest = &rest[end + 1..];
    }
    result.push_str(rest);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_image_link_and_post_form_without_any_loader() {
        let doc = parse(include_str!("../../../fixtures/prototypes/index.html")).unwrap();
        assert!(
            doc.blocks
                .iter()
                .any(|b| matches!(b, Block::Link { target, .. } if target == "/details"))
        );
        assert!(
            doc.blocks
                .iter()
                .any(|b| matches!(b, Block::Image { source, .. } if source == "/logo.png"))
        );
        assert!(doc.blocks.iter().any(|b| matches!(b, Block::Form { action, fields, .. } if action == "/echo" && fields[0].name == "message")));
    }
    #[test]
    fn rejects_external_resources_scripts_and_malformed_content() {
        for html in [
            "<script>alert(1)</script>",
            "<img src='https://example.org/x' alt='x'>",
            "<a href='file:///a'>x</a>",
            "<form action='//host/x' method='post'></form>",
            "<p onclick='x'>x</p>",
            "<p><b>x</p>",
            "<!DOCTYPE x>",
            "<img src='/a' src='/b' alt='x'>",
            "<p>&unknown;</p>",
        ] {
            assert!(parse(html).is_err(), "{html}");
        }
        assert!(parse(&"<p>".repeat(20)).is_err());
        assert!(parse(&"x".repeat(MAX_BYTES + 1)).is_err());
    }
    #[test]
    fn encoded_external_urls_do_not_bypass_the_policy() {
        assert!(parse("<a href='&#104;ttps://evil.test/'>x</a>").is_err());
        assert!(parse("<img src='/&#46;&#46;/secret' alt='x'>").is_err());
        assert_eq!(
            parse("<p>Réseau &amp; sécurité</p>").unwrap().blocks[0],
            Block::Text {
                text: "Réseau & sécurité".into(),
                heading: false
            }
        );
    }
}
