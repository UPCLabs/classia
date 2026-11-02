use super::{
    dto::{ActivityCreateDto, ActivityUpdateDto},
    error::ActivityError,
    models::{Activity, ActivityStatus},
    repository::ActivityRepository,
};
use crate::{
    courses::{CourseService, CourseStatus},
    users::UserRole,
};
use chrono::Utc;
use std::sync::Arc;
use uuid::Uuid;

pub struct ActivityService {
    repository: ActivityRepository,
    course_service: Arc<CourseService>,
}
impl ActivityService {
    pub fn new(repository: ActivityRepository, course_service: Arc<CourseService>) -> Self {
        Self {
            repository,
            course_service,
        }
    }
    pub async fn create(
        &self,
        actor: Uuid,
        role: &UserRole,
        course_id: Uuid,
        mut body: ActivityCreateDto,
    ) -> Result<Activity, ActivityError> {
        let course = self
            .course_service
            .get_for_user(course_id, actor, role)
            .await
            .map_err(ActivityError::from)?;
        if *role != UserRole::Teacher {
            return Err(ActivityError::Forbidden);
        }
        if course.status != CourseStatus::Active {
            return Err(ActivityError::CourseNotActive);
        }
        normalize_create(&mut body);
        validate_fields(&body.title, &body.description)?;
        if body.due_at <= Utc::now() {
            return Err(ActivityError::DueDateNotFuture);
        }
        self.repository
            .create(course_id, body)
            .await
            .map_err(ActivityError::Database)
    }
    pub async fn update(
        &self,
        actor: Uuid,
        role: &UserRole,
        id: Uuid,
        mut body: ActivityUpdateDto,
    ) -> Result<Activity, ActivityError> {
        let current = self.find(id).await?;
        let course = self
            .course_service
            .get_for_user(current.course_id, actor, role)
            .await
            .map_err(ActivityError::from)?;
        if *role != UserRole::Teacher {
            return Err(ActivityError::Forbidden);
        }
        if course.status != CourseStatus::Active {
            return Err(ActivityError::CourseNotActive);
        }
        normalize_update(&mut body);
        if let Some(title) = &body.title
            && (title.is_empty() || title.chars().count() > 150)
        {
            return Err(ActivityError::Validation(
                "El título debe tener entre 1 y 150 caracteres".into(),
            ));
        }
        if let Some(description) = &body.description
            && (description.is_empty() || description.chars().count() > 5000)
        {
            return Err(ActivityError::Validation(
                "La descripción debe tener entre 1 y 5000 caracteres".into(),
            ));
        }
        if let Some(due_at) = body.due_at
            && due_at <= Utc::now()
        {
            return Err(ActivityError::DueDateNotFuture);
        }
        let unchanged = body.title.as_ref().is_none_or(|v| v == &current.title)
            && body
                .description
                .as_ref()
                .is_none_or(|v| v == &current.description)
            && body.due_at.is_none_or(|v| v == current.due_at);
        if unchanged {
            return Err(ActivityError::NoChanges);
        }
        self.repository.update(id, body).await.map_err(map_database)
    }

    pub async fn publish(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        activity_id: Uuid,
    ) -> Result<Activity, ActivityError> {
        if *actor_role != UserRole::Teacher {
            return Err(ActivityError::Forbidden);
        }

        let activity = self.find(activity_id).await?;

        let course = self
            .course_service
            .get_for_user(activity.course_id, actor_id, actor_role)
            .await
            .map_err(ActivityError::from)?;

        if course.status != CourseStatus::Active {
            return Err(ActivityError::CourseNotActive);
        }
        if activity.status == ActivityStatus::Published {
            return Err(ActivityError::AlreadyPublished);
        }
        if activity.due_at <= Utc::now() {
            return Err(ActivityError::DueDateNotFuture);
        }

        self.repository
            .publish(activity_id)
            .await
            .map_err(ActivityError::Database)?
            .ok_or(ActivityError::AlreadyPublished)
    }

    pub async fn list_for_user(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        course_id: Uuid,
    ) -> Result<Vec<Activity>, ActivityError> {
        self.course_service
            .get_for_user(course_id, actor_id, actor_role)
            .await
            .map_err(ActivityError::from)?;

        match actor_role {
            UserRole::Student => self
                .repository
                .list_published_by_course(course_id)
                .await
                .map_err(ActivityError::Database),
            _ => self
                .repository
                .list_activities_by_course(course_id)
                .await
                .map_err(ActivityError::Database),
        }
    }

    pub async fn get_for_user(
        &self,
        actor_id: Uuid,
        actor_role: &UserRole,
        activity_id: Uuid,
    ) -> Result<Activity, ActivityError> {
        let activity = self.find(activity_id).await?;

        self.course_service
            .get_for_user(activity.course_id, actor_id, actor_role)
            .await
            .map_err(ActivityError::from)?;

        match actor_role {
            UserRole::Student => {
                if activity.status == ActivityStatus::Draft {
                    return Err(ActivityError::ActivityNotFound);
                }

                Ok(activity)
            }
            _ => Ok(activity),
        }
    }

    async fn find(&self, activity_id: Uuid) -> Result<Activity, ActivityError> {
        self.repository.get(activity_id).await.map_err(map_database)
    }
}
fn normalize_create(body: &mut ActivityCreateDto) {
    body.title = body.title.trim().to_string();
    body.description = body.description.trim().to_string();
}
fn normalize_update(body: &mut ActivityUpdateDto) {
    body.title = body.title.take().map(|v| v.trim().to_string());
    body.description = body.description.take().map(|v| v.trim().to_string());
}
fn validate_fields(title: &str, description: &str) -> Result<(), ActivityError> {
    if title.is_empty() || title.chars().count() > 150 {
        return Err(ActivityError::Validation(
            "El título debe tener entre 1 y 150 caracteres".into(),
        ));
    }
    if description.is_empty() || description.chars().count() > 5000 {
        return Err(ActivityError::Validation(
            "La descripción debe tener entre 1 y 5000 caracteres".into(),
        ));
    }
    Ok(())
}
fn map_database(error: sqlx::Error) -> ActivityError {
    if matches!(error, sqlx::Error::RowNotFound) {
        ActivityError::ActivityNotFound
    } else {
        ActivityError::Database(error)
    }
}
