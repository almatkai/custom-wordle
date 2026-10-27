use serde_json::json;
use tezbar_custom_wordle_v2::*;

#[test]
fn accepted_lengths_casing_key_cycle_and_fixed_route() {
    for n in [1, 2, 5, 6, 7, 31, 10_000, MAX_UTF16_UNITS] {
        let result = generate(&"A".repeat(n)).unwrap();
        assert_eq!(result.word, "a".repeat(n));
        let expected: String = "wordle".chars().cycle().take(n).collect();
        assert_eq!(result.url, format!("{URL_PREFIX}{expected}"));
    }
    assert_eq!(generate("APPLE").unwrap().url, format!("{URL_PREFIX}wdgop"));
    assert_eq!(generate("\u{feff} HeLLo\u{2009}").unwrap().word, "hello");
    assert_eq!(generate("K").unwrap().word, "k");
}

#[test]
fn ecmascript_whitespace_and_utf16_failures() {
    for unit in [
        9, 10, 11, 12, 13, 32, 160, 5760, 8192, 8193, 8194, 8195, 8196, 8197, 8198, 8199, 8200,
        8201, 8202, 8232, 8233, 8239, 8287, 12288, 65279,
    ] {
        assert_eq!(generate_utf16(&[unit]), Err(Failure::Empty));
        assert_eq!(generate_utf16(&[unit, 65, unit]).unwrap().word, "a");
    }
    for units in [
        vec![0xd800],
        vec![0xdc00],
        vec![0xd83d, 0xde00],
        vec![0x85],
        vec![0x180e],
        vec![0x200b],
        vec![0x0130],
    ] {
        assert_eq!(generate_utf16(&units), Err(Failure::Letters));
    }
    assert_eq!(generate(" ! a "), Err(Failure::Spaces));
    for word in [
        "a\tb",
        "a\nb",
        "a+b",
        "a&b",
        "a%b",
        "a?b",
        "é",
        "Қазақ",
        "日本語",
        "😀",
        "ſ",
    ] {
        assert_eq!(generate(word), Err(Failure::Letters));
    }
}

#[test]
fn no_coercion_missing_malformed_and_resource_errors() {
    for value in [
        json!(null),
        json!(false),
        json!(1),
        json!([]),
        json!({}),
        json!("text"),
    ] {
        assert_eq!(
            from_payload(&json!({"argumentValues":value})),
            Err(Failure::MalformedArgument)
        );
        assert_eq!(
            from_payload(&json!({"argumentValues":{"word":value}})),
            if value.is_string() {
                generate(value.as_str().unwrap())
            } else {
                Err(Failure::MalformedArgument)
            }
        );
    }
    assert_eq!(
        from_payload(&json!({"formValues":{"word":"apple"},"preferences":{"word":"apple"}})),
        Err(Failure::MalformedArgument)
    );
    assert_eq!(
        generate(&" ".repeat(MAX_UTF16_UNITS + 1)),
        Err(Failure::Resource)
    );
    assert_eq!(
        generate(&"😀".repeat(MAX_UTF16_UNITS / 2 + 1)),
        Err(Failure::Resource)
    );
}

#[test]
fn strict_effect_acknowledgements() {
    for value in [
        json!(null),
        json!(true),
        json!(false),
        json!({}),
        json!({"ok":false}),
        json!({"ok":true,"success":false}),
        json!({"ok":true,"error":"private"}),
        json!({"ok":true,"error":false}),
        json!({"ok":true,"success":null}),
        json!({"ok":"true"}),
    ] {
        assert!(!acknowledged(&value), "{value}");
    }
    for value in [
        json!({"ok":true}),
        json!({"ok":true,"error":null}),
        json!({"ok":true,"success":true,"extra":1}),
    ] {
        assert!(acknowledged(&value));
    }
}
