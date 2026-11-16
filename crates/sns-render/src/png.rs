//! Rasterize a frozen scene; this module returns bytes and never opens a destination.
use super::*;
use resvg::{tiny_skia as sk, usvg};
use sns_fonts::{Face, PathVerb};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PngOptions {
    pub longest_side: u32,
    pub transparent: bool,
}
impl Default for PngOptions {
    fn default() -> Self {
        Self {
            longest_side: 2400,
            transparent: true,
        }
    }
}
pub struct PngImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
}
impl PngOptions {
    pub fn dimensions(self, scene: &Scene) -> Result<(u32, u32), Diagnostic> {
        if !(256..=4096).contains(&self.longest_side) {
            return Err(limit("png_dimensions"));
        }
        let bounds = scene.bounds();
        let scale = (self.longest_side - 32) as f32 / bounds.width.max(bounds.height);
        Ok((
            (bounds.width * scale).round().max(1.) as u32 + 32,
            (bounds.height * scale).round().max(1.) as u32 + 32,
        ))
    }
}
fn ink(color: Color) -> sk::Paint<'static> {
    let mut p = sk::Paint::default();
    p.set_color_rgba8(color[0], color[1], color[2], color[3]);
    p
}
fn rect(bounds: Bounds) -> Result<sk::Rect, Diagnostic> {
    sk::Rect::from_xywh(bounds.x, bounds.y, bounds.width, bounds.height)
        .ok_or_else(|| limit("scene_geometry"))
}
fn rounded(b: Bounds, radius: f32) -> Result<sk::Path, Diagnostic> {
    if radius == 0. {
        return Ok(sk::PathBuilder::from_rect(rect(b)?));
    }
    let r = radius.min(b.width / 2.).min(b.height / 2.);
    let k = r * 0.5522848;
    let x = b.x;
    let y = b.y;
    let right = x + b.width;
    let bottom = y + b.height;
    let mut p = sk::PathBuilder::new();
    p.move_to(x + r, y);
    p.line_to(right - r, y);
    p.cubic_to(right - r + k, y, right, y + r - k, right, y + r);
    p.line_to(right, bottom - r);
    p.cubic_to(
        right,
        bottom - r + k,
        right - r + k,
        bottom,
        right - r,
        bottom,
    );
    p.line_to(x + r, bottom);
    p.cubic_to(x + r - k, bottom, x, bottom - r + k, x, bottom - r);
    p.line_to(x, y + r);
    p.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    p.close();
    p.finish().ok_or_else(|| limit("scene_path"))
}
pub fn render_png(scene: &Scene, options: PngOptions) -> Result<PngImage, Diagnostic> {
    let (width, height) = options.dimensions(scene)?;
    let original = scene.bounds();
    let scale = (options.longest_side - 32) as f32 / original.width.max(original.height);
    // Keep a real pixel margin even for legal projects spanning millions of world units.
    let b = original.inset(-16. / scale);
    let world = sk::Transform::from_row(scale, 0., 0., scale, -b.x * scale, -b.y * scale);
    let mut image = sk::Pixmap::new(width, height).ok_or_else(|| limit("png_allocation"))?;
    if !options.transparent {
        image.fill(if scene.diagram() {
            sk::Color::from_rgba8(246, 249, 251, 255)
        } else {
            sk::Color::from_rgba8(20, 28, 38, 255)
        });
    }
    let mut symbols = BTreeMap::new();
    let mut glyphs = BTreeMap::new();
    let mut path_verbs = 0;
    for item in scene.items() {
        match &item.primitive {
            Primitive::Rect {
                bounds,
                fill,
                border,
                border_width,
                radius,
            } => {
                let path = rounded(*bounds, *radius)?;
                image.fill_path(&path, &ink(*fill), sk::FillRule::Winding, world, None);
                if *border_width > 0. && border[3] > 0 {
                    let path = rounded(
                        bounds.inset(*border_width / 2.),
                        (*radius - *border_width / 2.).max(0.),
                    )?;
                    image.stroke_path(
                        &path,
                        &ink(*border),
                        &sk::Stroke {
                            width: *border_width,
                            ..Default::default()
                        },
                        world,
                        None,
                    );
                }
            }
            Primitive::Line {
                points,
                color,
                width,
                dashed,
            } => {
                let [x, y, end_x, end_y] = *points;
                let pieces = if *dashed { 12 } else { 1 };
                for piece in 0..pieces {
                    let start = piece as f32 / pieces as f32;
                    let end = if *dashed {
                        (piece as f32 + 0.55) / pieces as f32
                    } else {
                        1.
                    };
                    let mut p = sk::PathBuilder::new();
                    p.move_to(x + (end_x - x) * start, y + (end_y - y) * start);
                    p.line_to(x + (end_x - x) * end, y + (end_y - y) * end);
                    if let Some(path) = p.finish() {
                        image.stroke_path(
                            &path,
                            &ink(*color),
                            &sk::Stroke {
                                width: *width,
                                ..Default::default()
                            },
                            world,
                            None,
                        );
                    }
                }
            }
            Primitive::Symbol {
                bounds,
                symbol,
                diagram,
            } => {
                let key = (*diagram, *symbol as u8);
                if let std::collections::btree_map::Entry::Vacant(slot) = symbols.entry(key) {
                    let opt = usvg::Options {
                        image_href_resolver: usvg::ImageHrefResolver {
                            resolve_data: Box::new(|_, _, _| None),
                            resolve_string: Box::new(|_, _| None),
                        },
                        ..Default::default()
                    };
                    let tree = usvg::Tree::from_data(symbol_svg(*symbol, *diagram), &opt)
                        .map_err(|_| Diagnostic::new("render.invalid_asset", "embedded_symbol"))?;
                    slot.insert(tree);
                }
                let tree = &symbols[&key];
                let transform = sk::Transform::from_row(
                    bounds.width / tree.size().width() * scale,
                    0.,
                    0.,
                    bounds.height / tree.size().height() * scale,
                    (bounds.x - b.x) * scale,
                    (bounds.y - b.y) * scale,
                );
                resvg::render(tree, transform, &mut image.as_mut());
            }
            Primitive::Text {
                bounds,
                origin,
                layout,
                color,
                italic,
                underline,
            } => {
                paint_text(
                    &mut image,
                    *bounds,
                    *origin,
                    layout,
                    *color,
                    *italic,
                    *underline,
                    b,
                    scale,
                    &mut glyphs,
                    &mut path_verbs,
                )?;
            }
        }
    }
    let bytes = image
        .encode_png()
        .map_err(|_| Diagnostic::new("render.encode_failed", "png"))?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err(limit("png_document"));
    }
    Ok(PngImage {
        bytes,
        width,
        height,
    })
}
#[allow(clippy::too_many_arguments)]
fn paint_text(
    image: &mut sk::Pixmap,
    bounds: Bounds,
    origin: [f32; 2],
    layout: &TextLayout,
    color: Color,
    italic: bool,
    underline: bool,
    scene: Bounds,
    scale: f32,
    cache: &mut BTreeMap<(Face, u32), Option<sk::Path>>,
    verbs: &mut usize,
) -> Result<(), Diagnostic> {
    let px = (bounds.x - scene.x) * scale;
    let py = (bounds.y - scene.y) * scale;
    let origin_x = (origin[0] - scene.x) * scale;
    let origin_y = (origin[1] - scene.y) * scale;
    let x = px.floor().clamp(0., image.width() as f32) as u32;
    let y = py.floor().clamp(0., image.height() as f32) as u32;
    let end_x = (px + bounds.width * scale)
        .ceil()
        .clamp(0., image.width() as f32) as u32;
    let end_y = (py + bounds.height * scale)
        .ceil()
        .clamp(0., image.height() as f32) as u32;
    if end_x <= x || end_y <= y || color[3] == 0 {
        return Ok(());
    }
    let mut tile = sk::Pixmap::new(end_x - x, end_y - y).ok_or_else(|| limit("text_allocation"))?;
    let mut mask = sk::Mask::new(end_x - x, end_y - y).ok_or_else(|| limit("text_mask"))?;
    mask.fill_path(
        &sk::PathBuilder::from_rect(rect(Bounds::new(
            px - x as f32,
            py - y as f32,
            bounds.width * scale,
            bounds.height * scale,
        ))?),
        sk::FillRule::Winding,
        true,
        sk::Transform::identity(),
    );
    let paint = ink(color);
    for line in &layout.lines {
        for glyph in &line.glyphs {
            let key = (layout.face, glyph.id);
            if let std::collections::btree_map::Entry::Vacant(slot) = cache.entry(key) {
                let outline = sns_fonts::outline(layout.face, glyph.id, 1.)
                    .map_err(|_| Diagnostic::new("render.invalid_font", "glyph_outline"))?;
                *verbs += outline.len();
                if *verbs > 2_000_000 {
                    return Err(limit("scene_path_verbs"));
                }
                let mut path = sk::PathBuilder::new();
                for verb in outline {
                    match verb {
                        PathVerb::Move(x, y) => path.move_to(x, y),
                        PathVerb::Line(x, y) => path.line_to(x, y),
                        PathVerb::Quad(x, y, a, b) => path.quad_to(x, y, a, b),
                        PathVerb::Cubic(x, y, a, b, c, d) => path.cubic_to(x, y, a, b, c, d),
                        PathVerb::Close => path.close(),
                    }
                }
                slot.insert(path.finish());
            }
            if let Some(path) = &cache[&key] {
                let s = layout.size * scale;
                let transform = sk::Transform::from_row(
                    s,
                    0.,
                    if italic { s * 0.21256 } else { 0. },
                    -s,
                    origin_x + glyph.x * scale - x as f32,
                    origin_y + (line.baseline + glyph.y) * scale - y as f32,
                );
                tile.fill_path(path, &paint, sk::FillRule::Winding, transform, Some(&mask));
            }
        }
        if underline && line.width > 0. {
            let mut p = sk::PathBuilder::new();
            let y_line = origin_y + (line.baseline + layout.size * 0.12) * scale - y as f32;
            p.move_to(origin_x - x as f32, y_line);
            p.line_to(origin_x + line.width * scale - x as f32, y_line);
            if let Some(path) = p.finish() {
                tile.stroke_path(
                    &path,
                    &paint,
                    &sk::Stroke {
                        width: (layout.size * scale * 0.06).max(1.),
                        ..Default::default()
                    },
                    sk::Transform::identity(),
                    Some(&mask),
                );
            }
        }
    }
    image.draw_pixmap(
        x as i32,
        y as i32,
        tile.as_ref(),
        &sk::PixmapPaint::default(),
        sk::Transform::identity(),
        None,
    );
    Ok(())
}
