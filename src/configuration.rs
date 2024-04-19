use config::Config;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use surrealdb::opt::auth::Root;

use crate::AppError;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
}
#[derive(Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
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
