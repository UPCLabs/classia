use std::sync::Arc;

use chrono::Utc;
use uuid::Uuid;

use crate::activities::{
    error::ActivityError,
    models::{Activity, ActivityStatus},
    repository::ActivityRepository,
};
use crate::courses::{CourseError, CourseService, CourseStatus};
use crate::users::UserRole;

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
            .map_err(map_course_access_error)?;

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
            .map_err(map_course_access_error)?;

        match actor_role {
            UserRole::Student => self
                .repository
                .list_published_by_course(course_id)
                .await
                .map_err(ActivityError::Database),
            _ => Err(ActivityError::Forbidden),
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
            .map_err(map_course_access_error)?;

        match actor_role {
            UserRole::Student if activity.status == ActivityStatus::Draft => {
                Err(ActivityError::ActivityNotFound)
            }
            UserRole::Student => Ok(activity),
            _ => Err(ActivityError::Forbidden),
        }
    }

    async fn find(&self, activity_id: Uuid) -> Result<Activity, ActivityError> {
        self.repository
            .get_by_id(activity_id)
            .await
            .map_err(|error| match error {
                sqlx::Error::RowNotFound => ActivityError::ActivityNotFound,
                error => ActivityError::Database(error),
            })
    }
}

fn map_course_access_error(error: CourseError) -> ActivityError {
    match error {
        CourseError::Forbidden => ActivityError::Forbidden,
        CourseError::Database(error) => ActivityError::Database(error),
        _ => ActivityError::CourseNotFound,
    }
}
