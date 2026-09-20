use std::sync::Arc;

use crate::users::UserService;

#[derive(Clone)]
pub(crate) struct ClassiaState {
    pub(crate) user_service: Arc<UserService>,
}

impl ClassiaState {
    pub(crate) fn new(user_service: Arc<UserService>) -> Self {
        Self { user_service }
    }
}
