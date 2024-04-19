use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};

use crate::{validate_name, AppError};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FormData {
    pub email: String,
    pub name: String,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscriber {
    pub id: Thing,
    pub email: String,
    pub username: String,
    pub subscribed_at: surrealdb::sql::Datetime,
}
impl TryFrom<FormData> for Subscriber {
    type Error = AppError;
    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        if !validate_name(&value.name) {
            Err(AppError::InvalidName)
        } else {
            Ok(Self {
                id: Thing {
                    tb: String::from("subscriber"),
                    id: Id::uuid(),
                },
                email: value.email,
                username: value.name,
                subscribed_at: surrealdb::sql::Datetime(chrono::Utc::now()),
            })
        }
    }
}
