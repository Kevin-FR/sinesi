//! The same immutable scene drives PDF geometry and selectable text.
use crate::pdf::{Fonts, failed, fill, line, rectangle};
use krilla::{
    geom::{PathBuilder, Size, Transform},
    image::Image,
    paint::Stroke,
    surface::Surface,
};
use sns_model::Diagnostic;
use sns_render::{Bounds, Primitive, Scene};
use std::collections::BTreeMap;
type Symbols = BTreeMap<(bool, u8), Image>;
fn rounded(b: Bounds, r: f32) -> Result<krilla::geom::Path, Diagnostic> {
    if r <= 0. {
        return rectangle(b.x, b.y, b.width, b.height);
    }
    let r = r.min(b.width / 2.).min(b.height / 2.);
    let k = r * 0.5522848;
    let (x, y, right, bottom) = (b.x, b.y, b.x + b.width, b.y + b.height);
    let mut p = PathBuilder::new();
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
    p.finish().ok_or_else(failed)
}
pub(crate) fn draw(
    s: &mut Surface<'_>,
    fonts: &Fonts,
    symbols: &mut Symbols,
    scene: &Scene,
    x: f32,
    y: f32,
    scale: f32,
) -> Result<(), Diagnostic> {
    let b = scene.bounds();
    s.push_transform(&Transform::from_row(
        scale,
        0.,
        0.,
        scale,
        x - b.x * scale,
        y - b.y * scale,
    ));
    // A paper report uses an opaque view background, keeping all source icon colors intact.
    s.set_fill(Some(fill(if scene.diagram() {
        [246, 249, 252, 255]
    } else {
        [28, 33, 42, 255]
    })));
    s.set_stroke(None);
    s.draw_path(&rectangle(b.x, b.y, b.width, b.height)?);
    for item in scene.items() {
        match &item.primitive {
            Primitive::Rect {
                bounds,
                fill: color,
                border,
                border_width,
                radius,
            } => {
                s.set_fill(Some(fill(*color)));
                s.set_stroke(if *border_width > 0. && border[3] > 0 {
                    Some(Stroke {
                        paint: krilla::color::rgb::Color::new(border[0], border[1], border[2])
                            .into(),
                        width: *border_width,
                        ..Default::default()
                    })
                } else {
                    None
                });
                s.draw_path(&rounded(*bounds, *radius)?);
                s.set_stroke(None);
            }
            Primitive::Line {
                points,
                color,
                width,
                dashed,
            } => {
                let [a, b, c, d] = *points;
                let pieces = if *dashed { 12 } else { 1 };
                for piece in 0..pieces {
                    let t = piece as f32 / pieces as f32;
                    let end = if *dashed {
                        (piece as f32 + 0.5) / pieces as f32
                    } else {
                        1.
                    };
                    line(
                        s,
                        [
                            a + (c - a) * t,
                            b + (d - b) * t,
                            a + (c - a) * end,
                            b + (d - b) * end,
                        ],
                        *color,
                        *width,
                    )?;
                }
            }
            Primitive::Text {
                bounds,
                origin,
                layout,
                color,
                italic,
                underline,
            } => {
                s.push_clip_path(
                    &rectangle(bounds.x, bounds.y, bounds.width, bounds.height)?,
                    &Default::default(),
                );
                fonts.draw(
                    s,
                    origin[0],
                    origin[1],
                    layout,
                    crate::pdf::TextStyle {
                        color: *color,
                        italic: *italic,
                        underline: *underline,
                    },
                )?;
                s.pop();
            }
            Primitive::Symbol {
                bounds,
                symbol,
                diagram,
            } => {
                let key = (*diagram, *symbol as u8);
                if let std::collections::btree_map::Entry::Vacant(slot) = symbols.entry(key) {
                    let opt = resvg::usvg::Options {
                        image_href_resolver: resvg::usvg::ImageHrefResolver {
                            resolve_data: Box::new(|_, _, _| None),
                            resolve_string: Box::new(|_, _| None),
                        },
                        ..Default::default()
                    };
                    let tree = resvg::usvg::Tree::from_data(
                        sns_render::symbol_svg(*symbol, *diagram),
                        &opt,
                    )
                    .map_err(|_| failed())?;
                    let mut pixmap = resvg::tiny_skia::Pixmap::new(256, 256).ok_or_else(failed)?;
                    resvg::render(
                        &tree,
                        resvg::tiny_skia::Transform::from_scale(
                            256. / tree.size().width(),
                            256. / tree.size().height(),
                        ),
                        &mut pixmap.as_mut(),
                    );
                    let rgba: Vec<_> = pixmap
                        .pixels()
                        .iter()
                        .flat_map(|p| {
                            let c = p.demultiply();
                            [c.red(), c.green(), c.blue(), c.alpha()]
                        })
                        .collect();
                    slot.insert(Image::from_rgba8(rgba, 256, 256));
                }
                let image = symbols.get(&key).ok_or_else(failed)?.clone();
                s.push_transform(&Transform::from_translate(bounds.x, bounds.y));
                s.draw_image(
                    image,
                    Size::from_wh(bounds.width, bounds.height).ok_or_else(failed)?,
                );
                s.pop();
            }
        }
    }
    s.pop();
    Ok(())
}
