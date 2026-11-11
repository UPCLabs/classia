use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "audit_result", rename_all = "lowercase")]
pub enum AuditResult {
    Success,
    Denied,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditTarget {
    Delivery,
    Evaluation,
    User,
}

impl AuditTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Delivery => "delivery",
            Self::Evaluation => "evaluation",
            Self::User => "user",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(
    clippy::enum_variant_names,
    reason = "variants mirror the SDS Table 12.10 action names"
)]
pub enum AuditAction {
    SubmitDelivery,
    ReplaceDelivery,
    GradeDelivery,
    MfaSetupStarted,
    MfaActivated,
    MfaChallenge,
}

impl AuditAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SubmitDelivery => "submit_delivery",
            Self::ReplaceDelivery => "replace_delivery",
            Self::GradeDelivery => "grade_delivery",
            Self::MfaSetupStarted => "mfa.setup_started",
            Self::MfaActivated => "mfa.activated",
            Self::MfaChallenge => "mfa.challenge",
        }
    }
}

/// `details` must never contain secrets, file paths or file contents.
#[derive(Debug)]
pub struct NewAuditEvent {
    pub actor_user_id: Option<Uuid>,
    pub target: AuditTarget,
    pub target_id: Option<Uuid>,
    pub action: AuditAction,
    pub result: AuditResult,
    pub details: Value,
}
