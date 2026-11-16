//! Read-only structural access to an allowlisted subset of JavaBeans XML syntax.
use super::*;
use roxmltree::{Document, Node, NodeId, ParsingOptions};
pub(super) struct Source<'a> {
    pub doc: Document<'a>,
    refs: BTreeMap<String, NodeId>,
    used: std::cell::RefCell<BTreeMap<u32, FieldConversion>>,
}
impl<'a> Source<'a> {
    pub fn parse(text: &'a str) -> Result<Self, Diagnostic> {
        if text.to_ascii_uppercase().contains("<!DOCTYPE")
            || text.to_ascii_uppercase().contains("<!ENTITY")
        {
            return Err(error("import.unsafe_xml", "declaration"));
        }
        let doc = Document::parse_with_options(
            text,
            ParsingOptions {
                allow_dtd: false,
                nodes_limit: 200_000,
            },
        )
        .map_err(|_| error("import.invalid_xml", "syntax"))?;
        if doc.root_element().tag_name().name() != "java" {
            return Err(error("import.invalid_xml", "root"));
        }
        let mut refs = BTreeMap::new();
        for n in doc.descendants().filter(Node::is_element) {
            if n.ancestors().count() > 128 {
                return Err(error("resource.limit_exceeded", "xml_depth"));
            }
            if let Some(name) = n.attribute("id")
                && refs.insert(name.to_string(), n.id()).is_some()
            {
                return Err(error("import.invalid_reference", name));
            }
        }
        for n in doc.descendants() {
            if let Some(name) = n.attribute("idref")
                && !refs.contains_key(name)
            {
                return Err(error("import.invalid_reference", name));
            }
        }
        Ok(Self {
            doc,
            refs,
            used: Default::default(),
        })
    }
    pub fn view<'s>(&'s self, mut n: Node<'s, 'a>) -> Result<Node<'s, 'a>, Diagnostic> {
        let mut seen = BTreeSet::new();
        for _ in 0..128 {
            if !seen.insert(n.id().get()) {
                return Err(error("import.invalid_reference", "alias_cycle"));
            }
            if let Some(r) = n.attribute("idref") {
                n = self.doc.get_node(self.refs[r]).unwrap();
                continue;
            }
            let children = n.children().filter(Node::is_element).collect::<Vec<_>>();
            if matches!(n.tag_name().name(), "void")
                && children.len() == 1
                && children[0].has_tag_name("object")
            {
                n = children[0];
                continue;
            }
            return Ok(n);
        }
        Err(error("resource.limit_exceeded", "alias_depth"))
    }
    pub fn prop<'s>(
        &'s self,
        n: Node<'s, 'a>,
        name: &str,
        target: &str,
    ) -> Result<Option<Node<'s, 'a>>, Diagnostic> {
        let value = self.property(n, name)?;
        if let Some(v) = value {
            self.note(v, &format!("{target}.{name}"));
        }
        value.map(|v| self.view(v)).transpose()
    }
    pub fn property<'s>(
        &'s self,
        n: Node<'s, 'a>,
        name: &str,
    ) -> Result<Option<Node<'s, 'a>>, Diagnostic> {
        Ok(self
            .view(n)?
            .children()
            .find(|c| c.attribute("property") == Some(name)))
    }
    pub fn note(&self, node: Node<'_, 'a>, target: &str) {
        self.used.borrow_mut().insert(
            node.id().get(),
            FieldConversion {
                source: self.location(node),
                target: target.into(),
                rule: "known_property".into(),
            },
        );
    }
    pub fn value(&self, n: Node<'_, 'a>) -> Result<Option<String>, Diagnostic> {
        let n = self.view(n)?;
        if n.attribute("method") == Some("isEnabled") {
            return Ok(Some("true".into()));
        }
        let value = if matches!(
            n.tag_name().name(),
            "string" | "short" | "int" | "boolean" | "double" | "float" | "long"
        ) {
            Some(n)
        } else {
            n.children().find(|c| {
                matches!(
                    c.tag_name().name(),
                    "string" | "short" | "int" | "boolean" | "double" | "float" | "long"
                )
            })
        };
        Ok(value.map(|v| v.text().unwrap_or("").to_string()))
    }
    pub fn string(
        &self,
        n: Node<'_, 'a>,
        name: &str,
        target: &str,
    ) -> Result<Option<String>, Diagnostic> {
        self.prop(n, name, target)?
            .map(|v| self.value(v))
            .transpose()
            .map(Option::flatten)
    }
    pub fn location(&self, n: Node<'_, 'a>) -> String {
        format!(
            "xml@{}:{}",
            n.range().start,
            n.attribute("property")
                .or(n.attribute("method"))
                .or(n.attribute("class"))
                .unwrap_or(n.tag_name().name())
        )
    }
    pub fn converted(&self) -> Vec<FieldConversion> {
        self.used.borrow().values().cloned().collect()
    }
    pub fn unconverted(&self) -> Vec<String> {
        self.doc
            .descendants()
            .filter(|n| {
                (n.attribute("property").is_some()
                    && !self.used.borrow().contains_key(&n.id().get()))
                    || n.attribute("method").is_some_and(|m| {
                        !matches!(
                            m,
                            "add"
                                | "put"
                                | "get"
                                | "set"
                                | "getField"
                                | "isEnabled"
                                | "addMouseListener"
                        )
                    })
            })
            .map(|n| self.location(n))
            .collect()
    }
    pub fn unknown_classes(&self) -> Vec<String> {
        let known: serde_json::Value =
            serde_json::from_str(include_str!("../../../contracts/filius-known-classes.json"))
                .unwrap();
        let known = known["classes"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<BTreeSet<_>>();
        self.doc
            .descendants()
            .filter_map(|n| n.attribute("class"))
            .filter(|c| !known.contains(c))
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}
