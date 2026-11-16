//! The portable mission and report contracts accept what Sinesi writes.
use serde_json::Value;
use std::collections::BTreeSet;

fn schema(bytes: &[u8]) -> jsonschema::Validator {
    jsonschema::validator_for(&serde_json::from_slice::<Value>(bytes).unwrap()).unwrap()
}

#[test]
fn builtin_missions_and_reports_follow_their_contracts() {
    let missions = schema(include_bytes!("../../../contracts/mission.schema.json"));
    let reports = schema(include_bytes!(
        "../../../contracts/mission-report.schema.json"
    ));
    for file in ["forgotten-cable", "forgotten-gateway", "broken-directory"] {
        let path = format!("{}/missions/{file}.json", env!("CARGO_MANIFEST_DIR"));
        let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert!(missions.is_valid(&value), "{file}");
    }
    for mission in sns_missions::builtin() {
        for (hints, observed) in [(0, BTreeSet::new()), (9, BTreeSet::from(["ping".into()]))] {
            for language in ["fr", "de"] {
                let report = sns_missions::report(
                    &mission,
                    &mission.project,
                    &[],
                    &observed,
                    hints,
                    language,
                );
                assert!(reports.is_valid(&serde_json::to_value(&report).unwrap()));
            }
        }
    }
    // An unknown check kind is refused by both the contract and the reader.
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../missions/forgotten-gateway.json")).unwrap();
    value["objectives"][0]["check"] = serde_json::json!({"kind":"teleport","device":"a"});
    assert!(!missions.is_valid(&value));
    assert!(sns_missions::parse(&serde_json::to_vec(&value).unwrap()).is_err());
}
