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
          "project_id": "client-backend-x",
          "private_key_id": "",
          "private_key": "",
          "client_email": "client-tracing@client-backend-x.iam.gserviceaccount.com",
          "client_id": "102582281467419878202",
          "auth_uri": "https://accounts.google.com/o/oauth2/auth",
          "token_uri": "https://oauth2.googleapis.com/token",
          "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
          "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/client-tracing%40client-backend-x.iam.gserviceaccount.com"
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
