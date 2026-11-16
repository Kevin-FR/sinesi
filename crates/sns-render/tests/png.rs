use sha2::{Digest, Sha256};
use sns_model::*;
use sns_render::*;
use std::collections::BTreeSet;
fn project() -> Project {
    sns_project::parse_project(include_bytes!("../../../fixtures/valid/annotations.json")).unwrap()
}
fn record(name: &str, image: &PngImage, scene: &Scene) {
    if let Some(root) = std::env::var_os("SNS_RECORD_J7_PNG") {
        let root = std::path::Path::new(&root);
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join(format!("{name}.png")), &image.bytes).unwrap();
        let b = scene.bounds();
        let c = scene.counts();
        std::fs::write(root.join(format!("{name}.json")),serde_json::to_vec_pretty(&serde_json::json!({"width":image.width,"height":image.height,"sha256":format!("{:x}",Sha256::digest(&image.bytes)),"bounds":[b.x,b.y,b.width,b.height],"devices":c.devices,"links":c.links,"annotations":c.annotations,"glyphs":c.glyphs,"clipped_texts":c.clipped_texts,"object_ids":scene.object_ids().iter().map(Id::as_str).collect::<Vec<_>>()})).unwrap()).unwrap();
    }
}
#[test]
fn png_frames_every_object_on_transparent_margin_in_both_views_without_mutating_project() {
    let mut project = project();
    project.annotations[0].x = -300.;
    project.annotations[0].y = -250.;
    project.annotations[1].x = 850.;
    project.annotations[1].y = 480.;
    let original = project.clone();
    let expected: BTreeSet<_> = project
        .devices
        .iter()
        .map(|d| d.id.clone())
        .chain(project.links.iter().map(|l| l.id.clone()))
        .chain(project.annotations.iter().map(|a| a.id.clone()))
        .collect();
    for diagram in [false, true] {
        let scene = Scene::build(&project, diagram).unwrap();
        assert_eq!(scene.object_ids(), &expected);
        let b = scene.bounds();
        assert!(b.x <= -324. && b.y <= -274.);
        assert!(b.x + b.width >= 1174. && b.y + b.height >= 634.);
        let options = PngOptions {
            longest_side: 1200,
            transparent: true,
        };
        let a = render_png(&scene, options).unwrap();
        let again = render_png(&scene, options).unwrap();
        assert_eq!(a.bytes, again.bytes);
        let decoded = image::load_from_memory(&a.bytes).unwrap().to_rgba8();
        assert_eq!(decoded.dimensions(), (a.width, a.height));
        assert_eq!(a.width.max(a.height), 1200);
        for x in 0..a.width {
            assert_eq!(decoded.get_pixel(x, 0).0[3], 0);
            assert_eq!(decoded.get_pixel(x, a.height - 1).0[3], 0);
        }
        for y in 0..a.height {
            assert_eq!(decoded.get_pixel(0, y).0[3], 0);
            assert_eq!(decoded.get_pixel(a.width - 1, y).0[3], 0);
        }
        let scale = (1200. - 32.) / b.width.max(b.height);
        for device in &project.devices {
            let position = project
                .layout
                .devices
                .iter()
                .find(|p| p.device_id == device.id)
                .unwrap();
            let x = (16. + (position.x - b.x) * scale) as u32;
            let y = (16. + (position.y - b.y - 10.) * scale) as u32;
            // Check a region rather than a center pixel: schematic outlines have empty interiors.
            let visible = (x - 20..=x + 20)
                .flat_map(|x| (y - 20..=y + 20).map(move |y| (x, y)))
                .any(|(x, y)| decoded.get_pixel(x, y).0[3] > 0);
            assert!(visible, "{}", device.label);
        }
        let opaque = render_png(
            &scene,
            PngOptions {
                transparent: false,
                ..options
            },
        )
        .unwrap();
        let pixels = image::load_from_memory(&opaque.bytes).unwrap().to_rgba8();
        assert!(pixels.pixels().all(|p| p.0[3] == 255));
        record(
            if diagram {
                "j7-png-diagram"
            } else {
                "j7-png-icons"
            },
            &a,
            &scene,
        );
    }
    assert_eq!(project, original);
}
#[test]
fn annotation_only_and_fallback_layout_work_while_invalid_inputs_fail_before_output() {
    let mut p = sns_topology::Editor::empty().project().clone();
    let note = project().annotations[1].clone();
    p = p.with_annotation(note).unwrap();
    let scene = Scene::build(&p, true).unwrap();
    let image = render_png(
        &scene,
        PngOptions {
            longest_side: 512,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(image.width.max(image.height), 512);
    for size in [0, 255, 4097, u32::MAX] {
        assert!(
            render_png(
                &scene,
                PngOptions {
                    longest_side: size,
                    transparent: true
                }
            )
            .is_err()
        );
    }
    p.annotations[0].text = "\u{10ffff}".into();
    assert!(Scene::build(&p, true).is_err());
    let mut p = project();
    p.layout.devices.clear();
    let saved = p.clone();
    let scene = Scene::build(&p, true).unwrap();
    assert_eq!(p, saved);
    assert_eq!(scene.counts().devices, 4);
    assert!(Scene::build(sns_topology::Editor::empty().project(), true).is_err());
}
#[test]
fn every_user_provided_symbol_is_rendered_in_both_views() {
    use sns_topology::{Edit, Editor};
    let mut base = Editor::empty().project().clone();
    base.schema_version = 9;
    base.semantic_profile = DOCUMENTATION_PROFILE.into();
    let mut e = Editor::new(base).unwrap();
    let symbols = [
        DeviceSymbol::Desktop,
        DeviceSymbol::Laptop,
        DeviceSymbol::Server,
        DeviceSymbol::Printer,
        DeviceSymbol::Tablet,
        DeviceSymbol::Router,
        DeviceSymbol::Switch,
        DeviceSymbol::AccessPoint,
        DeviceSymbol::Modem,
        DeviceSymbol::Cloud,
    ];
    for (i, symbol) in symbols.into_iter().enumerate() {
        e.apply(
            e.revision(),
            Edit::AddNamed {
                kind: if symbol == DeviceSymbol::Cloud {
                    DeviceKind::Switch
                } else {
                    DeviceKind::Workstation
                },
                name: format!("{symbol:?}"),
                x: (i % 5) as f32 * 150.,
                y: (i / 5) as f32 * 130.,
            },
        )
        .unwrap();
        let id = e.project().devices.last().unwrap().id.clone();
        e.apply(
            e.revision(),
            Edit::SetSymbol {
                device: id,
                symbol: Some(symbol),
            },
        )
        .unwrap();
    }
    let mut project = e.project().clone();
    for (index, (font, bold)) in [
        (AnnotationFont::Proportional, false),
        (AnnotationFont::Proportional, true),
        (AnnotationFont::Monospace, false),
        (AnnotationFont::Monospace, true),
    ]
    .into_iter()
    .enumerate()
    {
        project = project
            .with_annotation(Annotation {
                id: Id::new(format!("font-proof-{index}")).unwrap(),
                text: format!("{font:?}, bold={bold} : École — Grüße, acción, città, ação"),
                x: -40.,
                y: 240. + index as f32 * 95.,
                style: Some(AnnotationStyle {
                    font,
                    bold,
                    font_size: 20.,
                    width: 730.,
                    height: 90.,
                    italic: bold,
                    underline: bold,
                    ..Default::default()
                }),
            })
            .unwrap();
    }
    for diagram in [false, true] {
        let scene = Scene::build(&project, diagram).unwrap();
        let image = render_png(
            &scene,
            PngOptions {
                longest_side: 1600,
                ..Default::default()
            },
        )
        .unwrap();
        record(
            if diagram {
                "j7-png-symbols-diagram"
            } else {
                "j7-png-symbols-icons"
            },
            &image,
            &scene,
        );
    }
}
