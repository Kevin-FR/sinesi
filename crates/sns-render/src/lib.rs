//! Immutable document scene and CPU raster adapter. No widgets, OS font lookup or network I/O.
mod assets;
mod png;
pub use assets::symbol_svg;
pub use png::{PngImage, PngOptions, render_png};
use sns_fonts::{Face, TextLayout};
use sns_model::*;
use std::collections::{BTreeMap, BTreeSet};
pub type Color = [u8; 4];
struct TextPaint {
    color: Color,
    italic: bool,
    underline: bool,
}
const MAX_TEXT_BYTES: usize = 1_048_576;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
    fn union(self, other: Self) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        Self::new(
            x,
            y,
            (self.x + self.width).max(other.x + other.width) - x,
            (self.y + self.height).max(other.y + other.height) - y,
        )
    }
    fn inset(self, margin: f32) -> Self {
        Self::new(
            self.x + margin,
            self.y + margin,
            (self.width - margin * 2.).max(1.),
            (self.height - margin * 2.).max(1.),
        )
    }
}
#[derive(Clone, Debug)]
pub enum Primitive {
    Rect {
        bounds: Bounds,
        fill: Color,
        border: Color,
        border_width: f32,
        radius: f32,
    },
    Line {
        points: [f32; 4],
        color: Color,
        width: f32,
        dashed: bool,
    },
    Symbol {
        bounds: Bounds,
        symbol: DeviceSymbol,
        diagram: bool,
    },
    Text {
        bounds: Bounds,
        origin: [f32; 2],
        layout: TextLayout,
        color: Color,
        italic: bool,
        underline: bool,
    },
}
#[derive(Clone, Debug)]
pub struct Item {
    pub source: Id,
    pub primitive: Primitive,
}
#[derive(Clone, Debug, Default)]
pub struct Counts {
    pub devices: usize,
    pub links: usize,
    pub annotations: usize,
    pub wireless_links: usize,
    pub glyphs: usize,
    pub clipped_texts: usize,
}
#[derive(Clone, Debug)]
pub struct Scene {
    bounds: Bounds,
    items: Vec<Item>,
    counts: Counts,
    objects: BTreeSet<Id>,
    diagram: bool,
}
impl Scene {
    pub fn bounds(&self) -> Bounds {
        self.bounds
    }
    pub fn items(&self) -> &[Item] {
        &self.items
    }
    pub fn counts(&self) -> &Counts {
        &self.counts
    }
    pub fn object_ids(&self) -> &BTreeSet<Id> {
        &self.objects
    }
    pub fn diagram(&self) -> bool {
        self.diagram
    }
    pub fn build(project: &Project, diagram: bool) -> Result<Self, Diagnostic> {
        Self::build_with_labels(project, diagram, &BTreeMap::new())
    }
    /// A client may freeze effective display labels (for example a DHCP lease),
    /// without baking those transient addresses into the persistent project.
    pub fn build_with_labels(
        project: &Project,
        diagram: bool,
        labels: &BTreeMap<Id, String>,
    ) -> Result<Self, Diagnostic> {
        project.validate()?;
        for (id, label) in labels {
            if project.device(id).is_none() {
                return Err(Diagnostic::new("project.invalid_reference", id.as_str()));
            }
            if label.len() > sns_fonts::MAX_TEXT_BYTES {
                return Err(limit("scene_labels"));
            }
        }
        let mut scene = Self {
            bounds: Bounds::new(0., 0., 0., 0.),
            items: Vec::new(),
            counts: Counts {
                devices: project.devices.len(),
                links: project.links.len(),
                annotations: project.annotations.len(),
                ..Default::default()
            },
            objects: BTreeSet::new(),
            diagram,
        };
        let mut extent = None;
        let mut include = |b: Bounds| {
            extent = Some(extent.map_or(b, |old: Bounds| old.union(b)));
        };
        let mut text_bytes = 0;
        for note in &project.annotations {
            let style = note.effective_style();
            let bounds = Bounds::new(note.x, note.y, style.width, style.height);
            include(bounds);
            scene.objects.insert(note.id.clone());
            if note.style.is_some() {
                scene.push(
                    &note.id,
                    Primitive::Rect {
                        bounds,
                        fill: style.fill_color,
                        border: style.border_color,
                        border_width: style.border_width,
                        radius: 0.,
                    },
                );
            }
            text_bytes += note.text.len();
            if text_bytes > MAX_TEXT_BYTES {
                return Err(limit("scene_text"));
            }
            let inner = if note.style.is_some() {
                bounds.inset(6.)
            } else {
                bounds
            };
            let face = match (style.font, style.bold) {
                (AnnotationFont::Monospace, true) => Face::MonoBold,
                (AnnotationFont::Monospace, false) => Face::Mono,
                (AnnotationFont::Proportional, true) => Face::SansBold,
                _ => Face::Sans,
            };
            let layout = text(&note.text, face, style.font_size, inner, 1024, false)?;
            scene.text(
                &note.id,
                bounds,
                [inner.x, inner.y],
                layout,
                TextPaint {
                    color: if note.style.is_some() {
                        style.text_color
                    } else if diagram {
                        [0, 0, 139, 255]
                    } else {
                        [255, 255, 128, 255]
                    },
                    italic: style.italic,
                    underline: style.underline,
                },
            )?;
        }
        let positions: BTreeMap<_, _> = sns_topology::complete_positions(project)
            .into_iter()
            .map(|p| (p.device_id, (p.x, p.y)))
            .collect();
        let contrast = project
            .annotations
            .iter()
            .any(|a| a.style.as_ref().is_some_and(|s| s.fill_color[3] > 0));
        let cable = if diagram {
            [40, 99, 140, 255]
        } else {
            [98, 184, 183, 255]
        };
        for link in &project.links {
            let a = project
                .device_for_interface(&link.endpoints[0])
                .and_then(|d| positions.get(&d.id));
            let b = project
                .device_for_interface(&link.endpoints[1])
                .and_then(|d| positions.get(&d.id));
            if let (Some(a), Some(b)) = (a, b) {
                scene.objects.insert(link.id.clone());
                scene.link(&link.id, [a.0, a.1, b.0, b.1], cable, false, contrast);
            }
        }
        for (client, ap) in project.wireless_links() {
            let a = project
                .device_for_interface(&client)
                .and_then(|d| positions.get(&d.id));
            let b = project
                .device_for_interface(&ap)
                .and_then(|d| positions.get(&d.id));
            if let (Some(a), Some(b)) = (a, b) {
                scene.counts.wireless_links += 1;
                scene.link(
                    &client,
                    [a.0, a.1, b.0, b.1],
                    [32, 150, 215, 255],
                    true,
                    contrast,
                );
            }
        }
        for device in &project.devices {
            let (x, y) = positions[&device.id];
            let bounds = Bounds::new(x - 60., y - 41., 120., 82.);
            include(bounds);
            scene.objects.insert(device.id.clone());
            if !diagram {
                scene.push(
                    &device.id,
                    Primitive::Rect {
                        bounds,
                        fill: [40, 54, 71, 255],
                        border: [128, 128, 128, 255],
                        border_width: 1.,
                        radius: 6.,
                    },
                );
            }
            let symbol = project
                .lan
                .get(&device.id)
                .and_then(|c| c.symbol)
                .unwrap_or(if device.network.ssid.is_some() {
                    DeviceSymbol::AccessPoint
                } else {
                    DeviceSymbol::automatic(device.kind)
                });
            scene.push(
                &device.id,
                Primitive::Symbol {
                    bounds: Bounds::new(x - 26., y - 36., 52., 52.),
                    symbol,
                    diagram,
                },
            );
            let label = labels
                .get(&device.id)
                .cloned()
                .unwrap_or_else(|| sns_topology::display_label(project, device));
            text_bytes += label.len();
            if text_bytes > MAX_TEXT_BYTES {
                return Err(limit("scene_text"));
            }
            let layout = text(
                &label,
                Face::Sans,
                12.,
                Bounds::new(0., 0., 110., 24.),
                1,
                true,
            )?;
            let width = layout.lines.first().map(|l| l.width).unwrap_or(0.);
            let inside = diagram
                && matches!(symbol, DeviceSymbol::Router | DeviceSymbol::Switch)
                && width <= 40.;
            let height = layout.height.min(24.);
            let caption = Bounds::new(
                x - width / 2.,
                y + if inside { -10. } else { 27. } - height / 2.,
                width.max(1.),
                height,
            );
            if diagram && !inside {
                scene.push(
                    &device.id,
                    Primitive::Rect {
                        bounds: caption.inset(-2.),
                        fill: [246, 249, 251, 255],
                        border: [0, 0, 0, 0],
                        border_width: 0.,
                        radius: 2.,
                    },
                );
            }
            scene.text(
                &device.id,
                caption.inset(-2.),
                [caption.x, caption.y],
                layout,
                TextPaint {
                    color: if diagram {
                        [0, 0, 0, 255]
                    } else {
                        [255, 255, 255, 255]
                    },
                    italic: false,
                    underline: false,
                },
            )?;
        }
        let bounds = extent.ok_or_else(|| Diagnostic::new("render.empty_scene", "project"))?;
        scene.bounds = bounds.inset(-24.);
        Ok(scene)
    }
    fn push(&mut self, id: &Id, primitive: Primitive) {
        self.items.push(Item {
            source: id.clone(),
            primitive,
        });
    }
    fn link(&mut self, id: &Id, points: [f32; 4], color: Color, dashed: bool, contrast: bool) {
        if contrast {
            self.push(
                id,
                Primitive::Line {
                    points,
                    color: [15, 30, 40, 255],
                    width: 4.,
                    dashed,
                },
            );
        }
        self.push(
            id,
            Primitive::Line {
                points,
                color,
                width: 2.,
                dashed,
            },
        );
    }
    fn text(
        &mut self,
        id: &Id,
        bounds: Bounds,
        origin: [f32; 2],
        layout: TextLayout,
        paint: TextPaint,
    ) -> Result<(), Diagnostic> {
        let TextPaint {
            color,
            italic,
            underline,
        } = paint;
        self.counts.glyphs += layout.lines.iter().map(|l| l.glyphs.len()).sum::<usize>();
        self.counts.clipped_texts += usize::from(layout.clipped);
        if self.counts.glyphs > sns_fonts::MAX_GLYPHS {
            return Err(limit("scene_glyphs"));
        }
        self.push(
            id,
            Primitive::Text {
                bounds,
                origin,
                layout,
                color,
                italic,
                underline,
            },
        );
        Ok(())
    }
}
fn text(
    value: &str,
    face: Face,
    size: f32,
    bounds: Bounds,
    rows: usize,
    ellipsis: bool,
) -> Result<TextLayout, Diagnostic> {
    sns_fonts::layout(
        value,
        face,
        size,
        bounds.width,
        bounds.height,
        rows,
        ellipsis,
    )
    .map_err(|e| match e {
        sns_fonts::FontError::Unsupported(c) => {
            Diagnostic::new("render.unsupported_glyph", format!("U+{:04X}", c as u32))
        }
        sns_fonts::FontError::Limit => limit("document_text"),
        sns_fonts::FontError::Invalid => Diagnostic::new("render.invalid_font", "bundled_font"),
    })
}
fn limit(context: &str) -> Diagnostic {
    Diagnostic::new("resource.limit_exceeded", context)
}
