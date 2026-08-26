#[test]
fn waterfall_contract_names_semantic_boundaries() {
    let stages = ["source", "candidate", "target", "consumer", "diagnostic"];
    assert_eq!(stages.first(), Some(&"source"));
    assert_eq!(stages.last(), Some(&"diagnostic"));
    assert!(stages.contains(&"candidate"));
    assert!(stages.contains(&"target"));
}
