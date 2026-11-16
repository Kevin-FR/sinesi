use sns_model::*;
use sns_topology::{GatewayAdvice, gateway_advice};

fn host(gateway: Option<&str>, ipv4: Option<Ipv4Config>) -> Device {
    let project: Project =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/routed.json")).unwrap();
    let mut device = project.devices[0].clone();
    device.network.default_gateway = gateway.map(str::to_owned);
    device.interfaces[0].ipv4 = ipv4;
    device
}
fn static_ip(address: &str, prefix_length: u8) -> Option<Ipv4Config> {
    Some(Ipv4Config::Static {
        address: address.into(),
        prefix_length,
    })
}

#[test]
fn gateway_advice_explains_missing_invalid_outside_and_reachable_values() {
    let ip = || static_ip("192.168.10.2", 24);
    assert_eq!(gateway_advice(&host(None, ip())), GatewayAdvice::Missing);
    assert_eq!(
        gateway_advice(&host(Some("  "), ip())),
        GatewayAdvice::Missing
    );
    assert_eq!(
        gateway_advice(&host(Some("192.168.10.1"), ip())),
        GatewayAdvice::Reachable
    );
    assert_eq!(
        gateway_advice(&host(Some("192.168.1.1"), ip())),
        GatewayAdvice::OutsideNetwork {
            network: "192.168.10.0/24".into()
        }
    );
    assert_eq!(
        gateway_advice(&host(Some("192.168.10.2"), ip())),
        GatewayAdvice::OwnAddress
    );
    assert_eq!(
        gateway_advice(&host(Some("192.168.10"), ip())),
        GatewayAdvice::Invalid
    );
    // /16 widens the network: 192.168.1.1 becomes reachable.
    assert_eq!(
        gateway_advice(&host(Some("192.168.1.1"), static_ip("192.168.10.2", 16))),
        GatewayAdvice::Reachable
    );
    // DHCP hosts and network devices are not judged on a guess.
    assert_eq!(
        gateway_advice(&host(Some("10.0.0.1"), Some(Ipv4Config::Dhcp))),
        GatewayAdvice::NotApplicable
    );
    let mut router = host(None, ip());
    router.kind = DeviceKind::Router;
    assert_eq!(gateway_advice(&router), GatewayAdvice::NotApplicable);
}

#[test]
fn free_position_never_covers_an_existing_device() {
    let mut project: Project =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/lan.json")).unwrap();
    let mut editor = sns_topology::Editor::new(project.clone()).unwrap();
    for n in 0..12 {
        let (x, y) = sns_topology::free_position(editor.project());
        for p in &editor.project().layout.devices {
            assert!(
                (p.x - x).abs() >= 130. || (p.y - y).abs() >= 100.,
                "device {n} at ({x},{y}) covers {}",
                p.device_id.as_str()
            );
        }
        editor
            .apply(
                editor.revision(),
                sns_topology::Edit::AddNamed {
                    kind: DeviceKind::Workstation,
                    name: format!("Poste {n}"),
                    x,
                    y,
                },
            )
            .unwrap();
    }
    project.layout.devices.clear();
    assert!(sns_topology::free_position(&project).0.is_finite());
}
