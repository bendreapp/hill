use actix_web::{HttpResponse, ResponseError};


/// Top-level application error.
/// Every feature defines its own domain errors via `thiserror`.
/// This enum wraps them into HTTP responses at the presentation boundary.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{message}")]
    NotFound { message: String },

    #[error("{message}")]
    Unauthorized { message: String },

    #[error("{message}")]
    Forbidden { message: String },

    #[error("{message}")]
    BadRequest { message: String },

    #[error("{message}")]
    Conflict { message: String },

    #[error("{message}")]
    PlanLimitExceeded { message: String },

    #[error("{message}")]
    Encryption { message: String },

    #[error("{message}")]
    Integration { message: String },

    #[error("{message}")]
    Database { message: String },

    #[error("{message}")]
    Internal { message: String },
}

impl AppError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound { message: msg.into() }
    }

    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::Unauthorized { message: msg.into() }
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden { message: msg.into() }
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest { message: msg.into() }
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict { message: msg.into() }
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal { message: msg.into() }
    }
}

#[derive(serde::Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}

#[derive(serde::Serialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, code) = match self {
            Self::NotFound { .. } => (actix_web::http::StatusCode::NOT_FOUND, "NOT_FOUND"),
            Self::Unauthorized { .. } => (actix_web::http::StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            Self::Forbidden { .. } => (actix_web::http::StatusCode::FORBIDDEN, "FORBIDDEN"),
            Self::BadRequest { .. } => (actix_web::http::StatusCode::BAD_REQUEST, "BAD_REQUEST"),
            Self::Conflict { .. } => (actix_web::http::StatusCode::CONFLICT, "CONFLICT"),
            Self::PlanLimitExceeded { .. } => (actix_web::http::StatusCode::FORBIDDEN, "PLAN_LIMIT_EXCEEDED"),
            Self::Encryption { .. } => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "ENCRYPTION_ERROR"),
            Self::Integration { .. } => (actix_web::http::StatusCode::BAD_GATEWAY, "INTEGRATION_ERROR"),
            Self::Database { .. } => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR"),
            Self::Internal { .. } => (actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        HttpResponse::build(status).json(ErrorBody {
            error: ErrorDetail {
                code: code.to_string(),
                message: self.to_string(),
            },
        })
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        Self::Database {
            message: "A database error occurred".to_string(),
        }
    }
}
