use sns_model::*;
use sns_topology::*;
fn original() -> Project {
    serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap()
}
#[test]
fn annotation_geometry_style_and_stacking_are_transactional_and_reversible() {
    let base = original();
    let mut editor = Editor::new(base.clone()).unwrap();
    for kind in [AnnotationKind::Rectangle, AnnotationKind::Text] {
        editor
            .apply(
                editor.revision(),
                Edit::AddAnnotation {
                    text: "École — LAN\n192.168.1.0/24".into(),
                    x: -20.,
                    y: -40.,
                    kind,
                },
            )
            .unwrap();
    }
    let first = editor.project().annotations[0].clone();
    let mut modified = first.clone();
    modified.x = 35.;
    modified.y = 50.;
    let style = modified.style.as_mut().unwrap();
    style.width = 620.;
    style.height = 360.;
    style.font_size = 23.;
    style.bold = true;
    style.italic = true;
    style.underline = true;
    style.font = AnnotationFont::Monospace;
    style.fill_color = [200, 230, 240, 160];
    editor
        .apply(editor.revision(), Edit::Annotate(modified.clone()))
        .unwrap();
    assert_eq!(editor.project().annotations[0], modified);
    assert_eq!(editor.project().schema_version, 9);
    assert_eq!(editor.project().semantic_profile, DOCUMENTATION_PROFILE);
    assert_eq!(editor.project().devices, base.devices);
    assert_eq!(editor.project().links, base.links);
    assert_eq!(editor.project().lan, base.lan);
    assert_eq!(editor.project().layout, base.layout);
    editor.undo();
    assert_eq!(editor.project().annotations[0], first);
    editor.redo();
    assert_eq!(editor.project().annotations[0], modified);
    let saved = editor.project().clone();
    let revision = editor.revision();
    for value in [0., f32::NAN, f32::INFINITY, 10_001.] {
        let mut bad = modified.clone();
        bad.style.as_mut().unwrap().width = value;
        assert!(editor.apply(revision, Edit::Annotate(bad)).is_err());
        assert_eq!(editor.project(), &saved);
        assert_eq!(editor.revision(), revision);
    }
    let mut bad = modified.clone();
    bad.id = base.devices[0].id.clone();
    assert!(editor.apply(revision, Edit::Annotate(bad)).is_err());
    assert_eq!(editor.project(), &saved);
    let copy = editor
        .apply(
            revision,
            Edit::Duplicate {
                selection: saved.annotations.iter().map(|a| a.id.clone()).collect(),
                dx: 40.,
                dy: 60.,
            },
        )
        .unwrap();
    assert_eq!(copy.created.len(), 2);
    for original in &saved.annotations {
        let duplicate = editor
            .project()
            .annotations
            .iter()
            .find(|a| a.id == copy.identities[&original.id])
            .unwrap();
        assert_eq!(duplicate.style, original.style);
        assert_eq!(duplicate.text, original.text);
        assert_eq!(duplicate.x, original.x + 40.);
        assert_eq!(duplicate.y, original.y + 60.);
    }
}

#[test]
fn annotations_do_not_change_network_state_and_survive_runtime_merge() {
    let base = original();
    let mut editor = Editor::new(base.clone()).unwrap();
    editor
        .apply(
            0,
            Edit::AddAnnotation {
                text: "Documentation seulement".into(),
                x: 10.,
                y: 20.,
                kind: AnnotationKind::Text,
            },
        )
        .unwrap();
    let before = sns_engine::Engine::from_project(base.clone(), 42, Default::default()).unwrap();
    let after =
        sns_engine::Engine::from_project(editor.project().clone(), 42, Default::default()).unwrap();
    for field in [
        "network",
        "applications",
        "queue",
        "rng_state",
        "virtual_time_us",
    ] {
        assert_eq!(
            before.checkpoint_state().unwrap()[field],
            after.checkpoint_state().unwrap()[field],
            "{field}"
        );
    }
    let notes = editor.project().annotations.clone();
    editor.accept_applications(base).unwrap();
    assert_eq!(editor.project().annotations, notes);
    assert_eq!(editor.project().schema_version, 9);
}
