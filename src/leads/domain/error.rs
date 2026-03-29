use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum LeadsError {
    #[error("Lead not found")]
    LeadNotFound,
    #[error("Invitation not found")]
    InvitationNotFound,
    #[error("Invitation expired")]
    InvitationExpired,
    #[error("Invitation already claimed")]
    InvitationAlreadyClaimed,
    #[error("Client already has portal access")]
    ClientAlreadyHasPortal,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl From<LeadsError> for AppError {
    fn from(e: LeadsError) -> Self {
        match e {
            LeadsError::LeadNotFound => AppError::not_found("Lead"),
            LeadsError::InvitationNotFound => AppError::not_found("Invitation"),
            LeadsError::InvitationExpired => AppError::bad_request("Invitation has expired"),
            LeadsError::InvitationAlreadyClaimed => AppError::bad_request("Invitation already used"),
            LeadsError::ClientAlreadyHasPortal => AppError::bad_request("Client already has portal access"),
            LeadsError::Database(e) => AppError::from(e),
        }
    }
}
