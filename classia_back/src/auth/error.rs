pub enum AuthError {
    JwtError(String),
    InternalError(String),
    InvalidCredentials,
    InactiveUser,
}
