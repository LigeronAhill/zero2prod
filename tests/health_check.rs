use once_cell::sync::Lazy;
use tokio::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    startup::app,
    telemetry::{get_subscriber, init_subscriber},
    Storage,
};
// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test", "info", std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber("test", "info", std::io::sink);
        init_subscriber(subscriber);
    };
});

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
#[tokio::test]
async fn health_check_works() {
    let address = spawn_app().await.address;
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();
    let response = client
        .get(&format!("{address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");
    let saved = test_app
        .db
        .get_subscriber("ursula_le_guin@gmail.com")
        .await
        .unwrap();
    test_app.db.delete_subscriber(saved.clone()).await.unwrap();
    assert_eq!(200, response.status().as_u16());
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.username, "le guin");
}
#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()
        .unwrap();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");
        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let configuration = get_configuration().unwrap();
    let db = Storage::init(configuration).await.unwrap();
    let app = app(db.clone());
    tokio::spawn(zero2prod::startup::run(listener, app));
    let address = format!("http://127.0.0.1:{}", port);
    TestApp { address, db }
}
pub struct TestApp {
    pub address: String,
    pub db: Storage,
}
