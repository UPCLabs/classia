use crate::error::AppError;
use axum::http::StatusCode;
use tracing::error;

#[derive(Debug)]
pub enum AuthError {
    JwtError(String),
    InternalError(String),
    InvalidCredentials,
    InactiveUser,
    MfaCodeInvalid,
    MfaCodeReused,
    MfaSessionExpired,
    MfaAlreadyEnabled,
    MfaNotPending,
    MfaLocked { retry_after_secs: i64 },
}

impl From<AuthError> for AppError {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::JwtError(reason) => {
                error!(error = &reason, "JWT error");
                AppError {
                    status: StatusCode::UNAUTHORIZED,
                    code: StatusCode::UNAUTHORIZED.as_str(),
                    message: reason,
                }
            }
            AuthError::InternalError(reason) => {
                error!(error = &reason, "Internal error");

                AppError {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR",
                    message: "No se pudo completar la operación. Inténtalo de nuevo.".to_string(),
                }
            }
            AuthError::InvalidCredentials => {
                error!("Invalid credentials");

                AppError {
                    status: StatusCode::UNAUTHORIZED,
                    code: StatusCode::UNAUTHORIZED.as_str(),
                    message: "Credenciales inválidas".to_string(),
                }
            }
            AuthError::InactiveUser => AppError {
                status: StatusCode::BAD_REQUEST,
                code: StatusCode::BAD_REQUEST.as_str(),
                message: "Usuario no disponible".to_string(),
            },
            AuthError::MfaCodeInvalid => AppError::new(
                StatusCode::BAD_REQUEST,
                "MFA_CODE_INVALID",
                "Código inválido o vencido. Inténtalo de nuevo.",
            ),
            AuthError::MfaCodeReused => AppError::new(
                StatusCode::BAD_REQUEST,
                "MFA_CODE_REUSED",
                "Este código ya fue usado. Espera el siguiente código.",
            ),
            AuthError::MfaSessionExpired => AppError::new(
                StatusCode::UNAUTHORIZED,
                "MFA_SESSION_EXPIRED",
                "La verificación expiró. Inicia sesión de nuevo.",
            ),
            AuthError::MfaAlreadyEnabled => AppError::new(
                StatusCode::CONFLICT,
                "MFA_ALREADY_ENABLED",
                "La verificación en dos pasos ya está activa en esta cuenta.",
            ),
            AuthError::MfaNotPending => AppError::new(
                StatusCode::CONFLICT,
                "MFA_NOT_ENABLED",
                "No hay una configuración de MFA pendiente. Comienza la configuración.",
            ),
            AuthError::MfaLocked { retry_after_secs } => AppError::new(
                StatusCode::TOO_MANY_REQUESTS,
                "MFA_LOCKED",
                format!(
                    "Demasiados intentos fallidos. Inténtalo de nuevo en {} minutos.",
                    (retry_after_secs + 59) / 60
                ),
            ),
        }
    }
}
