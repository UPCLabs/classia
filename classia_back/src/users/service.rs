use uuid::Uuid;
use validator::Validate;

use crate::{
    users::{
        UserRole,
        dto::{ChangePasswordDto, UpdateUserDto, UserCreateDto, UserQueryParams},
        error::UserError,
        models::{User, UserStatus},
        repository::UserRepository,
    },
    util::{
        database::is_unique_violation,
        password::hash_password,
        validation::{normalize_email, validation_message},
    },
};

pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub async fn get_user_by_id(&self, id_user: Uuid) -> Result<User, UserError> {
        let consult = self
            .user_repository
            .get_user_by_id(id_user)
            .await
            .map_err(UserError::Database)?;

        match consult {
            Some(user) => Ok(user),
            None => Err(UserError::UserNotFound),
        }
    }

    pub async fn find_active_user(&self, id_user: Uuid) -> Result<Option<User>, UserError> {
        let user = self
            .user_repository
            .get_user_by_id(id_user)
            .await
            .map_err(UserError::Database)?;

        Ok(user.filter(|user| user.status == "active"))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, UserError> {
        let consult = self
            .user_repository
            .get_user_by_email(&normalize_email(email))
            .await
            .map_err(UserError::Database)?;

        match consult {
            Some(user) => Ok(user),
            None => Err(UserError::UserNotFound),
        }
    }

    pub async fn create_user(
        &self,
        body: UserCreateDto,
        caller_role: UserRole,
    ) -> Result<User, UserError> {
        body.validate()
            .map_err(|errors| UserError::ValidationError(validation_message(errors)))?;

        if !caller_role.is_admin() {
            return Err(UserError::Forbidden(
                "Only administrators can create users".into(),
            ));
        }

        if !caller_role.can_assign_role(&body.role) {
            return Err(UserError::Forbidden(
                "Un administrador no puede crear un Superadministrador".into(),
            ));
        }

        let email = normalize_email(&body.email);

        let exists = self
            .user_repository
            .exists_by_email(&email)
            .await
            .map_err(UserError::Database)?;

        if exists {
            return Err(UserError::EmailAlreadyExists);
        }

        let password_hash = hash_password(&body.password)
            .map_err(|error| UserError::InternalError(error.to_string()))?;

        self.user_repository
            .insert_user(&body.name, &email, &password_hash, body.role)
            .await
            .map_err(map_write_error)
    }

    pub async fn change_password(
        &self,
        id_user: Uuid,
        body: ChangePasswordDto,
    ) -> Result<(), UserError> {
        body.validate()
            .map_err(|errors| UserError::ValidationError(validation_message(errors)))?;

        let exists = self
            .user_repository
            .exists_by_id(id_user)
            .await
            .map_err(UserError::Database)?;

        if !exists {
            return Err(UserError::UserNotFound);
        }

        let new_password_hash = hash_password(&body.new_password)
            .map_err(|error| UserError::InternalError(error.to_string()))?;

        self.user_repository
            .update_password(id_user, &new_password_hash)
            .await
            .map_err(UserError::Database)
    }

    pub async fn update_user(
        &self,
        id_user: Uuid,
        body: UpdateUserDto,
        caller_role: UserRole,
    ) -> Result<User, UserError> {
        body.validate()
            .map_err(|errors| UserError::ValidationError(validation_message(errors)))?;

        let target = self
            .user_repository
            .get_user_by_id(id_user)
            .await
            .map_err(UserError::Database)?
            .ok_or(UserError::UserNotFound)?;

        let assigns_forbidden_role = body
            .role
            .as_ref()
            .is_some_and(|new_role| !caller_role.can_assign_role(new_role));

        if !caller_role.can_assign_role(&target.role) || assigns_forbidden_role {
            return Err(UserError::Forbidden(
                "Admins cannot grant or modify the SuperAdmin role".into(),
            ));
        }

        let email = body.email.as_deref().map(normalize_email);

        if let Some(email) = &email {
            let existing = self
                .user_repository
                .get_user_by_email(email)
                .await
                .map_err(UserError::Database)?;

            if existing.is_some_and(|existing| existing.id != id_user) {
                return Err(UserError::EmailAlreadyExists);
            }
        }

        self.user_repository
            .update_user(id_user, body.name, email, body.role)
            .await
            .map_err(map_write_error)
    }

    pub async fn update_user_status(
        &self,
        id_user: Uuid,
        status: UserStatus,
        caller_id: Uuid,
        caller_role: UserRole,
    ) -> Result<User, UserError> {
        if id_user == caller_id {
            return Err(UserError::ValidationError(
                "No puedes cambiar tu propio estado".into(),
            ));
        }

        let target = self.get_user_by_id(id_user).await?;

        if !caller_role.can_assign_role(&target.role) {
            return Err(UserError::Forbidden(
                "Admins cannot modify a SuperAdmin".into(),
            ));
        }

        self.user_repository
            .update_status(id_user, status.as_str())
            .await
            .map_err(map_write_error)
    }

    pub async fn list_users(&self, params: UserQueryParams) -> Result<Vec<User>, UserError> {
        let q = params
            .q
            .map(|q| q.trim().to_string())
            .filter(|q| !q.is_empty());

        let status = match params.status.as_deref() {
            Some("active") | Some("inactive") | None => params.status,
            Some(_) => {
                return Err(UserError::ValidationError(
                    "status: must be 'active' or 'inactive'".into(),
                ));
            }
        };

        self.user_repository
            .list_users(q, params.role, status)
            .await
            .map_err(UserError::Database)
    }
}

fn map_write_error(error: sqlx::Error) -> UserError {
    match error {
        sqlx::Error::RowNotFound => UserError::UserNotFound,
        error if is_unique_violation(&error) => UserError::EmailAlreadyExists,
        error => UserError::Database(error),
    }
}
