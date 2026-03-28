use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Client not found")]
    ClientNotFound,

    #[error("Plan limit exceeded: maximum {0} active clients allowed")]
    PlanLimitExceeded(i64),

    #[error("A client with this email already exists")]
    DuplicateEmail,

    #[error("Database error: {0}")]
    Database(String),
}

impl From<ClientError> for AppError {
    fn from(err: ClientError) -> Self {
        match err {
            ClientError::ClientNotFound => AppError::not_found("Client not found"),
            ClientError::PlanLimitExceeded(max) => AppError::PlanLimitExceeded {
                message: format!("Plan limit exceeded: maximum {} active clients allowed", max),
            },
            ClientError::DuplicateEmail => {
                AppError::conflict("A client with this email already exists")
            }
            ClientError::Database(msg) => AppError::Database { message: msg },
        }
    }
}
