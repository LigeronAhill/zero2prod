use std::sync::Arc;

use crate::{
    subscriber::{FormData, Subscriber},
    Result, Storage,
};
use axum::{http::HeaderMap, Extension, Form, Json};
#[tracing::instrument(
    name = "Adding new subscriber",
    skip(storage, input, headers),
    fields(
        request_id = %get_req_id(headers.clone()), 
        subscriber_email = %input.email,
        subscriber_name = %input.name
    )
)]
pub async fn subscribe(
    Extension(storage): Extension<Arc<Storage>>,
    headers: HeaderMap,
    Form(input): Form<FormData>,
) -> Result<Json<Subscriber>> {
    let subscriber = storage.add_subscriber(input).await?;
    Ok(Json(subscriber))
}
fn get_req_id(headers: HeaderMap) -> u64 {
    headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok()).and_then(|s| s.parse::<u64>().ok())
        .unwrap_or_default()
}
