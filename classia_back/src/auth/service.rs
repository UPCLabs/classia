use crate::auth::jwt::JwtService;

pub struct AuthService {
    jwt_service: JwtService,
}

impl AuthService {
    pub fn new(jwt_service: JwtService) -> Self {
        Self { jwt_service }
    }
}
