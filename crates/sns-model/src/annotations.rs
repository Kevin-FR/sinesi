//! Durable, renderer-independent documentation geometry and text styles.
use crate::*;
pub const DOCUMENTATION_PROFILE: &str = "sns-educational-documentation-v1";
pub const DOCUMENTATION_SCHEMA_VERSION: u32 = 9;
pub const MAX_ANNOTATIONS: usize = 4_096;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationKind {
    #[default]
    Text,
    Rectangle,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnnotationFont {
    #[default]
    Proportional,
    Monospace,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnotationStyle {
    pub kind: AnnotationKind,
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub font: AnnotationFont,
    pub italic: bool,
    #[serde(default)]
    pub bold: bool,
    pub underline: bool,
    /// Unpremultiplied sRGB RGBA, shared by screen and export adapters.
    pub text_color: [u8; 4],
    pub fill_color: [u8; 4],
    pub border_color: [u8; 4],
    pub border_width: f32,
}
impl Default for AnnotationStyle {
    fn default() -> Self {
        Self {
            kind: AnnotationKind::Text,
            width: 240.,
            height: 100.,
            font_size: 16.,
            font: AnnotationFont::Proportional,
            italic: false,
            bold: false,
            underline: false,
            text_color: [15, 45, 65, 255],
            fill_color: [250, 244, 190, 235],
            border_color: [60, 100, 130, 255],
            border_width: 0.,
        }
    }
}
impl AnnotationStyle {
    pub fn for_kind(kind: AnnotationKind) -> Self {
        if kind == AnnotationKind::Rectangle {
            Self {
                kind,
                width: 400.,
                height: 260.,
                fill_color: [130, 200, 235, 210],
                border_width: 2.,
                ..Self::default()
            }
        } else {
            Self::default()
        }
    }
    pub fn validate(&self) -> bool {
        [self.width, self.height]
            .iter()
            .all(|v| v.is_finite() && (24.0..=10_000.).contains(v))
            && self.font_size.is_finite()
            && (8.0..=96.).contains(&self.font_size)
            && self.border_width.is_finite()
            && (0.0..=20.).contains(&self.border_width)
    }
}
impl Annotation {
    pub fn effective_style(&self) -> AnnotationStyle {
        self.style.clone().unwrap_or_else(|| AnnotationStyle {
            font_size: 14.,
            ..AnnotationStyle::default()
        })
    }
}
impl Project {
    /// Upsert in place: editing a note must not silently change its stacking order.
    pub fn with_annotation(&self, annotation: Annotation) -> Result<Self, Diagnostic> {
        let mut next = self.clone();
        if annotation.style.is_some() && next.schema_version < DOCUMENTATION_SCHEMA_VERSION {
            next.schema_version = DOCUMENTATION_SCHEMA_VERSION;
            next.semantic_profile = DOCUMENTATION_PROFILE.into();
        }
        if let Some(old) = next.annotations.iter_mut().find(|a| a.id == annotation.id) {
            *old = annotation;
        } else {
            next.annotations.push(annotation);
        }
        next.validate()?;
        if serde_json::to_vec(&next).map_or(true, |b| b.len() > MAX_PROJECT_DOCUMENT_BYTES) {
            return Err(Diagnostic::new(
                "resource.limit_exceeded",
                "project_document",
            ));
        }
        Ok(next)
    }
}
