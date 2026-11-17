use super::{
    dto::{DeliveryResponseDto, EvaluationInputDto},
    error::DeliveryError,
    storage::{MAX_REQUEST_BYTES, UploadedFile},
};
use crate::{auth::AuthenticatedUser, error::AppError, state::ClassiaState};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, Multipart, Path, State, rejection::JsonRejection},
    http::{StatusCode, header},
    response::IntoResponse,
    routing::{get, put},
};
use serde::Serialize;
use uuid::Uuid;

const DOCX_MIME: &str = "application/vnd.openxmlformats-officedocument.wordprocessingml.document";

#[derive(Debug, Serialize)]
pub(crate) struct MyDeliveryResponseDto {
    delivery: Option<DeliveryResponseDto>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DeliveryListResponseDto {
    items: Vec<DeliveryResponseDto>,
    total: usize,
}

pub fn route() -> Router<ClassiaState> {
    Router::new()
        .route(
            "/{delivery_id}",
            put(replace_delivery).layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES)),
        )
        .route("/{delivery_id}/file", get(download_delivery_file))
        .route("/{delivery_id}/evaluation", put(grade_delivery))
}

/// Multipart body of the upload routes: `file` and, when replacing,
/// `expected_version`.
struct DeliveryForm {
    file: Option<UploadedFile>,
    expected_version: Option<i32>,
}

async fn read_form(mut multipart: Multipart) -> Result<DeliveryForm, DeliveryError> {
    let mut form = DeliveryForm {
        file: None,
        expected_version: None,
    };
    while let Some(field) = multipart.next_field().await.map_err(multipart_error)? {
        match field.name() {
            Some("file") if form.file.is_none() => {
                let file_name = field.file_name().map(str::to_owned);
                let bytes = field.bytes().await.map_err(multipart_error)?.to_vec();
                form.file = Some(UploadedFile { file_name, bytes });
            }
            Some("file") => {
                return Err(DeliveryError::Validation(
                    "Solo se admite un archivo por entrega".into(),
                ));
            }
            Some("expected_version") => {
                let text = field.text().await.map_err(multipart_error)?;
                form.expected_version = text.trim().parse().ok();
            }
            _ => {}
        }
    }
    Ok(form)
}

fn multipart_error(error: axum::extract::multipart::MultipartError) -> DeliveryError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        DeliveryError::FileTooLarge
    } else {
        DeliveryError::Validation("El cuerpo multipart no es válido".into())
    }
}

fn required_file(file: Option<UploadedFile>) -> Result<UploadedFile, DeliveryError> {
    file.ok_or_else(|| DeliveryError::Validation("Debes adjuntar un archivo".into()))
}

pub(crate) async fn submit_delivery(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(activity_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<DeliveryResponseDto>), AppError> {
    let form = read_form(multipart).await?;
    let delivery = state
        .delivery_service
        .submit(user.id, &user.role, activity_id, required_file(form.file)?)
        .await?;
    Ok((StatusCode::CREATED, Json(delivery.into())))
}

async fn replace_delivery(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(delivery_id): Path<Uuid>,
    multipart: Multipart,
) -> Result<Json<DeliveryResponseDto>, AppError> {
    let form = read_form(multipart).await?;
    let delivery = state
        .delivery_service
        .replace(
            user.id,
            &user.role,
            delivery_id,
            form.expected_version,
            required_file(form.file)?,
        )
        .await?;
    Ok(Json(delivery.into()))
}

async fn grade_delivery(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(delivery_id): Path<Uuid>,
    body: Result<Json<EvaluationInputDto>, JsonRejection>,
) -> Result<Json<DeliveryResponseDto>, AppError> {
    let Json(body) = body.map_err(|_| {
        DeliveryError::Validation(
            "La calificación requiere score numérico y expected_version entero".into(),
        )
    })?;
    let delivery = state
        .evaluation_service
        .grade(user.id, &user.role, delivery_id, body)
        .await?;
    Ok(Json(delivery.into()))
}

pub(crate) async fn get_my_delivery(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<MyDeliveryResponseDto>, AppError> {
    let delivery = state
        .delivery_service
        .get_delivery_status(user.id, &user.role, activity_id)
        .await?;

    Ok(Json(MyDeliveryResponseDto {
        delivery: delivery.map(Into::into),
    }))
}

pub(crate) async fn list_deliveries(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(activity_id): Path<Uuid>,
) -> Result<Json<DeliveryListResponseDto>, AppError> {
    let deliveries = state
        .delivery_service
        .list_activity_deliveries(user.id, &user.role, activity_id)
        .await?;

    let items: Vec<DeliveryResponseDto> = deliveries.into_iter().map(Into::into).collect();

    Ok(Json(DeliveryListResponseDto {
        total: items.len(),
        items,
    }))
}

async fn download_delivery_file(
    user: AuthenticatedUser,
    State(state): State<ClassiaState>,
    Path(delivery_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let bytes = state
        .delivery_service
        .download_delivery_file(user.id, &user.role, delivery_id)
        .await?;

    Ok((
        [
            (header::CONTENT_TYPE, DOCX_MIME.to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"entrega-{delivery_id}.docx\""),
            ),
        ],
        bytes,
    ))
}
