use crate::shared::error::AppError;

#[derive(Debug, thiserror::Error)]
pub enum BillingError {
    #[error("Invoice not found")]
    InvoiceNotFound,

    #[error("Payment failed: {0}")]
    PaymentFailed(String),

    #[error("Database error: {0}")]
    Database(String),
}

impl From<BillingError> for AppError {
    fn from(err: BillingError) -> Self {
        match err {
            BillingError::InvoiceNotFound => AppError::not_found("Invoice not found"),
            BillingError::PaymentFailed(msg) => AppError::Integration { message: msg },
            BillingError::Database(msg) => AppError::Database { message: msg },
        }
    }
}
