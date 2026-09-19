#[derive(Debug)]
pub enum UserError {
    UserNotFound,
    Database(sqlx::Error),
}

impl From<UserError> for String {
    fn from(value: UserError) -> Self {
        match value {
            UserError::UserNotFound => format!("Usuario no encontrado"),
            UserError::Database(error) => format!("Database error: {}", error),
        }
    }
}
