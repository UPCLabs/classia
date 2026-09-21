use std::sync::Arc;

use crate::{courses::CourseService, users::UserService};

#[derive(Clone)]
pub(crate) struct ClassiaState {
    pub(crate) user_service: Arc<UserService>,
    pub(crate) course_service: Arc<CourseService>,
}

impl ClassiaState {
    pub(crate) fn new(user_service: Arc<UserService>, course_service: Arc<CourseService>) -> Self {
        Self {
            user_service,
            course_service,
        }
    }
}
