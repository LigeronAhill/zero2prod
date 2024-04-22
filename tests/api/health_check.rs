use crate::helpers::{spawn_app, APP_USER_AGENT};

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
