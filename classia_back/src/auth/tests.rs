use chrono::Utc;
use uuid::Uuid;

use super::{
    error::AuthError,
    extractor::find_cookie,
    jwt::{JwtConfig, JwtService},
};
use crate::{
    error::AppError,
    users::{User, UserRole},
};

const SECRET: &str = "0123456789abcdef0123456789abcdef";

fn user(role: UserRole) -> User {
    let now = Utc::now();
    User {
        id: Uuid::now_v7(),
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password: "unused".to_string(),
        role,
        status: "active".to_string(),
        token: String::new(),
        created_at: now,
        updated_at: now,
    }
}

fn service(secret: &str, issuer: &str, expiration: i64) -> JwtService {
    JwtService::new(JwtConfig {
        secret: secret.to_string(),
        issuer: issuer.to_string(),
        expiration,
    })
    .expect("valid JWT configuration")
}

#[test]
fn rejects_short_jwt_secret() {
    let result = JwtService::new(JwtConfig {
        secret: "too-short".to_string(),
        issuer: "classia".to_string(),
        expiration: 60,
    });

    assert!(
        matches!(result, Err(AuthError::InternalError(message)) if message == "Secret is too short")
    );
}

#[test]
fn jwt_round_trip_preserves_identity_and_role() {
    let jwt = service(SECRET, "classia", 300);
    let user = user(UserRole::Teacher);

    let token = jwt.generate(&user).expect("token generation succeeds");
    let claims = jwt.validate(&token).expect("generated token is valid");

    assert_eq!(claims.sub, user.id);
    assert_eq!(claims.role, UserRole::Teacher);
    assert_eq!(claims.iss, "classia");
    assert_eq!(claims.exp - claims.iat, 300);
}

#[test]
fn rejects_token_signed_with_another_secret() {
    let token = service(SECRET, "classia", 300)
        .generate(&user(UserRole::Student))
        .expect("token generation succeeds");
    let other = service("fedcba9876543210fedcba9876543210", "classia", 300);

    assert!(
        matches!(other.validate(&token), Err(AuthError::JwtError(message)) if message == "Token invalid")
    );
}

#[test]
fn rejects_expired_token() {
    let jwt = service(SECRET, "classia", -1);
    let token = jwt
        .generate(&user(UserRole::Admin))
        .expect("token generation succeeds");

    assert!(
        matches!(jwt.validate(&token), Err(AuthError::JwtError(message)) if message == "Token expired")
    );
}

#[test]
fn cookie_lookup_matches_exact_name() {
    let cookies = "theme=dark; classia_access=abc.def==; classia_access_backup=other";

    assert_eq!(find_cookie(cookies, "classia_access"), Some("abc.def=="));
    assert_eq!(find_cookie(cookies, "missing"), None);
}

#[test]
fn auth_errors_map_to_expected_statuses() {
    let invalid: AppError = AuthError::InvalidCredentials.into();
    let inactive: AppError = AuthError::InactiveUser.into();
    let internal: AppError = AuthError::InternalError("failure".to_string()).into();

    assert_eq!(invalid.status, axum::http::StatusCode::UNAUTHORIZED);
    assert_eq!(invalid.message, "Credenciales inválidas");
    assert_eq!(inactive.status, axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(
        internal.status,
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    );
}

mod mfa_crypto {
    use uuid::Uuid;

    use crate::auth::mfa::{MfaCipher, code_at, generate_totp, is_six_digits, verify_code};

    const KEY: &str = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";

    #[test]
    fn rejects_keys_that_are_not_32_hex_bytes() {
        assert!(MfaCipher::from_hex("abcd").is_err());
        assert!(MfaCipher::from_hex(&"zz".repeat(32)).is_err());
        assert!(MfaCipher::from_hex(KEY).is_ok());
    }

    #[test]
    fn encrypted_secret_round_trips_only_for_the_same_user() {
        let cipher = MfaCipher::from_hex(KEY).expect("valid key");
        let user = Uuid::now_v7();
        let encrypted = cipher
            .encrypt(user, b"secret-bytes-0123456")
            .expect("encrypts");

        assert_eq!(
            cipher
                .decrypt(user, &encrypted.ciphertext, &encrypted.nonce)
                .expect("decrypts"),
            b"secret-bytes-0123456"
        );
        assert!(
            cipher
                .decrypt(Uuid::now_v7(), &encrypted.ciphertext, &encrypted.nonce)
                .is_err()
        );
        assert!(
            cipher
                .decrypt(user, &encrypted.ciphertext, &[0; 11])
                .is_err()
        );
    }

    #[test]
    fn each_encryption_uses_a_fresh_nonce() {
        let cipher = MfaCipher::from_hex(KEY).expect("valid key");
        let user = Uuid::now_v7();
        let first = cipher.encrypt(user, b"same").expect("encrypts");
        let second = cipher.encrypt(user, b"same").expect("encrypts");

        assert_ne!(first.nonce, second.nonce);
    }

    #[test]
    fn verifies_codes_within_one_step_and_rejects_others() {
        let totp = generate_totp("teacher@example.com").expect("generates");
        let now = 1_800_000_000;

        assert!(totp.url.starts_with("otpauth://totp/Classia:"));
        assert_eq!(
            verify_code(&totp.raw_secret, &code_at(&totp.raw_secret, now), now).unwrap(),
            Some(now / 30)
        );
        assert!(
            verify_code(&totp.raw_secret, &code_at(&totp.raw_secret, now - 30), now)
                .unwrap()
                .is_some()
        );
        assert!(
            verify_code(&totp.raw_secret, &code_at(&totp.raw_secret, now - 90), now)
                .unwrap()
                .is_none()
        );
        assert!(
            verify_code(&totp.raw_secret, "12a456", now)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn code_format_is_exactly_six_digits() {
        assert!(is_six_digits("012345"));
        assert!(!is_six_digits("12345"));
        assert!(!is_six_digits("1234567"));
        assert!(!is_six_digits("12 456"));
    }
}
