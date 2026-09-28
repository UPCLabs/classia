use std::sync::Arc;

use uuid::Uuid;
use validator::Validate;

use crate::courses::{
    dto::{CourseCreateDto, CourseUpdateDto, EnrollStudentDto},
    error::{CourseError, EnrollmentError},
    models::{Course, CourseStatus, Enrollment},
    repository::CourseRepository,
};
use crate::users::{UserError, UserRole, UserService};

pub struct CourseService {
    repository: CourseRepository,
    user_service: Arc<UserService>,
}

impl CourseService {
    pub fn new(repository: CourseRepository, user_service: Arc<UserService>) -> Self {
        Self {
            repository,
            user_service,
        }
    }

    pub async fn list_for_user(
        &self,
        user_id: Uuid,
        role: &UserRole,
    ) -> Result<Vec<Course>, CourseError> {
        match role {
            UserRole::SuperAdmin | UserRole::Admin => self.repository.list_all().await,
            UserRole::Teacher => self.repository.list_by_teacher(user_id).await,
            UserRole::Student => self.repository.list_by_student(user_id).await,
        }
        .map_err(CourseError::Database)
    }

    pub async fn get_for_user(
        &self,
        course_id: Uuid,
        user_id: Uuid,
        role: &UserRole,
    ) -> Result<Course, CourseError> {
        let course = self
            .repository
            .get_by_id(course_id)
            .await
            .map_err(map_course_database_error)?;

        let allowed = match role {
            UserRole::SuperAdmin | UserRole::Admin => true,
            UserRole::Teacher => course.teacher_id == user_id,
            UserRole::Student => self
                .repository
                .is_enrolled(course_id, user_id)
                .await
                .map_err(CourseError::Database)?,
        };

        if !allowed {
            return Err(CourseError::Forbidden);
        }

        Ok(course)
    }

    pub async fn create_course(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        mut body: CourseCreateDto,
    ) -> Result<Course, CourseError> {
        normalize_create(&mut body);
        validate(&body)?;

        if !actor_role.is_admin() {
            body.teacher_id = actor_id;
        }

        self.validate_teacher(body.teacher_id).await?;

        self.repository.create(body).await.map_err(map_write_error)
    }

    pub async fn update_course(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        course_id: Uuid,
        mut body: CourseUpdateDto,
    ) -> Result<Course, CourseError> {
        normalize_update(&mut body);
        validate(&body)?;

        let current = self
            .repository
            .get_by_id(course_id)
            .await
            .map_err(map_course_database_error)?;

        if !actor_role.is_admin() && current.teacher_id != actor_id {
            return Err(CourseError::Forbidden);
        }

        if body.teacher_id.is_some() && !actor_role.is_admin() {
            return Err(CourseError::Forbidden);
        }

        if let Some(teacher_id) = body.teacher_id {
            self.validate_teacher(teacher_id).await?;
        }

        if let Some(capacity) = body.capacity {
            if i64::from(capacity) < current.enrolled_count {
                return Err(CourseError::CapacityBelowEnrollment);
            }
        }

        self.repository.update(course_id, body).await.map_err(map_write_error)
    }

    async fn validate_teacher(&self, teacher_id: Uuid) -> Result<(), CourseError> {
        let teacher = self
            .user_service
            .get_user_by_id(teacher_id)
            .await
            .map_err(|error| match error {
                UserError::UserNotFound => CourseError::TeacherNotFound,
                UserError::Database(error) => CourseError::Database(error),
                _ => CourseError::TeacherNotFound,
            })?;

        if teacher.role != UserRole::Teacher {
            return Err(CourseError::InvalidTeacherRole);
        }
        if teacher.status != "active" {
            return Err(CourseError::TeacherInactive);
        }

        Ok(())
    }
}

pub struct EnrollmentService {
    repository: CourseRepository,
    user_service: Arc<UserService>,
    course_service: Arc<CourseService>,
}

impl EnrollmentService {
    pub fn new(
        repository: CourseRepository,
        user_service: Arc<UserService>,
        course_service: Arc<CourseService>,
    ) -> Self {
        Self {
            repository,
            user_service,
            course_service,
        }
    }

