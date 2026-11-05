//! One error type for every handler. Renders as JSON `{ "error": "..." }`
//! with a matching HTTP status. Messages are user-facing (Vietnamese).

#[cfg(feature = "actix")]
use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    Unauthorized,
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    PaymentRequired(String),
    TooMany(String),
    Internal(String),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn bad(msg: impl Into<String>) -> Self {
        AppError::BadRequest(msg.into())
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        AppError::Forbidden(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        AppError::NotFound(msg.into())
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        AppError::Internal(msg.into())
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::BadRequest(m) => write!(f, "{m}"),
            AppError::Unauthorized => write!(f, "Bạn cần đăng nhập."),
            AppError::Forbidden(m) => write!(f, "{m}"),
            AppError::NotFound(m) => write!(f, "{m}"),
            AppError::Conflict(m) => write!(f, "{m}"),
            AppError::PaymentRequired(m) => write!(f, "{m}"),
            AppError::TooMany(m) => write!(f, "{m}"),
            AppError::Internal(_) => write!(f, "Lỗi máy chủ, vui lòng thử lại."),
        }
    }
}

#[cfg(feature = "actix")]
impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::PaymentRequired(_) => StatusCode::PAYMENT_REQUIRED,
            AppError::TooMany(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        if let AppError::Internal(detail) = self {
            tracing::error!("internal error: {detail}");
        }
        HttpResponse::build(self.status_code()).json(serde_json::json!({ "error": self.to_string() }))
    }
}

#[cfg(feature = "mongodb")]
impl From<mongodb::error::Error> for AppError {
    fn from(e: mongodb::error::Error) -> Self {
        AppError::Internal(format!("mongo: {e}"))
    }
}

impl From<bson::ser::Error> for AppError {
    fn from(e: bson::ser::Error) -> Self {
        AppError::Internal(format!("bson ser: {e}"))
    }
}

impl From<bson::de::Error> for AppError {
    fn from(e: bson::de::Error) -> Self {
        AppError::Internal(format!("bson de: {e}"))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::BadRequest(format!("JSON không hợp lệ: {e}"))
    }
}

#[cfg(feature = "reqwest")]
impl From<reqwest::Error> for AppError {
    fn from(e: reqwest::Error) -> Self {
        AppError::Internal(format!("http: {e}"))
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e.to_string())
    }
}
