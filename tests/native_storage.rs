use std::collections::HashMap;

#[test]
fn restores_large_tu4_source_from_native_storage() {
    // Native eframe storage wraps project JSON (and its escaped newlines) in RON.
    // Many escapes between quotes triggered quadratic scanning at startup.
    let source = "1,λ,>,2 // move right\n".repeat(200_000);
    let library = serde_json::json!({ "source": source }).to_string();
    let saved = HashMap::from([("tmtinker.projects.v1".to_owned(), library)]);
    let encoded =
        ron::ser::to_string_pretty(&saved, Default::default()).expect("serialize storage");
    let restored: HashMap<String, String> = ron::from_str(&encoded).expect("restore storage");
    assert_eq!(restored, saved);
}

#[test]
fn native_storage_preserves_string_escapes() {
    let values = ["", "plain λ", "\"quoted\"", "\\\n\r\t\0", "λ\n\"\\😀"];
    for value in values {
        let encoded = ron::to_string(value).expect("serialize string");
        let restored: String = ron::from_str(&encoded).expect("restore string");
        assert_eq!(restored, value);
    }
    let unicode: String = ron::from_str(r#""\u{3bb}\n\u{1f600}""#).expect("unicode escapes");
    assert_eq!(unicode, "λ\n😀");
    assert!(ron::from_str::<String>("\"unterminated\\n").is_err());
}