    pub async fn list_students(
        &self,
        teacher_id: Uuid,
        course_id: Uuid,
    ) -> Result<Vec<Enrollment>, EnrollmentError> {
        self.ensure_owner(teacher_id, course_id).await?;
        self.repository
            .list_enrolled_students(course_id)
            .await
            .map_err(EnrollmentError::Database)
    }

    pub async fn list_available_students(
        &self,
        teacher_id: Uuid,
        course_id: Uuid,
        search: Option<&str>,
    ) -> Result<Vec<(Uuid, String, String)>, EnrollmentError> {
        self.ensure_owner(teacher_id, course_id).await?;
        self.repository
            .list_available_students(course_id, search)
            .await
            .map_err(EnrollmentError::Database)
    }

    pub async fn register_student(
        &self,
        teacher_id: Uuid,
        course_id: Uuid,
        body: EnrollStudentDto,
    ) -> Result<(), EnrollmentError> {
        let course = self.ensure_owner(teacher_id, course_id).await?;

        if course.status != CourseStatus::Active {
            return Err(EnrollmentError::CourseNotActive);
        }

        let student = self
            .user_service
            .get_user_by_id(body.student_id)
            .await
            .map_err(|error| match error {
                UserError::UserNotFound => EnrollmentError::StudentNotFound,
                UserError::Database(error) => EnrollmentError::Database(error),
                _ => EnrollmentError::StudentNotFound,
            })?;

        if student.role != UserRole::Student {
            return Err(EnrollmentError::StudentMustHaveStudentRole);
        }
        if student.status != "active" {
            return Err(EnrollmentError::StudentInactive);
        }
        if self
            .repository
            .is_enrolled(course_id, body.student_id)
            .await
            .map_err(EnrollmentError::Database)?
        {
            return Err(EnrollmentError::AlreadyEnrolled);
        }
        if course.enrolled_count >= i64::from(course.capacity) {
            return Err(EnrollmentError::CourseFull);
        }

        self.repository
            .enroll_student(course_id, &body)
            .await
            .map_err(|error| {
                if is_unique_violation(&error) {
                    EnrollmentError::AlreadyEnrolled
                } else {
                    EnrollmentError::Database(error)
                }
            })
    }

    async fn ensure_owner(
        &self,
        teacher_id: Uuid,
        course_id: Uuid,
    ) -> Result<Course, EnrollmentError> {
        let course = self
            .course_service
            .get_for_user(course_id, teacher_id, &UserRole::Teacher)
            .await
            .map_err(|error| match error {
                CourseError::CourseNotFound => EnrollmentError::CourseNotFound,
                CourseError::Forbidden => EnrollmentError::TeacherDoesNotOwnCourse,
                CourseError::Database(error) => EnrollmentError::Database(error),
                _ => EnrollmentError::CourseNotFound,
            })?;

        Ok(course)
    }
}

fn normalize_create(body: &mut CourseCreateDto) {
    body.name = body.name.trim().to_string();
    body.code = body.code.trim().to_uppercase();
}

fn normalize_update(body: &mut CourseUpdateDto) {
    body.name = body.name.take().map(|value| value.trim().to_string());
    body.code = body.code.take().map(|value| value.trim().to_uppercase());
}

fn validate<T: Validate>(body: &T) -> Result<(), CourseError> {
    body.validate().map_err(|errors| {
        let message = errors
            .field_errors()
            .values()
            .next()
            .and_then(|field_errors| field_errors.first())
            .and_then(|error| error.message.clone())
            .map(|message| message.to_string())
            .unwrap_or_else(|| "Datos inválidos".to_string());
        CourseError::ValidationError(message)
    })
}

fn map_course_database_error(error: sqlx::Error) -> CourseError {
    match error {
        sqlx::Error::RowNotFound => CourseError::CourseNotFound,
        error => CourseError::Database(error),
    }
}

fn map_write_error(error: sqlx::Error) -> CourseError {
    if is_unique_violation(&error) {
        CourseError::DuplicateCode
    } else {
        map_course_database_error(error)
    }
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(
        error,
        sqlx::Error::Database(database_error)
            if database_error.code().as_deref() == Some("23505")
    )
}
