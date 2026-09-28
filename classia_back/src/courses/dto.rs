use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::courses::models::{Course, CourseStatus, Enrollment};

#[derive(Debug, Deserialize, Validate)]
pub struct CourseCreateDto {
    #[validate(length(
        min = 1,
        max = 150,
        message = "El nombre debe tener entre 1 y 150 caracteres"
    ))]
    pub name: String,

    #[validate(length(
        min = 1,
        max = 6,
        message = "El código debe tener entre 1 y 6 caracteres"
    ))]
    pub code: String,

    pub teacher_id: Uuid,
    pub status: CourseStatus,

    #[validate(range(min = 1, message = "La capacidad debe ser mayor a cero"))]
    pub capacity: i16,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CourseUpdateDto {
    #[validate(length(
        min = 1,
        max = 150,
        message = "El nombre debe tener entre 1 y 150 caracteres"
    ))]
    pub name: Option<String>,

    #[validate(length(
        min = 1,
        max = 6,
        message = "El código debe tener entre 1 y 6 caracteres"
    ))]
    pub code: Option<String>,

    pub teacher_id: Option<Uuid>,
    pub status: Option<CourseStatus>,

    #[validate(range(min = 1, message = "La capacidad debe ser mayor a cero"))]
    pub capacity: Option<i16>,
}

#[derive(Debug, Deserialize)]
pub struct EnrollStudentDto {
    pub student_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct CourseResponseDto {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub teacher: TeacherResponseDto,
    pub status: CourseStatus,
    pub capacity: i16,
    pub enrolled_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TeacherResponseDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct StudentResponseDto {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

impl From<Course> for CourseResponseDto {
    fn from(value: Course) -> Self {
        Self {
            id: value.id,
            name: value.name,
            code: value.code,
            teacher: TeacherResponseDto {
                id: value.teacher_id,
                name: value.teacher_name,
                email: value.teacher_email,
            },
            status: value.status,
            capacity: value.capacity,
            enrolled_count: value.enrolled_count,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<Enrollment> for StudentResponseDto {
    fn from(value: Enrollment) -> Self {
        Self {
            id: value.student_id,
            name: value.student_name,
            email: value.student_email,
        }
    }
}
