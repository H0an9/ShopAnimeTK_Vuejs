use crate::{models::ErrorBody, AppState};
use axum::{
    http::StatusCode,
    middleware,
    response::{IntoResponse, Response},
    Json, Router,
};

pub mod animes;
pub mod auth_db;
pub mod categories;
pub mod dashboard;
pub mod orders;
pub mod products;
pub mod promotions;
pub mod statuses;
pub mod tags;
pub mod users;

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth_db::routes())
        .nest("/dashboard", dashboard::routes())
        .nest("/products", products::routes())
        .nest("/categories", categories::routes())
        .nest("/animes", animes::routes())
        .nest("/orders", orders::routes())
        .nest("/promotions", promotions::routes())
        .nest("/tags", tags::routes())
        .nest("/users", users::routes())
        .nest("/statuses", statuses::routes())
        .route_layer(middleware::from_fn_with_state(
            state,
            auth_db::require_admin,
        ))
}

pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    pub fn not_found(message: &str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => ApiError::not_found("Không tìm thấy dữ liệu"),
            other => Self {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: other.to_string(),
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                message: self.message,
            }),
        )
            .into_response()
    }
}

pub type ApiResult<T> = Result<T, ApiError>;
