use std::sync::Arc;

use crate::{
    subscriber::{FormData, Subscriber},
    Result, Storage,
};
use axum::{Extension, Form, Json};
#[tracing::instrument(
    name = "Adding new subscriber",
    skip(storage, input),
    fields(
        subscriber_email = %input.email,
        subscriber_name = &input.name
    )
)]
pub async fn subscribe(
    Extension(storage): Extension<Arc<Storage>>,
    Form(input): Form<FormData>,
) -> Result<Json<Subscriber>> {
    let subscriber = storage.add_subscriber(input).await?;
    Ok(Json(subscriber))
}
