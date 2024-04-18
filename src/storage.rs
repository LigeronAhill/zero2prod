use crate::{
    configuration::Settings,
    subscriber::{FormData, Subscriber},
    AppError, Result,
};
use secrecy::ExposeSecret;
use surrealdb::{
    engine::remote::ws::{Client, Ws},
    Surreal,
};
use tracing::debug;
#[derive(Clone, Debug)]
pub struct Storage {
    db: Surreal<Client>,
}
impl Storage {
    pub async fn init(config: Settings) -> Result<Storage> {
        let conn = config.database.connection_string();
        let root = config.database.root();
        let db = Surreal::new::<Ws>(conn.expose_secret()).await?;
        db.signin(root).await?;
        db.use_ns("zero2prod").use_db("newsletter").await?;
        let sql = [
            "DEFINE TABLE subscriber SCHEMAFULL;",
            "DEFINE FIELD username ON TABLE subscriber TYPE string;",
            "DEFINE FIELD email ON TABLE subscriber TYPE string ASSERT string::is::email($value);",
            "DEFINE FIELD subscribed_at ON TABLE subscriber TYPE datetime;",
            "DEFINE INDEX userEmailIndex ON TABLE subscriber COLUMNS email UNIQUE;",
        ];
        for q in sql {
            let qr = db.query(q).await?;
            debug!("Query result: {qr:?}");
        }
        Ok(Storage { db })
    }
    #[tracing::instrument(name = "Saving new subscriber details in the database", skip(input))]
    pub async fn add_subscriber(&self, input: FormData) -> Result<Subscriber> {
        let s = Subscriber::from(input);
        let subscriber: Option<Subscriber> =
            self.db.create(s.id.clone()).content(s).await.map_err(|e| {
                if e.to_string().contains("userEmailIndex") {
                    AppError::EmailAlreadyExists
                } else {
                    AppError::DatabaseError
                }
            })?;
        match subscriber {
            Some(s) => {
                tracing::debug!("New subscriber details have been saved");
                Ok(s)
            }
            None => {
                tracing::error!("Failed to save subscriber to db!");
                Err(AppError::DatabaseError)
            }
        }
    }
    #[tracing::instrument(name = "Retrieveing subscriber by email")]
    pub async fn get_subscriber_by_email<T>(&self, email: T) -> Result<Subscriber>
    where
        T: ToString + std::fmt::Debug,
    {
        tracing::debug!("Searching for '{}' in database.", email.to_string());
        let sql = format!(
            "SELECT * FROM subscriber WHERE email == '{}';",
            email.to_string()
        );
        let mut res = self
            .db
            .query(sql)
            .await
            .map_err(|_| AppError::DatabaseError)?;
        let s: Vec<Subscriber> = res.take(0).map_err(|_| AppError::DatabaseError)?;
        match s.first() {
            Some(s) => Ok(s.clone()),
            None => Err(AppError::UserNotFound),
        }
    }
    #[tracing::instrument]
    pub async fn delete_subscriber(&self, subscriber: Subscriber) -> Result<()> {
        tracing::debug!(
            "Deleting subscriber with email: '{}' from database.",
            subscriber.email
        );
        let _deleted: Option<Subscriber> = self
            .db
            .delete(subscriber.id)
            .await
            .map_err(|_| AppError::DatabaseError)?;
        Ok(())
    }
}
