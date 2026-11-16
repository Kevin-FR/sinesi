//! Six explicit translation catalogues, independent of projects, clocks and GUI.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, sync::OnceLock};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Fr,
    En,
    Es,
    It,
    Pt,
    De,
}
impl Language {
    pub const ALL: [Self; 6] = [Self::Fr, Self::En, Self::Es, Self::It, Self::Pt, Self::De];
    pub fn code(self) -> &'static str {
        match self {
            Self::Fr => "fr",
            Self::En => "en",
            Self::Es => "es",
            Self::It => "it",
            Self::Pt => "pt",
            Self::De => "de",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Fr => "Français",
            Self::En => "English",
            Self::Es => "Español",
            Self::It => "Italiano",
            Self::Pt => "Português",
            Self::De => "Deutsch",
        }
    }
    pub fn try_text(self, key: &str) -> Option<&'static str> {
        catalogue()
            .get(key)
            .and_then(|m| m.get(self.code()))
            .map(String::as_str)
    }
    pub fn describe_error(self, error: &str) -> String {
        let code = error.split(':').next().unwrap_or(error);
        self.try_text(code)
            .map(|text| format!("{text} [{code}]"))
            .unwrap_or_else(|| error.to_owned())
    }
    /// Missing keys are development errors; never silently fall back to another language.
    pub fn text(self, key: &str) -> &'static str {
        catalogue()
            .get(key)
            .and_then(|m| m.get(self.code()))
            .map(String::as_str)
            .expect("unknown translation key or missing language")
    }
    pub fn format(self, key: &str, params: &[(&str, &str)]) -> Result<String, String> {
        let mut tail = self.text(key);
        let mut out = String::new();
        while let Some(start) = tail.find('{') {
            out.push_str(&tail[..start]);
            let end = tail[start + 1..]
                .find('}')
                .ok_or_else(|| format!("i18n.invalid_template:{key}"))?
                + start
                + 1;
            let name = &tail[start + 1..end];
            let value = params
                .iter()
                .find(|(k, _)| *k == name)
                .map(|(_, v)| *v)
                .ok_or_else(|| format!("i18n.missing_parameter:{key}:{name}"))?;
            out.push_str(value);
            tail = &tail[end + 1..];
        }
        out.push_str(tail);
        Ok(out)
    }
}
fn catalogue() -> &'static BTreeMap<String, BTreeMap<String, String>> {
    static C: OnceLock<BTreeMap<String, BTreeMap<String, String>>> = OnceLock::new();
    C.get_or_init(|| {
        serde_json::from_str(include_str!("../locales/messages.json"))
            .expect("embedded translation catalogue")
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    fn placeholders(text: &str) -> BTreeSet<&str> {
        text.split('{')
            .skip(1)
            .map(|s| s.split('}').next().unwrap())
            .collect()
    }
    #[test]
    fn all_six_languages_have_every_key_and_matching_parameters() {
        assert!(catalogue().len() > 100);
        for (key, entries) in catalogue() {
            assert_eq!(entries.len(), 6, "{key}");
            let expected = placeholders(&entries["fr"]);
            for lang in Language::ALL {
                let text = lang.text(key);
                assert!(!text.trim().is_empty(), "{}:{key}", lang.code());
                assert_eq!(placeholders(text), expected, "{}:{key}", lang.code());
            }
        }
    }
    #[test]
    fn language_and_formatted_values_are_portable_and_no_fallback_occurs() {
        for lang in Language::ALL {
            let s = serde_json::to_string(&lang).unwrap();
            assert_eq!(serde_json::from_str::<Language>(&s).unwrap(), lang);
            let text = lang
                .format("ping_reply", &[("source", "192.168.1.3"), ("rtt", "8000")])
                .unwrap();
            assert!(text.contains("192.168.1.3"));
            assert!(text.contains("8000"));
            assert!(lang.format("ping_reply", &[]).is_err());
        }
        assert!(serde_json::from_str::<Language>("\"xx\"").is_err());
        assert_ne!(Language::Fr.text("save"), Language::De.text("save"));
    }
}
