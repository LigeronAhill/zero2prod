use serde::{Deserialize, Serialize};
use surrealdb::sql::{Id, Thing};

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
impl From<FormData> for Subscriber {
    fn from(value: FormData) -> Self {
        Self {
            id: Thing {
                tb: String::from("subscriber"),
                id: Id::uuid(),
            },
            email: value.email,
            username: value.name,
            subscribed_at: surrealdb::sql::Datetime(chrono::Utc::now()),
        }
    }
}
