use gcp_auth::{AuthenticationManager, CustomServiceAccount, Token};
use tracing::warn;

pub struct GcpAuth {
    authentication_manager: AuthenticationManager,
    token: Option<Token>,
}

impl GcpAuth {
    pub fn new() -> Self {
        let credentials_json = r#"{
          "type": "service_account",
          "project_id": "client-backend-logs",
          "private_key_id": "6f6d92acab0e104ded58403a95729649f733c6c6",
          "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvAIBADANBgkqhkiG9w0BAQEFAASCBKYwggSiAgEAAoIBAQCHbNbS8iPwhBbb\n5qvhOh6jmWNitKqBf0d1K/0Cy/z9+HLWG7dD3tlF4adR3c/DJrZICU19jtfxQ8k+\n+zvM7Hu4vUMfuXNxam8waw2wmwclM9jskxSjijf0cDBoYX2u+OAO9q1hfJh7csbS\n73wt2lteGiBZHZjJ8Iu67f3zD+O7OaAsYGDQfmFR6INwgdlc90F5hBfu69tWUQul\nEV5sUVxrpA1y8O48FjqQw0zgkokxzJG6fyE0zdVWa/ZXVWusziBfwL2bcWRBtTkX\npIomcRUFtCE9QyQ2aYDpbfTCV/oMhHVdmEva+Ufw1Cn1P9AKwNjo8+E/jtUBlMpA\nHGHXV4vFAgMBAAECggEAEEP+02xY67j5w+TDTTR9XJn+SWs4tYATlZu4cl8M3nY6\nWMQQcXrxLscJITiajS53o1RWtddz0Vsab1Gen6DhxVEaIFPWbU9i1nZtOQKNy6ac\nDeImjvP5f4cAEXFwNpVW2AVB0t0ztVQWVFyGVj2NTtKlgv6ejNay+bs/DrQgXqrC\n+23rskydgHTg2lM6k0PvBEm+rb2ofp2mLvFT1GoQ1nTkiFoBINQj6ySlOoYOe1xD\nwt0LLKO9+AZxqLGgGnOM6fjlBlpQeykWxnqCeUV1eekJRIYoawJO+9+GwobdOwYf\n6u0znVgzkQaE1JOGw6T7UyjfYym+utRVhLc4VStMewKBgQC6hzG7aPz+uubifXLq\nOqjRzf1j+e06ilJzkfUdkgEm4onKSs448LQ63g7ClOHUmrXnFBk7nBsYfx7gsk7e\nW0YnXcvkazy3eUZYjOo6i3L5Ksn+hMKYvbKRca7ZZEYjPohA3TIXEA+eVkj1dyyf\n5lOZR3YfTWQkiNclk0lWQdqlXwKBgQC53SgKO5s24J/r39no230UlXetvuYhj3MM\nB75R3tIxY2KoU3RGZgazT10PJOXlR2aR9hm6asNWOWQ33rYIfpooNpFQT01p4pyC\nFBOq86Jz4Zj4X+W1JMAsCOjtJsLmHCx4X8aA8TroBeW8XjEAAOuqAcibOhXxi+kh\nchTJpf0dWwKBgGSd+N23TIG8ID+cnBhtfBNwoncDokwpwUBGQ4qn76ciHGHa6FMe\nxJncnumnlgoxSl5UIShelN8p82YRySl44ubpWcrlbeMqsB+kI9Vg06xEwKFiy/XI\nnkKGqGLsDEmGuckDiLmYGFS5BrIijxfSDtZoDffkr1hl2GiUgsLc0yzRAoGAWSoP\nm1kdDSaVkcLB90PUrZtG3sNGA7OdrZfJZd1PWQIkwjvG/D7V2A+qxSkeYO/v9PXr\nQdMWArdlrKfbDBgwPDpxW+WvbayZCI45ITngJVeE2yKmQFBxIK7lj1+ZAGtjqhvI\nTvNrHwQ1QjitqSLH67cPVeh3vnkwTis8bcCDvSECgYAUeQpsd6VN72ANfKEx5txq\nRlH+VIjnrMkyrwVIUJRQHNvO/NJmMt6Evf/236F29fvrlezlJUFfLl+EJAzcFZQ9\nlxwKATS0JbeBHGtrpfQYfz9/zx6HWrRRVHX9lCBGSeA5gb50uEHeoVrkXCW9o29D\na06Q9FmDixxldiuol0YuTw==\n-----END PRIVATE KEY-----\n",
          "client_email": "client-tracing@client-backend-logs.iam.gserviceaccount.com",
          "client_id": "110237614836282078916",
          "auth_uri": "https://accounts.google.com/o/oauth2/auth",
          "token_uri": "https://oauth2.googleapis.com/token",
          "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
          "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/client-tracing%40client-backend-logs.iam.gserviceaccount.com"
        }"#;

        let service_account = CustomServiceAccount::from_json(credentials_json)
            .expect("Unable to read credentials file");
        let authentication_manager = AuthenticationManager::from(service_account);
        Self {
            authentication_manager,
            token: None,
        }
    }

    pub async fn token_str(&mut self) -> Option<String> {
        if self.token.is_none()
            || (self.token.is_some() && self.token.as_ref().unwrap().has_expired())
        {
            let token = self.refresh_token().await;
            self.token = token;
        }

        if let Some(token) = self.token.as_ref() {
            Some(token.as_str().to_string())
        } else {
            None
        }
    }

    pub async fn refresh_token(&self) -> Option<Token> {
        let scopes = &["https://www.googleapis.com/auth/logging.write"];
        match self.authentication_manager.get_token(scopes).await {
            Ok(token) => return Some(token),
            Err(e) => {
                warn!(no_remote = true, "failed to refresh_token: {}", e);
                return None;
            }
        }
    }
}
