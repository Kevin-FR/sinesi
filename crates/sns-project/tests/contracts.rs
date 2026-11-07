use serde_json::Value;
use sns_engine::{Engine, Limits};
use sns_model::{CommandEnvelope, Diagnostic};

fn schema(name: &str) -> jsonschema::Validator {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../../contracts/{name}.schema.json"));
    let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    jsonschema::options()
        .should_validate_formats(true)
        .build(&value)
        .unwrap()
}
#[test]
fn neutral_schemas_accept_actual_serialized_contracts() {
    let scenario =
        sns_project::parse_scenario(include_bytes!("../../../fixtures/valid/scenario.json"))
            .unwrap();
    schema("scenario")
        .validate(&serde_json::to_value(&scenario).unwrap())
        .unwrap();
    schema("project")
        .validate(&serde_json::to_value(&scenario.project).unwrap())
        .unwrap();
    let command: CommandEnvelope =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/command.json")).unwrap();
    schema("command")
        .validate(&serde_json::to_value(command).unwrap())
        .unwrap();
    let diagnostic: Diagnostic =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/error.json")).unwrap();
    schema("error")
        .validate(&serde_json::to_value(diagnostic).unwrap())
        .unwrap();
    let mut engine = Engine::new(scenario, None, Limits::default()).unwrap();
    engine.run(100, None).unwrap();
    let event_schema = schema("event");
    for event in engine.trace() {
        event_schema
            .validate(&serde_json::to_value(event).unwrap())
            .unwrap();
    }
}
#[test]
fn schemas_reject_unknown_properties_versions_and_invalid_counters() {
    let mut value: Value =
        serde_json::from_slice(include_bytes!("../../../fixtures/valid/command.json")).unwrap();
    let schema = schema("command");
    for invalid in [
        serde_json::json!(42),
        serde_json::json!("01"),
        serde_json::json!("18446744073709551616"),
    ] {
        value["expected_revision"] = invalid;
        assert!(!schema.is_valid(&value));
    }
    value["expected_revision"] = serde_json::json!("18446744073709551615");
    assert!(schema.is_valid(&value));
    value["api_version"] = serde_json::json!(99);
    assert!(!schema.is_valid(&value));
    value["api_version"] = serde_json::json!(1);
    value["execute_java"] = serde_json::json!(true);
    assert!(!schema.is_valid(&value));
}
#[test]
fn semantic_reference_checks_complement_the_structural_schema() {
    for bytes in [
        include_bytes!("../../../fixtures/invalid/dangling-link.json").as_slice(),
        include_bytes!("../../../fixtures/invalid/duplicate-device.json").as_slice(),
    ] {
        let value: Value = serde_json::from_slice(bytes).unwrap();
        assert!(schema("project").is_valid(&value));
        assert!(sns_project::parse_project(bytes).is_err());
    }
    let value: Value = serde_json::from_slice(include_bytes!(
        "../../../fixtures/invalid/unknown-version.json"
    ))
    .unwrap();
    assert!(!schema("project").is_valid(&value));
}
