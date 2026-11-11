use std::sync::Arc;

use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::{PgExecutor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    audit::{
        models::{AuditAction, AuditResult, AuditTarget, NewAuditEvent},
        repository as audit,
    },
    auth::{
        claims::{Claims, TokenPurpose},
        dto::LoginRequest,
        error::AuthError,
        jwt::JwtService,
        mfa::{self, MfaCipher},
        mfa_repository::{self as repository, MfaState, MfaStatus},
    },
    users::UserService,
    util::password::validate_password,
};

/// Lifetime of the partial session between password and code (SDS v2.2 §14.3).
pub const MFA_SESSION_SECS: i64 = 300;

pub enum LoginOutcome {
    Authenticated(String),
    MfaRequired(String),
}

pub struct MfaSetupResult {
    pub secret: String,
    pub url: String,
}

/// Why the code is being checked: finishing a login or activating a pending setup.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChallengeMode {
    Login,
    Activation,
}

pub struct AuthService {
    user_service: Arc<UserService>,
    jwt_service: JwtService,
    pool: PgPool,
    cipher: Option<MfaCipher>,
}

impl AuthService {
    pub fn new(
        jwt_service: JwtService,
        user_service: Arc<UserService>,
        pool: PgPool,
        cipher: Option<MfaCipher>,
    ) -> Self {
        Self {
            jwt_service,
            user_service,
            pool,
            cipher,
        }
    }

    pub async fn login(&self, login: LoginRequest) -> Result<LoginOutcome, AuthError> {
        let user = self
            .user_service
            .get_user_by_email(&login.email)
            .await
            .map_err(|_| AuthError::InvalidCredentials)?;

        if !validate_password(&login.password, &user.password)
            .map_err(|e| AuthError::InternalError(e.to_string()))?
        {
            return Err(AuthError::InvalidCredentials);
        }

        if user.status != "active" {
            return Err(AuthError::InactiveUser);
        }

        let mfa_status = repository::status(&self.pool, user.id)
            .await
            .map_err(database_error)?;

        if mfa_status == Some(MfaStatus::Active) {
            let token =
                self.jwt_service
                    .generate_for(&user, TokenPurpose::MfaPending, MFA_SESSION_SECS)?;
            return Ok(LoginOutcome::MfaRequired(token));
        }

        Ok(LoginOutcome::Authenticated(
            self.jwt_service.generate(&user)?,
        ))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        self.jwt_service.validate(token)
    }

    pub async fn setup_mfa(&self, user_id: Uuid) -> Result<MfaSetupResult, AuthError> {
        let cipher = self.cipher()?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let state = repository::lock_active_user(&mut transaction, user_id)
            .await
            .map_err(database_error)?
            .ok_or(AuthError::InactiveUser)?;

        if state.status == MfaStatus::Active {
            return self
                .deny(
                    transaction,
                    user_id,
                    AuditAction::MfaSetupStarted,
                    AuthError::MfaAlreadyEnabled,
                    "already_enabled",
                )
                .await;
        }
        if let Some(error) = lock_error(&state) {
            return self
                .deny(
                    transaction,
                    user_id,
                    AuditAction::MfaSetupStarted,
                    error,
                    "locked",
                )
                .await;
        }

        let totp = mfa::generate_totp(&state.email)?;
        let encrypted = cipher.encrypt(user_id, &totp.raw_secret)?;

        repository::start_setup(
            &mut transaction,
            user_id,
            &encrypted.ciphertext,
            &encrypted.nonce,
        )
        .await
        .map_err(database_error)?;
        record_event(
            &mut *transaction,
            user_id,
            AuditAction::MfaSetupStarted,
            AuditResult::Success,
            None,
        )
        .await?;
        transaction.commit().await.map_err(database_error)?;

        Ok(MfaSetupResult {
            secret: totp.secret_base32,
            url: totp.url,
        })
    }

    /// Verifies a code. On success during login, returns the full session token.
    pub async fn challenge_mfa(
        &self,
        user_id: Uuid,
        code: &str,
        mode: ChallengeMode,
    ) -> Result<Option<String>, AuthError> {
        self.challenge_mfa_at(user_id, code, mode, Utc::now().timestamp() as u64)
            .await
    }

