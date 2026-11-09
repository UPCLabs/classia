use totp_rs::Secret;

use crate::auth::error::AuthError;

pub struct TotpSetup {
    pub totp: String,
    pub url: String,
}

pub fn generate_otp(email: &str) -> Result<TotpSetup, AuthError> {
    let totp = totp_rs::Builder::new()
        .with_issuer(Some("Classia"))
        .with_account_name(email)
        .build()
        .map_err(|e| AuthError::InternalError(e.to_string()))?;

    let url = totp
        .to_url()
        .map_err(|e| AuthError::InternalError(e.to_string()))?;

    Ok(TotpSetup {
        totp: totp.secret().to_base32(),
        url: url,
    })
}

pub fn validate_otp(otp: &str, secret: &str) -> bool {
    let Ok(secret) = Secret::try_from_base32(secret) else {
        return false;
    };

    let Ok(totp) = totp_rs::Builder::new().with_secret(secret).build() else {
        return false;
    };

    totp.check_current(otp).is_some()
}
