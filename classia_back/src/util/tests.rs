use serde::Deserialize;
use sqlx::PgPool;
use validator::Validate;

use super::{
    database::is_unique_violation,
    deserializers::{deserialize_optional_trimmed, deserialize_trimmed},
    password::{hash_password, validate_password},
    validation::{normalize_email, validation_message},
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

#[test]
fn emails_are_trimmed_and_lowercased() {
    assert_eq!(normalize_email("  Ada@Example.COM "), "ada@example.com");
}

#[sqlx::test(migrations = "./migrations")]
async fn detects_unique_violations_only(pool: PgPool) {
    let insert = |email: &'static str| {
        sqlx::query(
            "INSERT INTO users (name, email, password, role, status, token) \
             VALUES ('User', $1, 'hash', 'student', 'active', '')",
        )
        .bind(email)
    };

    insert("unique@example.com")
        .execute(&pool)
        .await
        .expect("first insert succeeds");
    let duplicate = insert("unique@example.com")
        .execute(&pool)
        .await
        .expect_err("duplicate email is rejected");
    assert!(is_unique_violation(&duplicate));

    let missing_row = sqlx::query("SELECT 1 FROM users WHERE false")
        .fetch_one(&pool)
        .await
        .expect_err("no row is returned");
    assert!(!is_unique_violation(&missing_row));
}
