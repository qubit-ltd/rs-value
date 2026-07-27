//! Owned collection V1 wire behavior.

#[test]
fn test_owned_collection_wire_round_trip() {
    use qubit_value::{ValueContainer, ValueWireV1};

    let wire = ValueWireV1::try_from(ValueContainer::from(vec![1_i32])).expect("construct wire");
    assert_eq!(
        serde_json::from_value::<ValueWireV1>(serde_json::to_value(wire).unwrap())
            .unwrap()
            .into_container(),
        ValueContainer::from(vec![1_i32])
    );
}
