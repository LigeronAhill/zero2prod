use config::Config;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use surrealdb::opt::auth::Root;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            database: DatabaseSettings::default(),
            application_port: 8000,
        }
    }
}
#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub namespace: String,
}
impl Default for DatabaseSettings {
    fn default() -> Self {
        let host = std::env::var("DB_HOST").unwrap_or(String::from("0.0.0.0"));
        Self {
            username: String::from("root"),
            password: Secret::new(String::from("root")),
            port: 5433,
            host,
            database_name: String::from("newsletter"),
            namespace: String::from("zero2prod"),
        }
    }
}
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()?;
    settings.try_deserialize()
}
impl DatabaseSettings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("{}:{}", self.host, self.port))
    }
    pub fn root(&self) -> Root {
        Root {
            username: &self.username,
            password: self.password.expose_secret(),
        }
    }
}
