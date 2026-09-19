use uuid::Uuid;

use crate::users::{models::User, repository::UserRepository};

pub struct UserService {
    user_repository: UserRepository,
}

impl UserService {
    pub fn new(user_repository: UserRepository) -> Self {
        UserService { user_repository }
    }

    pub async fn get_user_by_id(&self, id_user: Uuid) -> Result<User, sqlx::Error> {
        self.user_repository.get_user_by_id(id_user).await
    }
}