use actix_web::{web, HttpResponse};
use serde::Deserialize;
use uuid::Uuid;

use crate::billing::application::service::PaymentService;
use crate::billing::domain::entity::{CreateInvoiceInput, CreateOrderInput, MarkPaidInput};
use crate::shared::error::AppError;
use crate::shared::types::{AuthUser, Paginated};

#[derive(Debug, Deserialize)]
pub struct InvoiceListQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

pub async fn list_invoices(
    user: AuthUser,
    svc: web::Data<PaymentService>,
    query: web::Query<InvoiceListQuery>,
) -> Result<HttpResponse, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).min(100);
    let offset = (page - 1) * per_page;

    let (data, total) = svc
        .list_invoices(&[user.id], query.status.as_deref(), per_page, offset)
        .await?;

    Ok(HttpResponse::Ok().json(Paginated {
        data,
        total,
        page,
        per_page,
    }))
}

pub async fn create_invoice(
    user: AuthUser,
    svc: web::Data<PaymentService>,
    body: web::Json<CreateInvoiceInput>,
) -> Result<HttpResponse, AppError> {
    let invoice = svc.create_invoice(user.id, &body).await?;
    Ok(HttpResponse::Created().json(invoice))
}

pub async fn mark_paid(
    user: AuthUser,
    svc: web::Data<PaymentService>,
    id: web::Path<Uuid>,
    body: web::Json<MarkPaidInput>,
) -> Result<HttpResponse, AppError> {
    let invoice = svc
        .mark_paid(*id, user.id, &body.razorpay_payment_id, body.razorpay_order_id.as_deref())
        .await?;
    Ok(HttpResponse::Ok().json(invoice))
}

pub async fn create_razorpay_order(
    user: AuthUser,
    svc: web::Data<PaymentService>,
    body: web::Json<CreateOrderInput>,
) -> Result<HttpResponse, AppError> {
    let order = svc.create_razorpay_order(body.invoice_id, user.id).await?;
    Ok(HttpResponse::Ok().json(order))
}

// ─── Route Configuration ────────────────────────────────────────────────────

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
            .route("/api/v1/invoices", web::get().to(list_invoices))
            .route("/api/v1/invoices", web::post().to(create_invoice))
            .route("/api/v1/invoices/{id}/paid", web::post().to(mark_paid))
            .route("/api/v1/payments/create-order", web::post().to(create_razorpay_order));
}
