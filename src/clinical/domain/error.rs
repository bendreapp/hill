use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum ClinicalError {
    #[error("Session note not found")]
    NoteNotFound,

    #[error("Treatment plan not found")]
    PlanNotFound,

    #[error("Message not found")]
    MessageNotFound,

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<ClinicalError> for AppError {
    fn from(err: ClinicalError) -> Self {
        match err {
            ClinicalError::NoteNotFound => AppError::not_found("Session note not found"),
            ClinicalError::PlanNotFound => AppError::not_found("Treatment plan not found"),
            ClinicalError::MessageNotFound => AppError::not_found("Message not found"),
            ClinicalError::EncryptionFailed(msg) => AppError::Encryption { message: msg },
            ClinicalError::Database(msg) => AppError::Database { message: msg },
        }
    }
}
