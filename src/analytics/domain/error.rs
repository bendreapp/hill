use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum AnalyticsError {
    #[error("Database error: {0}")]
    Database(String),
}

impl From<AnalyticsError> for AppError {
    fn from(err: AnalyticsError) -> Self {
        match err {
            AnalyticsError::Database(msg) => AppError::Database { message: msg },
        }
    }
}
