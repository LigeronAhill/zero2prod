use config::Config;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use surrealdb::opt::auth::Root;

use crate::AppError;

/// Represents the application settings.
#[derive(Deserialize, Debug)]
pub struct Settings {
    /// The database settings.
    pub database: DatabaseSettings,
    /// The application settings.
    pub application: ApplicationSettings,
    /// The email client settings.
    pub email_client: EmailClientSettings,
}
/// Represents the application settings.
#[derive(Deserialize, Debug)]
pub struct ApplicationSettings {
    /// The port number for the application.
    pub port: u16,
    /// The host name for the application.
    pub host: String,
}
/// Represents the database settings.
#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    /// The username for the database.
    pub username: String,
    /// The password for the database.
    pub password: Secret<String>,
    /// The port number for the database.
    pub port: u16,
    /// The host name for the database.
    pub host: String,
    /// The name of the database.
    pub database_name: String,
    /// The namespace for the database.
    pub namespace: String,
}
/// Represents the email client settings.
#[derive(Deserialize, Debug)]
pub struct EmailClientSettings {
    /// The token for the email client.
    pub token: Secret<String>,
    /// The base URL for the email client.
    pub base_url: String,
    /// The sender email address for the email client.
    pub sender: String,
}
pub enum Environment {
    Local,
    Development,
    Production,
}
impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Development => "development",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = AppError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            "development" => Ok(Self::Development),
            _ => Err(AppError::EnvError),
        }
    }
}
/// Retrieves the application configuration settings.
///
/// This function reads the configuration settings from a TOML file based on the current environment
/// and returns a `Result` containing the `Settings` struct if successful.
pub fn get_configuration() -> crate::Result<Settings> {
    let path = std::env::current_dir()?.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "production".into())
        .try_into()?;
    let environment_filename = format!("{}.toml", environment.as_str());
    let env = path.join(environment_filename);
    let settings = Config::builder()
        .add_source(config::File::from(env))
        .build()?;
    let config = settings.try_deserialize()?;
    Ok(config)
}
impl Settings {
    pub fn connection_string(&self) -> Secret<String> {
        Secret::new(format!("{}:{}", self.database.host, self.database.port))
    }
    pub fn root(&self) -> Root {
        Root {
            username: &self.database.username,
            password: self.database.password.expose_secret(),
        }
    }
    pub fn app_addr(&self) -> String {
        format!("{}:{}", self.application.host, self.application.port)
    }
}
