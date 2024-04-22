use once_cell::sync::Lazy;
use tokio::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    startup::app,
    telemetry::{get_subscriber, init_subscriber},
    EmailClient, Storage,
};
pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

// Ensure that the `tracing` stack is only initialised once using `once_cell`
pub static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test", "info", std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test", "info", std::io::sink);
        init_subscriber(subscriber);
    };
});
pub struct TestApp {
    pub address: String,
    pub db: Storage,
}
impl TestApp {
    pub async fn post_subscriptions(&self, body: &'static str) -> reqwest::Response {
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap();
        client
            .post(&format!("{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}
pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("0.0.0.0:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let configuration = get_configuration().unwrap();
    let mail = EmailClient::new(&configuration).unwrap();
    let db = Storage::init(configuration).await.unwrap();
    let app = app(db.clone(), mail);
    tokio::spawn(zero2prod::startup::run(listener, app));
    let address = format!("http://0.0.0.0:{}", port);
    TestApp { address, db }
}
