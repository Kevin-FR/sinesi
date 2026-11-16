use sns_model::*;
use sns_topology::*;
use std::collections::BTreeSet;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn editor() -> Editor {
    Editor::new(serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap())
        .unwrap()
}
#[test]
fn group_delete_undo_restores_everything_and_history_branches() {
    let mut e = editor();
    let original = e.project().clone();
    e.apply(
        0,
        Edit::Delete(BTreeSet::from([id("host-a"), id("switch")])),
    )
    .unwrap();
    assert_eq!(e.project().links.len(), 0);
    assert_eq!(e.project().devices.len(), 2);
    e.undo();
    assert_eq!(e.project(), &original);
    assert!(!e.dirty());
    e.redo();
    assert!(e.dirty());
    e.undo();
    e.apply(
        e.revision(),
        Edit::Move(vec![
            Position {
                device_id: id("host-a"),
                x: 15.,
                y: 20.,
            },
            Position {
                device_id: id("host-b"),
                x: 25.,
                y: 30.,
            },
        ]),
    )
    .unwrap();
    assert!(!e.can_redo());
    assert_eq!(e.project().links, original.links);
    e.undo();
    assert_eq!(e.project(), &original);
}
#[test]
fn invalid_commands_are_transactional_and_identity_survives_configuration() {
    let mut e = editor();
    let original = e.project().clone();
    assert!(
        e.apply(1, Edit::Delete(BTreeSet::from([id("host-a")])))
            .is_err()
    );
    assert!(
        e.apply(
            0,
            Edit::Connect {
                a: id("host-a-p0"),
                b: id("switch-p5")
            }
        )
        .is_err()
    );
    assert_eq!(e.project(), &original);
    assert_eq!(e.revision(), 0);
    let mut d = original.devices[0].clone();
    d.label = "Renommé".into();
    let id = d.id.clone();
    e.apply(
        0,
        Edit::Configure(
            Box::new(d),
            LanConfig {
                display: DisplayName::Ipv4,
                ..Default::default()
            },
        ),
    )
    .unwrap();
    assert_eq!(e.project().devices[0].id, id);
    assert_eq!(
        display_label(e.project(), &e.project().devices[0]),
        "192.168.1.2"
    );
    let mut bad = e.project().devices[0].clone();
    bad.interfaces[0].ipv4 = Some(Ipv4Config::Static {
        address: "192.168.1.2".into(),
        prefix_length: 40,
    });
    assert!(
        e.apply(1, Edit::Configure(Box::new(bad), LanConfig::default()))
            .is_err()
    );
}
#[test]
fn palette_round_trip_and_free_port_connection() {
    let mut e = Editor::empty();
    for (n, kind) in [
        DeviceKind::Workstation,
        DeviceKind::Notebook,
        DeviceKind::Switch,
        DeviceKind::Router,
    ]
    .into_iter()
    .enumerate()
    {
        e.apply(
            e.revision(),
            Edit::Add {
                kind,
                x: n as f32 * 100.,
                y: 20.,
            },
        )
        .unwrap();
    }
    let a = e.project().devices[0].interfaces[0].id.clone();
    let b = e.project().devices[2].interfaces[0].id.clone();
    e.apply(e.revision(), Edit::Connect { a, b }).unwrap();
    let encoded = serde_json::to_vec(e.project()).unwrap();
    let decoded: Project = serde_json::from_slice(&encoded).unwrap();
    decoded.validate().unwrap();
    assert_eq!(&decoded, e.project());
    let d = e.project().devices[2].id.clone();
    e.apply(e.revision(), Edit::Disconnect(d)).unwrap();
    assert!(e.project().links.is_empty());
    e.undo();
    assert_eq!(e.project().links.len(), 1);
}
#[test]
fn imported_missing_layout_and_mac_collisions_are_recovered_without_reusing_identity() {
    let mut p: Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/lan.json")).unwrap();
    p.layout.devices.clear();
    p.devices[0].interfaces[0].mac = "02:00:00:00:00:02".into();
    let mut e = Editor::new(p).unwrap();
    assert_eq!(e.project().layout.devices.len(), 4);
    assert!(e.dirty());
    assert!(
        e.apply(
            0,
            Edit::Add {
                kind: DeviceKind::Switch,
                x: f32::NAN,
                y: 0.
            }
        )
        .is_err()
    );
    assert_eq!(e.revision(), 0);
    e.apply(
        0,
        Edit::Add {
            kind: DeviceKind::Switch,
            x: 0.,
            y: 0.,
        },
    )
    .unwrap();
    assert_eq!(e.project().devices.last().unwrap().id.as_str(), "device-1");
    e.project().validate().unwrap();
}

