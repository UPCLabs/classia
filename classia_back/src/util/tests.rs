use serde::Deserialize;
use validator::Validate;

use super::{
    deserializers::{deserialize_optional_trimmed, deserialize_trimmed},
    password::{hash_password, validate_password},
    validation::validation_message,
};

#[derive(Deserialize)]
struct TrimmedValues {
    #[serde(deserialize_with = "deserialize_trimmed")]
    required: String,
    #[serde(deserialize_with = "deserialize_optional_trimmed")]
    optional: Option<String>,
}

#[derive(Validate)]
struct InvalidValue {
    #[validate(length(min = 1, message = "obligatorio"))]
    name: String,
}

#[test]
fn hashes_and_validates_passwords() {
    let hash = hash_password("correct-password").expect("password hashing succeeds");

    assert!(validate_password("correct-password", &hash).expect("valid hash"));
    assert!(!validate_password("wrong-password", &hash).expect("valid hash"));
    assert!(validate_password("password", "not-a-phc-hash").is_err());
}

#[test]
fn deserializers_trim_required_and_optional_values() {
    let values: TrimmedValues = serde_json::from_value(serde_json::json!({
        "required": "  value  ",
        "optional": "  optional  "
    }))
    .expect("valid JSON");

    assert_eq!(values.required, "value");
    assert_eq!(values.optional.as_deref(), Some("optional"));
}

#[test]
fn validation_messages_include_field_and_reason() {
    let errors = InvalidValue {
        name: String::new(),
    }
    .validate()
    .expect_err("empty name is invalid");

    assert_eq!(validation_message(errors), "name: obligatorio");
}
