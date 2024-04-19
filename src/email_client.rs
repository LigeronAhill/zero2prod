use fake::{Fake, Faker};
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

use crate::{configuration::Settings, Result};
const HEADER: &str = "X-Postmark-Server-Token";
#[derive(Clone)]
pub struct EmailClient {
    http_client: reqwest::Client,
    base_url: reqwest::Url,
    sender: String,
    token: Secret<String>,
}
impl EmailClient {
    pub fn new(config: &Settings) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(Self {
            http_client,
            base_url: reqwest::Url::parse(&config.email_client.base_url)?,
            sender: config.email_client.sender.clone(),
            token: config.email_client.token.clone(),
        })
    }
    pub fn for_tests(server: String, sender: String) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(200))
            .build()?;
        Ok(Self {
            http_client,
            base_url: reqwest::Url::parse(&server)?,
            sender,
            token: Secret::new(Faker.fake()),
        })
    }
    pub async fn send_email(
        &self,
        recipient: String,
        subject: &str,
        html_body: &str,
        text_body: &str,
    ) -> Result<()> {
        let url = self.base_url.join("email")?;
        let request_body = SendEmailRequest {
            from: &self.sender,
            to: &recipient,
            subject,
            html_body,
            text_body,
        };
        self.http_client
            .post(url)
            .header(HEADER, self.token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}
#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
        Fake,
    };
    use wiremock::{
        matchers::{any, header, header_exists, method, path},
        Mock, MockServer, Request, ResponseTemplate,
    };

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            // Try to parse the body as a JSON value
            serde_json::from_slice::<serde_json::Value>(&request.body).is_ok_and(|body| {
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            })
        }
    }

    use crate::{email_client::HEADER, EmailClient, Result};
    struct MockData {
        mock_server: MockServer,
        email_client: EmailClient,
        subscriber_email: String,
        subject: String,
        content: String,
    }
    async fn generate_test_data() -> Result<MockData> {
        let mock_server = MockServer::start().await;
        let sender: String = SafeEmail().fake();
        let email_client = EmailClient::for_tests(mock_server.uri(), sender.clone())?;
        let subscriber_email: String = SafeEmail().fake();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();
        Ok(MockData {
            mock_server,
            email_client,
            subscriber_email,
            subject,
            content,
        })
    }
    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() -> Result<()> {
        let mock_data = generate_test_data().await?;
        Mock::given(header_exists(HEADER))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_data.mock_server)
            .await;
        mock_data
            .email_client
            .send_email(
                mock_data.subscriber_email,
                &mock_data.subject,
                &mock_data.content,
                &mock_data.content,
            )
            .await?;
        Ok(())
    }
    #[tokio::test]
    async fn send_email_succeeds_if_the_server_returns_200() -> Result<()> {
        let mock_data = generate_test_data().await?;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_data.mock_server)
            .await;
        let outcome = mock_data
            .email_client
            .send_email(
                mock_data.subscriber_email,
                &mock_data.subject,
                &mock_data.content,
                &mock_data.content,
            )
            .await;
        assert!(outcome.is_ok());
        Ok(())
    }
    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() -> Result<()> {
        let mock_data = generate_test_data().await?;
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_data.mock_server)
            .await;
        let outcome = mock_data
            .email_client
            .send_email(
                mock_data.subscriber_email,
                &mock_data.subject,
                &mock_data.content,
                &mock_data.content,
            )
            .await;
        assert!(outcome.is_err());
        Ok(())
    }
    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() -> Result<()> {
        let mock_data = generate_test_data().await?;
        let response = ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_data.mock_server)
            .await;
        let outcome = mock_data
            .email_client
            .send_email(
                mock_data.subscriber_email,
                &mock_data.subject,
                &mock_data.content,
                &mock_data.content,
            )
            .await;
        assert!(outcome.is_err());
        Ok(())
    }
    #[tokio::test]
    async fn send_email_sends_the_expected_request() -> Result<()> {
        let mock_data = generate_test_data().await?;
        Mock::given(header_exists(HEADER))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_data.mock_server)
            .await;
        let outcome = mock_data
            .email_client
            .send_email(
                mock_data.subscriber_email,
                &mock_data.subject,
                &mock_data.content,
                &mock_data.content,
            )
            .await;
        assert!(outcome.is_ok());
        Ok(())
    }
}
