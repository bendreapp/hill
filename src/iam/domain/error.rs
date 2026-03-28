use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum IamError {
    #[error("Therapist not found")]
    TherapistNotFound,

    #[error("Slug already taken")]
    SlugTaken,

    #[error("Practice not found")]
    PracticeNotFound,

    #[error("User already has a practice")]
    AlreadyHasPractice,

    #[error("User is not the practice owner")]
    NotPracticeOwner,

    #[error("Member not found")]
    MemberNotFound,

    #[error("Cannot remove the practice owner")]
    CannotRemoveOwner,

    #[error("Invitation not found")]
    InvitationNotFound,

    #[error("Invitation expired")]
    InvitationExpired,

    #[error("Invitation already accepted")]
    InvitationAlreadyAccepted,

    #[error("Onboarding token not found")]
    OnboardingTokenNotFound,

    #[error("Onboarding token expired or exhausted")]
    OnboardingTokenInvalid,

    #[error("Database error: {0}")]
    Database(String),
}

impl From<IamError> for AppError {
    fn from(err: IamError) -> Self {
        match err {
            IamError::TherapistNotFound => AppError::not_found("Therapist not found"),
            IamError::SlugTaken => AppError::conflict("Slug already taken"),
            IamError::PracticeNotFound => AppError::not_found("Practice not found"),
            IamError::AlreadyHasPractice => AppError::conflict("User already has a practice"),
            IamError::NotPracticeOwner => AppError::forbidden("Only the practice owner can do this"),
            IamError::MemberNotFound => AppError::not_found("Practice member not found"),
            IamError::CannotRemoveOwner => AppError::bad_request("Cannot remove the practice owner"),
            IamError::InvitationNotFound => AppError::not_found("Invitation not found"),
            IamError::InvitationExpired => AppError::bad_request("Invitation has expired"),
            IamError::InvitationAlreadyAccepted => AppError::conflict("Invitation already accepted"),
            IamError::OnboardingTokenNotFound => AppError::not_found("Onboarding token not found"),
            IamError::OnboardingTokenInvalid => AppError::bad_request("Token is expired or has reached its usage limit"),
            IamError::Database(msg) => AppError::Database { message: msg },
        }
    }
}
