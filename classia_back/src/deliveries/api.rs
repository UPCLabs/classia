use super::dto::DeliveryResponseDto;
use crate::{auth::AuthenticatedUser, error::AppError, state::ClassiaState};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::header,
    response::IntoResponse,
    routing::get,
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
    Router::new().route("/{delivery_id}/file", get(download_delivery_file))
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
