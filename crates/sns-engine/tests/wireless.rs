use sns_engine::{Engine, Limits};
use sns_model::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
fn project() -> Project {
    serde_json::from_str(include_str!("../../../fixtures/valid/wireless.json")).unwrap()
}
fn ping(p: Project, source: &str, target: &str) -> Engine {
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    e.command(CommandEnvelope {
        api_version: 1,
        command_id: id("wifi-ping"),
        project_id: e.project().project_id.clone(),
        run_id: e.run_id().clone(),
        expected_revision: Decimal(0),
        command: Command::Ping {
            source: id(source),
            destination: target.into(),
            ttl: 64,
        },
    })
    .unwrap();
    e.run(10000, Some(5_000_000)).unwrap();
    e
}
#[test]
fn ssid_association_reaches_wired_lan_and_ap_rename_only_detaches_its_clients() {
    let mut p = project();
    assert_eq!(p.wireless_links().len(), 1);
    let e = ping(p.clone(), "host-b", "192.168.1.2");
    assert!(e.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
    assert!(e.trace().iter().any(|t| t.kind == "wireless.frame.sent"));
    p.devices[3].network.ssid = Some("Autre classe".into());
    assert!(p.wireless_links().is_empty());
    let e = ping(p.clone(), "host-b", "192.168.1.2");
    assert!(!e.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
    let e = ping(p.clone(), "host-a", "192.168.1.4");
    assert!(e.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
    p.devices[1]
        .network
        .wireless
        .insert(id("host-b-p0"), "Autre classe".into());
    let bytes = serde_json::to_vec(&p).unwrap();
    let p = serde_json::from_slice(&bytes).unwrap();
    let e = ping(p, "host-b", "192.168.1.4");
    assert!(e.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
}
#[test]
fn access_point_relays_between_two_radios_and_rejects_duplicate_ssids() {
    let mut p = project();
    p.links.retain(|l| !l.endpoints.contains(&id("host-a-p0")));
    p.devices[0].interfaces[0].medium = Medium::Wireless;
    p.devices[0]
        .network
        .wireless
        .insert(id("host-a-p0"), "Classe".into());
    let e = ping(p.clone(), "host-a", "192.168.1.3");
    assert!(e.trace().iter().any(|t| t.kind == "icmp.echo.reply"));
    let mut ap = p.devices[3].clone();
    ap.id = id("second-ap");
    for (i, port) in ap.interfaces.iter_mut().enumerate() {
        port.id = id(&format!("second-ap-{i}"));
        port.mac = format!("02:02:02:02:03:{i:02x}");
    }
    p.devices.push(ap);
    assert!(p.validate().is_err());
}
