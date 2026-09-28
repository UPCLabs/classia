use std::sync::Arc;

use uuid::Uuid;
use validator::Validate;

use crate::courses::{
    dto::{CourseCreateDto, CourseUpdateDto, EnrollStudentDto},
    error::{CourseError, EnrollmentError},
    models::{Course, CourseStatus, Enrollment},
    repository::{CourseRepository, EnrollOutcome},
};
use crate::users::{User, UserError, UserRole, UserService};
use crate::util::database::is_unique_violation;

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
        let course = self.find(course_id).await?;

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
        if !can_manage_courses(actor_role) {
            return Err(CourseError::Forbidden);
        }

        normalize_create(&mut body);
        validate(&body)?;

        if !actor_role.is_admin() && body.teacher_id != actor_id {
            return Err(CourseError::Forbidden);
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
        if !can_manage_courses(actor_role) {
            return Err(CourseError::Forbidden);
        }

        normalize_update(&mut body);
        validate(&body)?;

        let current = self.find(course_id).await?;

        if !actor_role.is_admin() && current.teacher_id != actor_id {
            return Err(CourseError::Forbidden);
        }

        if let Some(teacher_id) = body.teacher_id {
            if !actor_role.is_admin() && teacher_id != current.teacher_id {
                return Err(CourseError::Forbidden);
            }
            self.validate_teacher(teacher_id).await?;
        }

        if body
            .capacity
            .is_some_and(|capacity| i64::from(capacity) < current.enrolled_count)
        {
            return Err(CourseError::CapacityBelowEnrollment);
        }

        self.repository
            .update(course_id, body)
            .await
            .map_err(map_write_error)
    }

    async fn find(&self, course_id: Uuid) -> Result<Course, CourseError> {
        self.repository
            .get_by_id(course_id)
            .await
            .map_err(map_course_database_error)
    }

    async fn validate_teacher(&self, teacher_id: Uuid) -> Result<(), CourseError> {
        let teacher = self
            .user_service
            .get_user_by_id(teacher_id)
            .await
            .map_err(|error| match error {
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
        actor_id: Uuid,
        actor_role: &UserRole,
        course_id: Uuid,
    ) -> Result<Vec<User>, EnrollmentError> {
        if !actor_role.is_admin() && *actor_role != UserRole::Teacher {
            return Err(EnrollmentError::Forbidden);
        }

        self.course_service
            .get_for_user(course_id, actor_id, actor_role)
            .await
            .map_err(map_course_access_error)?;

        self.repository
            .list_enrolled_students(course_id)
            .await
            .map_err(EnrollmentError::Database)
    }

    pub async fn list_available_students(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        course_id: Uuid,
        search: Option<&str>,
    ) -> Result<Vec<User>, EnrollmentError> {
        self.ensure_responsible_teacher(actor_id, actor_role, course_id)
            .await?;

        let search = search.map(str::trim).filter(|search| !search.is_empty());

        self.repository
            .list_available_students(course_id, search)
            .await
            .map_err(EnrollmentError::Database)
    }

    pub async fn enroll_student(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        course_id: Uuid,
        body: EnrollStudentDto,
    ) -> Result<Enrollment, EnrollmentError> {
        let course = self
            .ensure_responsible_teacher(actor_id, actor_role, course_id)
            .await?;

        if course.status != CourseStatus::Active {
            return Err(EnrollmentError::CourseNotActive);
        }

        let student = self
            .user_service
            .get_user_by_id(body.student_id)
            .await
            .map_err(|error| match error {
                UserError::Database(error) => EnrollmentError::Database(error),
                _ => EnrollmentError::StudentNotFound,
            })?;

        if student.role != UserRole::Student {
            return Err(EnrollmentError::InvalidStudentRole);
        }
        if student.status != "active" {
            return Err(EnrollmentError::StudentInactive);
        }

        let enrolled_at = match self
            .repository
            .enroll_student(course_id, student.id)
            .await
            .map_err(|error| {
                if is_unique_violation(&error) {
                    EnrollmentError::AlreadyEnrolled
                } else {
                    EnrollmentError::Database(error)
                }
            })? {
            EnrollOutcome::Enrolled(enrolled_at) => enrolled_at,
            EnrollOutcome::CourseFull => return Err(EnrollmentError::CourseFull),
            EnrollOutcome::AlreadyEnrolled => return Err(EnrollmentError::AlreadyEnrolled),
        };

        Ok(Enrollment {
            course_id,
            student,
            enrolled_at,
        })
    }

    async fn ensure_responsible_teacher(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        course_id: Uuid,
    ) -> Result<Course, EnrollmentError> {
        if *actor_role != UserRole::Teacher {
            return Err(EnrollmentError::Forbidden);
        }

        self.course_service
            .get_for_user(course_id, actor_id, actor_role)
            .await
            .map_err(map_course_access_error)
    }
}

fn can_manage_courses(role: &UserRole) -> bool {
    role.is_admin() || *role == UserRole::Teacher
}

fn map_course_access_error(error: CourseError) -> EnrollmentError {
    match error {
        CourseError::Forbidden => EnrollmentError::Forbidden,
        CourseError::Database(error) => EnrollmentError::Database(error),
        _ => EnrollmentError::CourseNotFound,
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