#[test]
fn router_interfaces_swap_without_changing_identity_and_resize_is_transactional() {
    let p: Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/routed.json")).unwrap();
    let mut editor = Editor::new(p.clone()).unwrap();
    let router = Id::new("r1").unwrap();
    let a = Id::new("r1-p0").unwrap();
    let b = Id::new("r1-p1").unwrap();
    editor
        .apply(
            0,
            Edit::SwapInterfaces {
                device: router.clone(),
                a: a.clone(),
                b: b.clone(),
            },
        )
        .unwrap();
    assert_eq!(editor.project().devices, p.devices);
    assert_eq!(editor.project().links[0].endpoints[1], b);
    assert_eq!(editor.project().links[2], p.links[2]);
    editor.undo();
    assert_eq!(editor.project(), &p);
    editor
        .apply(
            editor.revision(),
            Edit::ResizeInterfaces {
                device: router.clone(),
                count: 4,
            },
        )
        .unwrap();
    assert_eq!(
        editor.project().device(&router).unwrap().interfaces.len(),
        4
    );
    editor
        .apply(
            editor.revision(),
            Edit::ResizeInterfaces {
                device: router.clone(),
                count: 2,
            },
        )
        .unwrap();
    let before = editor.project().clone();
    assert!(
        editor
            .apply(
                editor.revision(),
                Edit::ResizeInterfaces {
                    device: router.clone(),
                    count: 1
                }
            )
            .is_err()
    );
    assert_eq!(*editor.project(), before);
    let mut p = before.clone();
    p.devices
        .iter_mut()
        .find(|d| d.id == router)
        .unwrap()
        .network
        .routes
        .push(StaticRoute {
            network: "198.51.100.0".parse().unwrap(),
            prefix: 24,
            gateway: None,
            interface: a,
        });
    assert!(p.validate().is_ok());
}
#[test]
fn home_router_has_persistent_lan_wan_roles_and_no_profile_downgrade() {
    let mut editor = Editor::empty();
    editor
        .apply(
            editor.revision(),
            Edit::Add {
                kind: DeviceKind::Gateway,
                x: 10.,
                y: 20.,
            },
        )
        .unwrap();
    let host = &editor.project().devices[0];
    let roles = host.network.gateway.clone().unwrap();
    assert_eq!(host.interfaces.len(), 2);
    assert_eq!(host.interfaces[0].id, roles.lan);
    assert_eq!(host.interfaces[1].id, roles.wan);
    assert_eq!(host.interfaces[1].ipv4, Some(sns_model::Ipv4Config::Dhcp));
    assert_eq!(editor.project().schema_version, 6);
    assert!(
        host.applications.iter().any(
            |a| matches!(&a.service,Some(sns_model::ServiceConfig::Http(c)) if c.administration)
        )
    );
    let saved = serde_json::to_vec(editor.project()).unwrap();
    let reloaded: sns_model::Project = serde_json::from_slice(&saved).unwrap();
    reloaded.validate().unwrap();
    assert_eq!(
        reloaded.devices[0].network.gateway.as_ref().unwrap(),
        &roles
    );
    editor
        .apply(
            editor.revision(),
            Edit::Add {
                kind: DeviceKind::Router,
                x: 50.,
                y: 40.,
            },
        )
        .unwrap();
    assert_eq!(editor.project().schema_version, 6);
    editor.undo();
    assert_eq!(editor.project().devices.len(), 1);
    editor.redo();
    assert_eq!(editor.project().devices.len(), 2);
}

#[test]
fn presentation_is_reversible_and_runtime_merge_preserves_local_topology_values() {
    let mut e = editor();
    let original = e.project().clone();
    e.apply(
        0,
        Edit::SetSymbol {
            device: id("switch"),
            symbol: Some(DeviceSymbol::Cloud),
        },
    )
    .unwrap();
    assert_eq!(e.project().schema_version, 8);
    let cloud = e.project().clone();
    e.undo();
    assert_eq!(e.project(), &original);
    e.redo();
    assert_eq!(e.project(), &cloud);
    let mut source = original
        .with_symbol(&id("switch"), Some(DeviceSymbol::Cloud))
        .unwrap();
    source
        .devices
        .iter_mut()
        .find(|d| d.id == id("host-a"))
        .unwrap()
        .label = "Old runtime name".into();
    let current = e.project().device(&id("host-a")).unwrap().clone();
    let mut changed = current;
    changed.label = "Local design name".into();
    e.apply(
        e.revision(),
        Edit::Configure(Box::new(changed), LanConfig::default()),
    )
    .unwrap();
    e.accept_runtime(source, &BTreeSet::from([id("switch")]))
        .unwrap();
    assert_eq!(
        e.project().device(&id("host-a")).unwrap().label,
        "Local design name"
    );
    assert_eq!(
        e.project().lan[&id("switch")].symbol,
        Some(DeviceSymbol::Cloud)
    );
}