    pub(crate) async fn challenge_mfa_at(
        &self,
        user_id: Uuid,
        code: &str,
        mode: ChallengeMode,
        unix_time: u64,
    ) -> Result<Option<String>, AuthError> {
        let cipher = self.cipher()?;
        let mut transaction = self.pool.begin().await.map_err(database_error)?;

        let state = repository::lock_active_user(&mut transaction, user_id)
            .await
            .map_err(database_error)?
            .ok_or(AuthError::MfaSessionExpired)?;

        let expected_status = match mode {
            ChallengeMode::Login => MfaStatus::Active,
            ChallengeMode::Activation => MfaStatus::Pending,
        };
        if state.status != expected_status {
            let error = match mode {
                ChallengeMode::Login => AuthError::MfaSessionExpired,
                ChallengeMode::Activation if state.status == MfaStatus::Active => {
                    AuthError::MfaAlreadyEnabled
                }
                ChallengeMode::Activation => AuthError::MfaNotPending,
            };
            return self
                .deny(
                    transaction,
                    user_id,
                    AuditAction::MfaChallenge,
                    error,
                    "invalid_state",
                )
                .await;
        }

        let now = Utc::now();
        if let Some(error) = lock_error(&state) {
            return self
                .deny(
                    transaction,
                    user_id,
                    AuditAction::MfaChallenge,
                    error,
                    "locked",
                )
                .await;
        }

        let secret = match decrypt_state(cipher, user_id, &state) {
            Ok(secret) => secret,
            Err(error) => {
                transaction.rollback().await.map_err(database_error)?;
                record_event(
                    &self.pool,
                    user_id,
                    AuditAction::MfaChallenge,
                    AuditResult::Failure,
                    Some("verification_unavailable"),
                )
                .await?;
                return Err(error);
            }
        };

        match mfa::verify_code(&secret, code, unix_time)? {
            Some(step) if state.last_step.is_some_and(|last| step as i64 <= last) => {
                self.deny(
                    transaction,
                    user_id,
                    AuditAction::MfaChallenge,
                    AuthError::MfaCodeReused,
                    "code_reused",
                )
                .await
            }
            Some(step) => {
                repository::record_success(
                    &mut transaction,
                    user_id,
                    step as i64,
                    mode == ChallengeMode::Activation,
                )
                .await
                .map_err(database_error)?;
                let action = match mode {
                    ChallengeMode::Activation => AuditAction::MfaActivated,
                    ChallengeMode::Login => AuditAction::MfaChallenge,
                };
                record_event(
                    &mut *transaction,
                    user_id,
                    action,
                    AuditResult::Success,
                    None,
                )
                .await?;
                transaction.commit().await.map_err(database_error)?;

                if mode == ChallengeMode::Activation {
                    return Ok(None);
                }

                let user = self
                    .user_service
                    .get_user_by_id(user_id)
                    .await
                    .map_err(|_| AuthError::MfaSessionExpired)?;
                Ok(Some(self.jwt_service.generate(&user)?))
            }
            None => {
                let failed_attempts = state.failed_attempts + 1;
                let locked_until = (failed_attempts >= mfa::MAX_FAILED_ATTEMPTS)
                    .then(|| now + Duration::minutes(mfa::LOCK_MINUTES));
                let stored_attempts = if locked_until.is_some() {
                    0
                } else {
                    failed_attempts
                };

                repository::record_failure(
                    &mut transaction,
                    user_id,
                    stored_attempts,
                    locked_until,
                )
                .await
                .map_err(database_error)?;
                transaction.commit().await.map_err(database_error)?;
                // The security counter must persist even though authentication
                // failed. Record the failure outside the business transaction.
                record_event(
                    &self.pool,
                    user_id,
                    AuditAction::MfaChallenge,
                    AuditResult::Failure,
                    Some("invalid_code"),
                )
                .await?;

                Err(match locked_until {
                    Some(_) => AuthError::MfaLocked {
                        retry_after_secs: mfa::LOCK_MINUTES * 60,
                    },
                    None => AuthError::MfaCodeInvalid,
                })
            }
        }
    }

    fn cipher(&self) -> Result<&MfaCipher, AuthError> {
        self.cipher.as_ref().ok_or_else(|| {
            AuthError::InternalError("MFA_ENCRYPTION_KEY is not configured".to_string())
        })
    }

    async fn deny<T>(
        &self,
        transaction: Transaction<'_, Postgres>,
        user_id: Uuid,
        action: AuditAction,
        error: AuthError,
        reason: &'static str,
    ) -> Result<T, AuthError> {
        transaction.rollback().await.map_err(database_error)?;
        record_event(
            &self.pool,
            user_id,
            action,
            AuditResult::Denied,
            Some(reason),
        )
        .await?;
        Err(error)
    }
}

fn lock_error(state: &MfaState) -> Option<AuthError> {
    let remaining = state.locked_until? - Utc::now();
    (remaining > Duration::zero()).then(|| AuthError::MfaLocked {
        retry_after_secs: ((remaining.num_milliseconds() + 999) / 1000).max(1),
    })
}

async fn record_event<'e>(
    executor: impl PgExecutor<'e>,
    user_id: Uuid,
    action: AuditAction,
    result: AuditResult,
    reason: Option<&'static str>,
) -> Result<(), AuthError> {
    audit::record(
        executor,
        &NewAuditEvent {
            actor_user_id: Some(user_id),
            target: AuditTarget::User,
            target_id: Some(user_id),
            action,
            result,
            details: reason.map_or_else(|| json!({}), |reason| json!({ "reason": reason })),
        },
    )
    .await
    .map_err(database_error)
}

fn decrypt_state(
    cipher: &MfaCipher,
    user_id: Uuid,
    state: &MfaState,
) -> Result<Vec<u8>, AuthError> {
    match (&state.ciphertext, &state.nonce) {
        (Some(ciphertext), Some(nonce)) => cipher.decrypt(user_id, ciphertext, nonce),
        _ => Err(AuthError::InternalError(
            "MFA secret is missing".to_string(),
        )),
    }
}

fn database_error(error: sqlx::Error) -> AuthError {
    AuthError::InternalError(format!("Database error: {error}"))
}
