//! Bounded LTR document layout. Glyph IDs, offsets and source clusters survive for PDF adapters.
use crate::*;
use harfrust::{ShaperData, UnicodeBuffer};
use skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
    outline::{DrawSettings, OutlinePen},
};
use std::{collections::BTreeMap, sync::OnceLock};
pub const MAX_TEXT_BYTES: usize = 65_536;
pub const MAX_GLYPHS: usize = 200_000;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Face {
    Sans,
    SansBold,
    Mono,
    MonoBold,
}
impl Face {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::Sans => SANS,
            Self::SansBold => SANS_BOLD,
            Self::Mono => MONO,
            Self::MonoBold => MONO_BOLD,
        }
    }
    fn index(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Debug)]
pub struct Glyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub cluster: usize,
    pub advance: f32,
}
#[derive(Clone, Debug)]
pub struct Line {
    pub text: String,
    pub baseline: f32,
    pub width: f32,
    pub glyphs: Vec<Glyph>,
}
#[derive(Clone, Debug)]
pub struct TextLayout {
    pub face: Face,
    pub size: f32,
    pub lines: Vec<Line>,
    pub height: f32,
    pub clipped: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub enum FontError {
    Limit,
    Unsupported(char),
    Invalid,
}
impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Limit => write!(f, "resource.limit_exceeded: document_text"),
            Self::Unsupported(c) => write!(f, "render.unsupported_glyph: U+{:04X}", *c as u32),
            Self::Invalid => write!(f, "render.invalid_font"),
        }
    }
}
impl std::error::Error for FontError {}
fn font(face: Face) -> FontRef<'static> {
    FontRef::new(face.bytes()).expect("verified bundled font")
}
fn data(face: Face) -> &'static ShaperData {
    static DATA: OnceLock<[ShaperData; 4]> = OnceLock::new();
    &DATA.get_or_init(|| {
        [Face::Sans, Face::SansBold, Face::Mono, Face::MonoBold].map(|f| ShaperData::new(&font(f)))
    })[face.index()]
}
fn shape(text: &str, face: Face, size: f32) -> Result<(Vec<Glyph>, f32), FontError> {
    let font = font(face);
    let metrics = font.metrics(Size::unscaled(), LocationRef::default());
    let scale = size / metrics.units_per_em as f32;
    let shaper = data(face).shaper(&font).build();
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.guess_segment_properties();
    // This profile covers the six LTR UI languages. A bidi paragraph is never silently reversed.
    if buffer.direction() != harfrust::Direction::LeftToRight {
        return Err(FontError::Unsupported(
            text.chars().find(|c| !c.is_whitespace()).unwrap_or(' '),
        ));
    }
    let output = shaper.shape(buffer, harfrust::ShapeOptions::new());
    let mut x = 0.;
    let mut glyphs = Vec::with_capacity(output.len());
    for (info, position) in output.glyph_infos().iter().zip(output.glyph_positions()) {
        if info.glyph_id == 0 {
            return Err(FontError::Unsupported(
                text[info.cluster as usize..]
                    .chars()
                    .next()
                    .unwrap_or('\u{FFFD}'),
            ));
        }
        glyphs.push(Glyph {
            id: info.glyph_id,
            x: x + position.x_offset as f32 * scale,
            y: -(position.y_offset as f32) * scale,
            cluster: info.cluster as usize,
            advance: position.x_advance as f32 * scale,
        });
        x += position.x_advance as f32 * scale;
    }
    Ok((glyphs, x))
}
/// Soft wraps at whitespace/hyphens; long words split only at shaping-cluster boundaries.
/// Runs and explicit blank lines remain bounded by the visible box and a global glyph budget.
pub fn layout(
    text: &str,
    face: Face,
    size: f32,
    width: f32,
    height: f32,
    max_rows: usize,
    ellipsis: bool,
) -> Result<TextLayout, FontError> {
    if text.len() > MAX_TEXT_BYTES
        || !size.is_finite()
        || !(1.0..=256.).contains(&size)
        || !width.is_finite()
        || width <= 0.
        || !height.is_finite()
        || height <= 0.
        || max_rows == 0
    {
        return Err(FontError::Limit);
    }
    let metrics = font(face).metrics(Size::new(size), LocationRef::default());
    let step = (metrics.ascent - metrics.descent + metrics.leading).max(size);
    let capacity = max_rows
        .min((height / step).floor().max(1.) as usize)
        .min(4096);
    let mut result = TextLayout {
        face,
        size,
        lines: Vec::new(),
        height: 0.,
        clipped: false,
    };
    let normalized = text
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .replace('\t', "    ");
    if normalized.len() > MAX_TEXT_BYTES {
        return Err(FontError::Limit);
    }
    let mut glyph_count = 0;
    let mut horizontal_overflow = false;
    for paragraph in normalized.split('\n') {
        if result.lines.len() >= capacity {
            result.clipped = true;
            break;
        }
        let (full, total) = shape(paragraph, face, size)?;
        if paragraph.is_empty() {
            result.lines.push(Line {
                text: String::new(),
                baseline: metrics.ascent + result.lines.len() as f32 * step,
                width: 0.,
                glyphs: Vec::new(),
            });
            continue;
        }
        let mut widths = BTreeMap::new();
        for g in &full {
            *widths.entry(g.cluster).or_insert(0.) += g.advance;
        }
        let mut offsets = BTreeMap::new();
        let mut cursor = 0.;
        for (cluster, advance) in widths {
            offsets.insert(cluster, cursor);
            cursor += advance;
        }
        offsets.entry(0).or_insert(0.);
        offsets.insert(paragraph.len(), total);
        let clusters: Vec<_> = offsets.into_iter().collect();
        let mut start = 0;
        while start + 1 < clusters.len() {
            if result.lines.len() >= capacity {
                result.clipped = true;
                break;
            }
            let mut end = start + 1;
            let mut last_break = None;
            while end + 1 < clusters.len() && clusters[end + 1].1 - clusters[start].1 <= width {
                let part = &paragraph[clusters[end - 1].0..clusters[end].0];
                if part.chars().all(char::is_whitespace) || part.ends_with(['-', '\u{2010}']) {
                    last_break = Some(end);
                }
                end += 1;
            }
            if end + 1 < clusters.len()
                && let Some(word_end) = last_break
            {
                end = word_end;
            }
            let raw = &paragraph[clusters[start].0..clusters[end].0];
            let line = raw.trim_end_matches(char::is_whitespace);
            let (mut glyphs, mut measured) = shape(line, face, size)?;
            // Re-shaping a line can change boundary kerning. Never split a UTF-8 cluster.
            while measured > width && end > start + 1 {
                end -= 1;
                let candidate = &paragraph[clusters[start].0..clusters[end].0];
                (glyphs, measured) =
                    shape(candidate.trim_end_matches(char::is_whitespace), face, size)?;
            }
            let content = paragraph[clusters[start].0..clusters[end].0]
                .trim_end_matches(char::is_whitespace)
                .to_owned();
            horizontal_overflow |= measured > width;
            glyph_count += glyphs.len();
            if glyph_count > MAX_GLYPHS {
                return Err(FontError::Limit);
            }
            result.lines.push(Line {
                text: content,
                baseline: metrics.ascent + result.lines.len() as f32 * step,
                width: measured,
                glyphs,
            });
            start = end;
            while start + 1 < clusters.len()
                && paragraph[clusters[start].0..clusters[start + 1].0]
                    .chars()
                    .all(char::is_whitespace)
            {
                start += 1;
            }
        }
        if result.clipped {
            break;
        }
    }
    result.clipped |= result.lines.len() as f32 * step > height || horizontal_overflow;
    result.clipped &= !normalized.trim().is_empty();
    if result.clipped
        && ellipsis
        && let Some(last) = result.lines.last_mut()
    {
        let mut cut = last.text.len();
        loop {
            let candidate = format!("{}…", &last.text[..cut]);
            let (glyphs, measured) = shape(&candidate, face, size)?;
            if measured <= width || cut == 0 {
                last.text = candidate;
                last.glyphs = glyphs;
                last.width = measured;
                break;
            }
            cut = last.text[..cut]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }
    result.height = result.lines.len() as f32 * step;
    Ok(result)
}
#[derive(Clone, Debug)]
pub enum PathVerb {
    Move(f32, f32),
    Line(f32, f32),
    Quad(f32, f32, f32, f32),
    Cubic(f32, f32, f32, f32, f32, f32),
    Close,
}
struct Pen(Vec<PathVerb>);
impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.push(PathVerb::Move(x, y));
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.0.push(PathVerb::Line(x, y));
    }
    fn quad_to(&mut self, x: f32, y: f32, a: f32, b: f32) {
        self.0.push(PathVerb::Quad(x, y, a, b));
    }
    fn curve_to(&mut self, x: f32, y: f32, a: f32, b: f32, c: f32, d: f32) {
        self.0.push(PathVerb::Cubic(x, y, a, b, c, d));
    }
    fn close(&mut self) {
        self.0.push(PathVerb::Close);
    }
}
pub fn outline(face: Face, glyph: u32, size: f32) -> Result<Vec<PathVerb>, FontError> {
    let font = font(face);
    let outlines = font.outline_glyphs();
    let glyph = outlines
        .get(skrifa::GlyphId::new(glyph))
        .ok_or(FontError::Invalid)?;
    let mut pen = Pen(Vec::new());
    glyph
        .draw(
            DrawSettings::unhinted(Size::new(size), LocationRef::default()),
            &mut pen,
        )
        .map_err(|_| FontError::Invalid)?;
    Ok(pen.0)
}
