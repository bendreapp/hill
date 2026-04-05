use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum EngagementError {
    #[error("Resource not found")]
    ResourceNotFound,

    #[error("Intake form not found")]
    IntakeFormNotFound,

    #[error("Intake response not found")]
    IntakeResponseNotFound,

    #[error("Intake form question not found")]
    IntakeQuestionNotFound,

    #[error("Message template not found")]
    MessageTemplateNotFound,

    #[error("Invalid template key: {0}")]
    InvalidTemplateKey(String),

    #[error("Broadcast failed: {0}")]
    BroadcastFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("This intake form has already been submitted")]
    AlreadySubmitted,

    #[error("Database error: {0}")]
    Database(String),
}

impl From<EngagementError> for AppError {
    fn from(err: EngagementError) -> Self {
        match err {
            EngagementError::MessageTemplateNotFound => AppError::not_found("Message template not found"),
            EngagementError::InvalidTemplateKey(msg) => AppError::BadRequest { message: msg },
            EngagementError::ResourceNotFound => AppError::not_found("Resource not found"),
            EngagementError::IntakeFormNotFound => AppError::not_found("Intake form not found"),
            EngagementError::IntakeResponseNotFound => {
                AppError::not_found("Intake response not found")
            }
            EngagementError::IntakeQuestionNotFound => {
                AppError::not_found("Intake form question not found")
            }
            EngagementError::AlreadySubmitted => AppError::Conflict { message: "This intake form has already been submitted".to_string() },
            EngagementError::BroadcastFailed(msg) => AppError::Integration { message: msg },
            EngagementError::EncryptionFailed(msg) => AppError::Encryption { message: msg },
            EngagementError::Database(msg) => AppError::Database { message: msg },
        }
    }
}
