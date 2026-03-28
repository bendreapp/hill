use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum SchedulingError {
    #[error("Session not found")]
    SessionNotFound,

    #[error("Blocked slot not found")]
    BlockedSlotNotFound,

    #[error("Recurring reservation not found")]
    RecurringReservationNotFound,

    #[error("Session type not found")]
    SessionTypeNotFound,

    #[error("Time conflict: the requested slot overlaps with an existing booking")]
    TimeConflict,

    #[error("Invalid time range: start must be before end")]
    InvalidTimeRange,

    #[error("Invalid status: {0}")]
    InvalidStatus(String),

    #[error("Booking window violation: session is outside the allowed booking window")]
    BookingWindowViolation,

    #[error("Database error: {0}")]
    Database(String),
}

impl From<SchedulingError> for AppError {
    fn from(err: SchedulingError) -> Self {
        match err {
            SchedulingError::SessionNotFound => AppError::not_found("Session not found"),
            SchedulingError::BlockedSlotNotFound => AppError::not_found("Blocked slot not found"),
            SchedulingError::RecurringReservationNotFound => {
                AppError::not_found("Recurring reservation not found")
            }
            SchedulingError::SessionTypeNotFound => AppError::not_found("Session type not found"),
            SchedulingError::TimeConflict => {
                AppError::conflict("The requested slot overlaps with an existing booking")
            }
            SchedulingError::InvalidTimeRange => {
                AppError::bad_request("Invalid time range: start must be before end")
            }
            SchedulingError::InvalidStatus(msg) => {
                AppError::bad_request(format!("Invalid status: {msg}"))
            }
            SchedulingError::BookingWindowViolation => {
                AppError::bad_request("Session is outside the allowed booking window")
            }
            SchedulingError::Database(msg) => AppError::Database { message: msg },
        }
    }
}
