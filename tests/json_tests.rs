//! Natural JSON projection behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_scalar() {
    use qubit_value::Value;

    assert_eq!(
        Value::Int32(42).to_json_value().expect("project scalar"),
        serde_json::json!(42),
    );
}
