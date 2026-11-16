//! Embedded user-provided vector assets, rasterized once; no archive runtime dependency.
use sns_model::DeviceSymbol;
use std::collections::BTreeMap;
#[derive(Default)]
pub struct NetworkArt {
    pub diagram: bool,
    textures: BTreeMap<(bool, u8), egui::TextureHandle>,
}
impl NetworkArt {
    pub fn texture(&mut self, ctx: &egui::Context, symbol: DeviceSymbol) -> egui::TextureId {
        let key = (self.diagram, symbol as u8);
        self.textures
            .entry(key)
            .or_insert_with(|| {
                let image = egui_extras::image::load_svg_bytes_with_size(
                    sns_render::symbol_svg(symbol, self.diagram),
                    egui::load::SizeHint::Size {
                        width: 256,
                        height: 256,
                        maintain_aspect_ratio: true,
                    },
                    &Default::default(),
                )
                .expect("checked embedded SVG");
                ctx.load_texture(
                    format!("network-{key:?}"),
                    image,
                    egui::TextureOptions::LINEAR,
                )
            })
            .id()
    }
}
