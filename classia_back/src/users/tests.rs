use axum::http::StatusCode;
use serde_json::json;
use validator::Validate;

use super::{
    dto::{ChangePasswordDto, UpdateUserDto, UpdateUserStatusDto, UserCreateDto},
    error::UserError,
    models::{UserRole, UserStatus},
};
use crate::error::AppError;

#[test]
fn user_roles_enforce_create_user_permission() {
    assert!(UserRole::SuperAdmin.is_admin());
    assert!(UserRole::Admin.is_admin());
    assert!(!UserRole::Teacher.is_admin());
    assert!(!UserRole::Student.is_admin());
}

#[test]
fn user_create_deserialization_trims_and_validates_fields() {
    let user: UserCreateDto = serde_json::from_value(json!({
        "name": "  Ada Lovelace  ",
        "email": "  ada@example.com  ",
        "password": "password123",
        "role": "Teacher"
    }))
    .expect("valid user payload");

    assert_eq!(user.name, "Ada Lovelace");
    assert_eq!(user.email, "ada@example.com");
    assert!(user.validate().is_ok());
}

#[test]
fn invalid_user_fields_are_rejected() {
    let invalid_email: UserCreateDto = serde_json::from_value(json!({
        "name": "Ada",
        "email": "not-an-email",
        "password": "password123",
        "role": "Teacher"
    }))
    .expect("payload deserializes before validation");
    let short_password = ChangePasswordDto {
        new_password: "short".to_string(),
    };

    assert!(invalid_email.validate().is_err());
    assert!(short_password.validate().is_err());
}

#[test]
fn optional_user_updates_are_trimmed() {
    let update: UpdateUserDto = serde_json::from_value(json!({
        "name": "  Grace Hopper  ",
        "email": null,
        "role": "Admin"
    }))
    .expect("valid update payload");

    assert_eq!(update.name.as_deref(), Some("Grace Hopper"));
    assert_eq!(update.email, None);
    assert_eq!(update.role, Some(UserRole::Admin));
}

#[test]
fn user_errors_map_to_stable_http_statuses() {
    let not_found: AppError = UserError::UserNotFound.into();
    let duplicate: AppError = UserError::EmailAlreadyExists.into();
    let invalid: AppError = UserError::ValidationError("invalid".to_string()).into();

    assert_eq!(not_found.status, StatusCode::NOT_FOUND);
    assert_eq!(duplicate.status, StatusCode::CONFLICT);
    assert_eq!(invalid.status, StatusCode::BAD_REQUEST);
}

#[test]
fn user_status_accepts_only_known_values() {
    let active: UpdateUserStatusDto =
        serde_json::from_value(json!({ "status": "active" })).expect("active is valid");
    let inactive: UpdateUserStatusDto =
        serde_json::from_value(json!({ "status": "inactive" })).expect("inactive is valid");

    assert_eq!(active.status, UserStatus::Active);
    assert_eq!(inactive.status.as_str(), "inactive");
    assert!(serde_json::from_value::<UpdateUserStatusDto>(json!({ "status": "deleted" })).is_err());
}

#[test]
fn admins_cannot_assign_or_modify_super_admin() {
    assert!(UserRole::SuperAdmin.can_assign_role(&UserRole::SuperAdmin));
    assert!(UserRole::Admin.can_assign_role(&UserRole::Teacher));
    assert!(!UserRole::Admin.can_assign_role(&UserRole::SuperAdmin));
}

#[test]
fn partial_user_updates_accept_missing_fields() {
    let update: UpdateUserDto =
        serde_json::from_value(json!({ "name": "Only Name" })).expect("partial update payload");

    assert_eq!(update.name.as_deref(), Some("Only Name"));
    assert_eq!(update.email, None);
    assert_eq!(update.role, None);
}
