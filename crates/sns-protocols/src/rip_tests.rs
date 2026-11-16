use crate::*;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sns_model::*;
fn id(s: &str) -> Id {
    Id::new(s).unwrap()
}
#[test]
fn malformed_rip_cannot_update_the_forwarding_table() {
    let p: Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/rip.json")).unwrap();
    let entries = serde_json::json!({"version":2,"entries":[{"network":"198.51.100.0","prefix":24,"metric":1}]});
    let bytes = serde_json::to_vec(&entries).unwrap();
    for (source, ttl, length, data) in [
        ("192.168.20.2", 1, 0, STANDARD.encode(&bytes)),
        (
            "203.0.113.2",
            1,
            (bytes.len() + 8) as u16,
            STANDARD.encode(&bytes),
        ),
        (
            "192.168.20.2",
            2,
            (bytes.len() + 8) as u16,
            STANDARD.encode(&bytes),
        ),
        ("192.168.20.2", 1, 10, "@@".into()),
    ] {
        let mut n = crate::Network::default();
        let frame = Frame {
            frame_id: Decimal(1),
            source_mac: "02:01:00:00:00:04".into(),
            destination_mac: "ff:ff:ff:ff:ff:ff".into(),
            ether_type: 0x0800,
            payload: Payload::Ipv4Transport {
                source: source.parse().unwrap(),
                destination: "224.0.0.9".parse().unwrap(),
                ttl,
                packet: TransportPacket::Udp {
                    source_port: 520,
                    destination_port: 520,
                    length,
                    data_base64: data,
                },
            },
            remaining_hops: 64,
        };
        let (_, facts) = n
            .process(
                &p,
                0,
                &Event {
                    correlation: id("invalid"),
                    action: Action::Receive {
                        interface: id("r1-p1"),
                        link: id("link-1"),
                        frame,
                    },
                },
            )
            .unwrap();
        assert!(n.routing.learned.is_empty());
        assert!(facts.iter().any(|f| f.kind == "rip.advertisement.rejected"));
    }
}
