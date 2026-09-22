use std::sync::Arc;

use crate::{auth::AuthService, courses::CourseService, users::UserService};

#[derive(Clone)]
pub(crate) struct ClassiaState {
    pub(crate) user_service: Arc<UserService>,
    pub(crate) course_service: Arc<CourseService>,
    pub(crate) auth_service: Arc<AuthService>,
}

impl ClassiaState {
    pub(crate) fn new(
        user_service: Arc<UserService>,
        course_service: Arc<CourseService>,
        auth_service: Arc<AuthService>,
    ) -> Self {
        Self {
            user_service,
            course_service,
            auth_service,
        }
    }
}
