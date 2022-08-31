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
          "private_key_id": "b74be6b39004a5830f55d746060fb22ac32f4a96",
          "private_key": "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCmuBds+WnfSklo\n8C53O5vXaeiSqCvjHlzWNtlE1oHlNOIzWrH6VX/fYjUd0PhynWtkS3LwvP88j/1K\nKKQsBHtRSl5rGuH4u0BO8v+MyfIyFS+QDQSsIjjcP5n1s0EujuTVQ1uv+EIup2ck\nlflzPyfCoriYjDG5BqMdjXr3S32Y5v7hGc7juBoGC5FGp3WFXo1SzblJ1cOcYuRC\n2/l6DTU2Z+A5OLBLUdIFaaiUFrgMruELDEtO86dmebXwEXWvzSNONAXXIXFVHuIW\n3LUrdCiaG5X5x2c7HiSNxMjBPRv+ZyWOKlHqM+6YANWVNDbGi1KQIA88oPY5rfie\n30bUu5fzAgMBAAECggEABnYkEkkoXUsXxUVpx5070QF9zhFOgiLvHi5KBWz649NE\n4RCmzTj7F8FKpbkvp+GKA/0Ym1P34HUEMBebJUmXLfNf7W7BS06Wk3i/hslc5awE\nh+S5RaKd6YtdgDROVmubWtideRh6GCzMxpI5dMoB1H1SAjeMxcsID2EoXuVCHObm\n+v+goq2/CsPLMhv0heryscZy+cuwgW216HTyrtZhFi5uRRHM/2QkQpU02ZLv8dZ0\nJkhv9hPOgqZ+sSSsxkjI3xmKns9qzvelKcs90OpXvYmcw0mJjjICtc+yRa5RwO38\nMByEiyds/aoMw7JCMiHAsR/ktAcC5LYhFEtsjpfngQKBgQDXNs2A3m/ss4wfGAIY\na7QrhMT0sGwsCOvxTSoE0F982pusT7DyYb2/rYptyWSEekJyUR0jVJoz1kv1zGpO\nrBwMy2FEZIgcZ2mCIS5ORwhXlhG5Uw5yOCaGePh5447xE/sjl1a5cSyJ+Ws8jFtE\n3IL1oAOW441EXbYjg3i8zr/YgwKBgQDGUIk3nW60uqC1uDSen/e5nljrRua9XAKB\ndM104tGbra+In78rC7s2WiSg6VtfCLUzys6tkfUCB9AVpLmBBYzjuc/CnySoixe6\n7Syght/bgn28tjanokFNeCIrpzIN9vIk065wmBMR3StTHRp3uHNQ9OPYTjh9ek2F\n2G1OjprH0QKBgAa6huhNjBBJeMsMFByxWVu29fV0gp1J1h6gO/0Uire7mfxkLXMW\njTkt8tMF5eeibutmD9Qn8/5E5/ESaAx3oxTfUvYxcnP447qW0PzPibo7TeiOaTg2\n+zVfGN2uuKxe5I4zUBnSKQNTUf/9n2jiwBFv5EzWZndJusUXejHlAOiDAoGAeALM\nxycyk5RNJSswTL4pv96fmOHzSKyhp43zt8R7bGaTT2681WPoC2BJwkb10HEIlyso\nH8mwJ2Zq+m4RBI9DT5xmqjR8jrJ9zdUxqIN2fPRe+xCZyqBaPHNy2k/37pN+veKM\nMf8Vqz3YthdqqQCqaFeKep/7d5PHzjPHEj8iU8ECgYEAorOiNT2+CDSFoS4gcwvh\nsePUJR3UxDYuDSeSqLOgzjS57Urhztp//S77up74wNGdGo+2Uhgs35NHD8A6d3gd\nZljIUXRbi/1q+NKYqRhgcT7sl7eBvuKTn+GUgIRFbBLI15cjxWgQRRzVHg8VwP3X\nCSL1/zAP91kd0/25ry267Jo=\n-----END PRIVATE KEY-----\n",
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
