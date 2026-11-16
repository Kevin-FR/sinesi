// Deterministic reference load shared by engine/UI measurement, not product data.
pub fn project() -> sns_model::Project {
    use sns_model::*;
    let id = |s: String| Id::new(s).unwrap();
    let mut p: Project = serde_json::from_str(include_str!("valid/lan.json")).unwrap();
    p.devices.clear();
    p.links.clear();
    p.layout.devices.clear();
    p.lan.clear();
    for n in 0..100 {
        let switch = n >= 50;
        let device_id = id(format!("node-{n}"));
        let interfaces = (0..if switch { 8 } else { 1 })
            .map(|port| Interface {
                id: id(format!("node-{n}-p{port}")),
                mac: format!("02:00:00:00:{n:02x}:{port:02x}"),
                medium: Medium::Ethernet,
                ipv4: if switch {
                    None
                } else {
                    Some(Ipv4Config::Static {
                        address: format!("192.168.1.{}", n + 2),
                        prefix_length: 24,
                    })
                },
            })
            .collect();
        p.devices.push(Device {
            id: device_id.clone(),
            kind: if switch {
                DeviceKind::Switch
            } else {
                DeviceKind::Workstation
            },
            label: format!("Nœud {n}"),
            interfaces,
            network: Network::default(),
            applications: vec![],
            files: vec![],
        });
        p.layout.devices.push(Position {
            device_id,
            x: (n % 10) as f32 * 150.,
            y: (n / 10) as f32 * 100.,
        });
    }
    let mut connect = |a: usize, ap: usize, b: usize, bp: usize| {
        p.links.push(Link {
            id: id(format!("load-link-{}", p.links.len())),
            endpoints: [id(format!("node-{a}-p{ap}")), id(format!("node-{b}-p{bp}"))],
            latency_us: Decimal(1000),
            jitter_us: Decimal(0),
        });
    };
    for n in 0..50 {
        connect(n, 0, n + 50, 0);
        connect(n + 50, 1, (n + 1) % 50 + 50, 2);
        connect(n + 50, 3, (n + 7) % 50 + 50, 4);
    }
    p.validate().unwrap();
    p
}
