use config::Config;
use serde::Deserialize;
use surrealdb::opt::auth::Root;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}
#[derive(Deserialize, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub namespace: String,
}
pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::with_name("configuration"))
        .build()?;
    settings.try_deserialize()
}
impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
    pub fn root(&self) -> Root {
        Root {
            username: &self.username,
            password: &self.password,
        }
    }
}
