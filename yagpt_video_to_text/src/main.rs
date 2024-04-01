use jsonwebtoken::{encode, EncodingKey, Header, Algorithm};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

// Constants for the IAM token API endpoint
const IAM_URL: &str = "https://iam.api.cloud.yandex.net/iam/v1/tokens";

/// Represents authentication information required to generate a JWT.
#[derive(Debug, Serialize, Deserialize)]
struct AuthInfo {
    service_account_id: String,
    id: String,
    private_key: String,
}

/// Claims to be encoded into the JWT.
#[derive(Debug, Serialize)]
struct Claims {
    aud: String,
    iss: String,
    iat: u64,
    exp: u64,
}

/// A trait defining functionality for IAM token generation.
trait IAMTokenGenerator {
    /// Loads authentication information from a specified file.
    fn load_auth_info(&self) -> Result<AuthInfo, Box<dyn std::error::Error>>;

    /// Generates an IAM token using the loaded authentication information.
    async fn generate_iam_token(&self) -> Result<String, Box<dyn std::error::Error>>;
}

/// A client capable of generating IAM tokens.
struct TokenClient {
    file_path: String,
}

impl TokenClient {
    /// Constructs a new `TokenClient` given a path to the authentication information file.
    pub fn new(file_path: String) -> Self {
        TokenClient { file_path }
    }
}

impl IAMTokenGenerator for TokenClient {
    fn load_auth_info(&self) -> Result<AuthInfo, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(&self.file_path)?;
        serde_json::from_str(&data).map_err(Into::into)
    }

    async fn generate_iam_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let auth_info = self.load_auth_info()?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        let claims = Claims {
            aud: IAM_URL.to_string(),
            iss: auth_info.service_account_id,
            iat: now,
            exp: now + 360,
        };

        let header = Header {
            alg: Algorithm::PS256,
            kid: Some(auth_info.id),
            ..Default::default()
        };

        let jwt = encode(&header, &claims, &EncodingKey::from_rsa_pem(auth_info.private_key.as_bytes())?)?;

        let client = Client::new();
        let response = client.post(IAM_URL)
            .json(&serde_json::json!({ "jwt": jwt }))
            .send()
            .await?;

        if response.status().is_success() {
            let body = response.json::<serde_json::Value>().await?;
            Ok(body["iamToken"].as_str().unwrap_or_default().to_string())
        } else {
            Err("Failed to obtain IAM token".into())
        }
    }
}

#[tokio::main]
async fn main() {
    let file_path = String::from("/home/lyra/tmp/authorized_key.json");
    let client = TokenClient::new(file_path);

    match client.generate_iam_token().await {
        Ok(token) => println!("IAM Token: {}", token),
        Err(e) => eprintln!("Error: {}", e),
    }
}
