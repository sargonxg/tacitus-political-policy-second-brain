use tacitus_capsules::strict_json::{from_slice, validate, MAX_BYTES, MAX_DEPTH, MAX_NODES};
#[test]
fn duplicate_keys_at_every_level_and_decoded_equivalence() {
    for raw in [
        r#"{"a":1,"a":2}"#,
        r#"{"x":[{"a":1,"\u0061":2}]}"#,
        r#"{"é":1,"\u00e9":2}"#,
        r#"{"😀":1,"\ud83d\ude00":2}"#,
    ] {
        let e = validate(raw.as_bytes()).unwrap_err();
        assert!(e.contains("duplicate JSON object key"), "{e}");
        assert_eq!(e, validate(raw.as_bytes()).unwrap_err());
    }
}
#[test]
fn arrays_scopes_and_unicode_remain_distinct_without_normalization() {
    for raw in [
        r#"[{"a":1},{"a":2}]"#,
        r#"{"a":{"a":1}}"#,
        r#"{"é":1,"e\u0301":2}"#,
        r#"[null,true,false,{},[],"literal { a:a }"]"#,
    ] {
        validate(raw.as_bytes()).unwrap();
        let parsed: serde_json::Value = from_slice(raw.as_bytes()).unwrap();
        assert_eq!(
            parsed,
            serde_json::from_str::<serde_json::Value>(raw).unwrap()
        );
    }
}
#[test]
fn depth_nodes_and_bytes_are_bounded() {
    let valid = format!("{}0{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
    validate(valid.as_bytes()).unwrap();
    let deep = format!(
        "{}0{}",
        "[".repeat(MAX_DEPTH + 1),
        "]".repeat(MAX_DEPTH + 1)
    );
    assert!(validate(deep.as_bytes())
        .unwrap_err()
        .contains("depth limit"));
    let at_nodes = format!("[{}]", vec!["0"; MAX_NODES - 1].join(","));
    validate(at_nodes.as_bytes()).unwrap();
    let mut at_bytes = vec![b' '; MAX_BYTES];
    at_bytes[0] = b'0';
    validate(&at_bytes).unwrap();
    let many = format!("[{}]", vec!["0"; MAX_NODES].join(","));
    assert!(validate(many.as_bytes())
        .unwrap_err()
        .contains("node limit"));
    assert!(validate(&vec![b' '; MAX_BYTES + 1])
        .unwrap_err()
        .contains("byte limit"));
}
#[test]
fn malformed_utf8_syntax_surrogates_and_nonfinite_numbers_rejected() {
    assert_eq!(validate(b"\"\xff\"").unwrap_err(), "invalid JSON UTF-8");
    for raw in [
        "NaN",
        "Infinity",
        "1e400",
        "01",
        "1.",
        "+1",
        "{}{}",
        "[1,]",
        r#""\ud800""#,
        r#"{"a":}"#,
    ] {
        assert!(validate(raw.as_bytes()).is_err(), "{raw}");
    }
}
#[test]
fn numeric_edges_preserve_existing_serde_semantics_not_jcs() {
    for raw in [
        "-0",
        "1e-300",
        "18446744073709551615",
        "-9223372036854775808",
        "9007199254740993",
    ] {
        let v: serde_json::Value = from_slice(raw.as_bytes()).unwrap();
        assert_eq!(v, serde_json::from_str::<serde_json::Value>(raw).unwrap());
    }
    assert!(from_slice::<u64>(b"18446744073709551616").is_err());
}
