use std::sync::Arc;

use crate::{
    auth::AuthService,
    courses::{CourseService, EnrollmentService},
    users::UserService,
};

#[derive(Clone)]
pub(crate) struct ClassiaState {
    pub(crate) user_service: Arc<UserService>,
    pub(crate) course_service: Arc<CourseService>,
    pub(crate) enrollment_service: Arc<EnrollmentService>,
    pub(crate) auth_service: Arc<AuthService>,
}

impl ClassiaState {
    pub(crate) fn new(
        user_service: Arc<UserService>,
        course_service: Arc<CourseService>,
        enrollment_service: Arc<EnrollmentService>,
        auth_service: Arc<AuthService>,
    ) -> Self {
        Self {
            user_service,
            course_service,
            enrollment_service,
            auth_service,
        }
    }
}
