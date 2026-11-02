use super::{dto::ActivityUpdateDto, error::ActivityError};
use chrono::Utc;

#[test]
fn activity_error_codes_are_contract_codes() {
    let error: crate::error::AppError = ActivityError::NoChanges.into();
    assert_eq!(error.code, "NO_CHANGES");
    assert!(Utc::now() < Utc::now() + chrono::Duration::seconds(1));
    let _ = ActivityUpdateDto {
        title: None,
        description: None,
        due_at: None,
    };
}
