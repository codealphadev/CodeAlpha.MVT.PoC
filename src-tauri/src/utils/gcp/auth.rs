use std::{env, path::PathBuf};

use gcp_auth::{AuthenticationManager, CustomServiceAccount, Token};
use tracing::{warn};

pub struct GcpAuth {
    authentication_manager: AuthenticationManager,
    token: Option<Token>,
}

impl GcpAuth {
    pub fn new() -> Self {
        let credentials_path = PathBuf::from("src/utils/gcp/logging_service_account.json");
        let mut absolute_path = env::current_dir().unwrap();
        absolute_path.push(credentials_path);

        let service_account = CustomServiceAccount::from_file(absolute_path)
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
