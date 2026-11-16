//! PDF adapter. Glyph shaping and page layout have already happened outside Krilla.
use crate::{Report, content::Text, limit};
use krilla::{
    Document,
    color::rgb,
    geom::{PathBuilder, Point, Rect},
    metadata::Metadata,
    num::NormalizedF32,
    page::PageSettings,
    paint::{Fill, Stroke},
    surface::Surface,
    text::{Font, GlyphId, KrillaGlyph},
};
use sns_fonts::{Face, TextLayout};
use sns_model::Diagnostic;
use std::collections::BTreeMap;

pub struct PdfReport {
    pub bytes: Vec<u8>,
    pub pages: usize,
    pub packet_rows: usize,
    pub interface_groups: usize,
    pub trace_partial: bool,
    pub clipped_scene_texts: usize,
}
pub(crate) fn failed() -> Diagnostic {
    Diagnostic::new("report.pdf_failed", "pdf")
}
pub(crate) fn fill(color: [u8; 4]) -> Fill {
    Fill {
        paint: rgb::Color::new(color[0], color[1], color[2]).into(),
        opacity: NormalizedF32::new(color[3] as f32 / 255.).unwrap_or(NormalizedF32::ONE),
        rule: Default::default(),
    }
}
pub(crate) fn rectangle(x: f32, y: f32, w: f32, h: f32) -> Result<krilla::geom::Path, Diagnostic> {
    let rect = Rect::from_xywh(x, y, w, h).ok_or_else(failed)?;
    let mut pb = PathBuilder::new();
    pb.push_rect(rect);
    pb.finish().ok_or_else(failed)
}
pub(crate) fn line(
    s: &mut Surface<'_>,
    points: [f32; 4],
    color: [u8; 4],
    width: f32,
) -> Result<(), Diagnostic> {
    let mut p = PathBuilder::new();
    p.move_to(points[0], points[1]);
    p.line_to(points[2], points[3]);
    let path = p.finish().ok_or_else(failed)?;
    s.set_fill(None);
    s.set_stroke(Some(Stroke {
        paint: rgb::Color::new(color[0], color[1], color[2]).into(),
        width,
        opacity: NormalizedF32::new(color[3] as f32 / 255.).unwrap_or(NormalizedF32::ONE),
        ..Default::default()
    }));
    s.draw_path(&path);
    s.set_stroke(None);
    Ok(())
}
pub(crate) struct Fonts(BTreeMap<Face, Font>);
pub(crate) struct TextStyle {
    pub color: [u8; 4],
    pub italic: bool,
    pub underline: bool,
}
impl TextStyle {
    pub fn plain(color: [u8; 4]) -> Self {
        Self {
            color,
            italic: false,
            underline: false,
        }
    }
}
impl Fonts {
    fn new() -> Result<Self, Diagnostic> {
        let mut fonts = BTreeMap::new();
        for face in [Face::Sans, Face::SansBold, Face::Mono, Face::MonoBold] {
            fonts.insert(
                face,
                Font::new(face.bytes().to_vec().into(), 0).ok_or_else(failed)?,
            );
        }
        Ok(Self(fonts))
    }
    pub(crate) fn draw(
        &self,
        s: &mut Surface<'_>,
        x: f32,
        y: f32,
        layout: &TextLayout,
        style: TextStyle,
    ) -> Result<(), Diagnostic> {
        let TextStyle {
            color,
            italic,
            underline,
        } = style;
        let font = self.0.get(&layout.face).ok_or_else(failed)?;
        for line in &layout.lines {
            let mut clusters: Vec<_> = line.glyphs.iter().map(|g| g.cluster).collect();
            clusters.push(line.text.len());
            clusters.sort_unstable();
            clusters.dedup();
            let ends: BTreeMap<_, _> = clusters.windows(2).map(|pair| (pair[0], pair[1])).collect();
            let mut cursor = 0.;
            let glyphs: Vec<_> = line
                .glyphs
                .iter()
                .map(|g| {
                    let end = ends.get(&g.cluster).copied().unwrap_or(line.text.len());
                    let glyph = KrillaGlyph::new(
                        GlyphId::new(g.id),
                        g.advance / layout.size,
                        (g.x - cursor) / layout.size,
                        -g.y / layout.size,
                        0.,
                        g.cluster..end,
                        None,
                    );
                    cursor += g.advance;
                    glyph
                })
                .collect();
            s.set_fill(Some(fill(color)));
            s.set_stroke(None);
            s.push_transform(&krilla::geom::Transform::from_translate(
                x,
                y + line.baseline,
            ));
            if italic {
                s.push_transform(&krilla::geom::Transform::from_row(
                    1., 0., -0.22, 1., 0., 0.,
                ));
            }
            s.draw_glyphs(
                Point::from_xy(0., 0.),
                &glyphs,
                font.clone(),
                &line.text,
                layout.size,
                false,
            );
            if italic {
                s.pop();
            }
            s.pop();
            if underline {
                crate::pdf::line(
                    s,
                    [
                        x,
                        y + line.baseline + layout.size * 0.10,
                        x + line.width,
                        y + line.baseline + layout.size * 0.10,
                    ],
                    color,
                    (layout.size * 0.05).max(0.5),
                )?;
            }
        }
        Ok(())
    }
}
pub fn render_pdf(report: &Report) -> Result<PdfReport, Diagnostic> {
    let mut doc = Document::new();
    doc.set_metadata(
        Metadata::new()
            .title(report.title.clone())
            .language(report.options.language.code().into())
            .creator("Sinesi".into())
            .producer("Sinesi / Krilla".into()),
    );
    let fonts = Fonts::new()?;
    let mut symbols = BTreeMap::new();
    for (number, content) in report.pages.iter().enumerate() {
        let settings = PageSettings::from_wh(595.28, 841.89).ok_or_else(failed)?;
        let mut page = doc.start_page_with(settings);
        let mut s = page.surface();
        let header = sns_fonts::layout(&content.section, Face::SansBold, 14., 523., 40., 2, true)
            .map_err(|_| failed())?;
        fonts.draw(
            &mut s,
            36.,
            24.,
            &header,
            TextStyle::plain([31, 83, 122, 255]),
        )?;
        line(&mut s, [36., 62., 559., 62.], [31, 83, 122, 255], 1.)?;
        if let Some((scene, x, y, scale)) = &content.scene {
            crate::scene::draw(&mut s, &fonts, &mut symbols, scene, *x, *y, *scale)?;
        }
        for Text {
            x,
            y,
            layout,
            color,
        } in &content.text
        {
            fonts.draw(&mut s, *x, *y, layout, TextStyle::plain(*color))?;
        }
        for y in &content.rules {
            line(&mut s, [36., *y, 559., *y], [212, 222, 231, 255], 0.4)?;
        }
        let footer = format!(
            "Sinesi · {} · {} / {}",
            report.options.language.text("report_title"),
            number + 1,
            report.pages.len()
        );
        let footer = sns_fonts::layout(&footer, Face::Sans, 8., 523., 20., 1, false)
            .map_err(|_| failed())?;
        fonts.draw(
            &mut s,
            36.,
            807.,
            &footer,
            TextStyle::plain([74, 91, 110, 255]),
        )?;
        s.finish();
        page.finish();
    }
    let bytes = doc.finish().map_err(|_| failed())?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(limit("report_pdf_bytes"));
    }
    Ok(PdfReport {
        bytes,
        pages: report.pages.len(),
        packet_rows: report.packet_rows,
        interface_groups: report.interface_groups,
        trace_partial: report.trace_partial,
        clipped_scene_texts: report.clipped_scene_texts,
    })
}
