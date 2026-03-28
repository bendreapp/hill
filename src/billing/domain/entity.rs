use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, utoipa::ToSchema)]
pub struct Invoice {
    pub id: Uuid,
    pub therapist_id: Uuid,
    pub client_id: Uuid,
    pub session_id: Option<Uuid>,
    pub invoice_number: String,
    pub amount_inr: i32,
    pub gst_amount_inr: i32,
    pub total_inr: i32,
    pub status: String,
    pub razorpay_payment_id: Option<String>,
    pub razorpay_order_id: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateInvoiceInput {
    pub client_id: Uuid,
    pub session_id: Option<Uuid>,
    pub amount_inr: i32,
    pub gst_percent: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, utoipa::ToSchema)]
pub struct PaymentOrder {
    pub razorpay_order_id: String,
    pub amount_paise: i64,
    pub currency: String,
    pub invoice_id: Uuid,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct MarkPaidInput {
    pub razorpay_payment_id: String,
    pub razorpay_order_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, utoipa::ToSchema)]
pub struct CreateOrderInput {
    pub invoice_id: Uuid,
}
