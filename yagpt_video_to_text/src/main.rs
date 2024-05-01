use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};

pub mod api {
    tonic::include_proto!("yandex.cloud.iam.v1");
}

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

/// A client capable of generating IAM tokens.
struct IAMGenerator {
    auth_info: AuthInfo,
}

impl IAMGenerator {
    /// Constructs a new `TokenClient` given a path to the authentication information file.
    pub fn new(file_path: String) -> io::Result<Self> {
        let data = fs::read_to_string(file_path)?;
        let auth_info = serde_json::from_str(&data).map_err(io::Error::from)?;
        Ok(Self { auth_info })
    }

    pub async fn generate_iam_token(&self) -> Result<String, Box<dyn std::error::Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let claims = Claims {
            aud: IAM_URL.to_string(),
            iss: self.auth_info.service_account_id.clone(),
            iat: now,
            exp: now + 360,
        };

        let header = Header {
            alg: Algorithm::PS256,
            kid: Some(self.auth_info.id.clone()),
            ..Default::default()
        };

        let jwt = encode(
            &header,
            &claims,
            &EncodingKey::from_rsa_pem(self.auth_info.private_key.as_bytes())?,
        )?;

        let mut a = api::iam_token_service_client::IamTokenServiceClient::connect(
            "https://iam.api.cloud.yandex.net",
        )
        .await?;

        let res = a
            .create(api::CreateIamTokenRequest {
                identity: Some(api::create_iam_token_request::Identity::Jwt(jwt.clone())),
            })
            .await?;

        Ok(res.get_ref().iam_token.clone())
    }
}

#[tokio::main]
async fn main() {
    let file_path = String::from("/home/lyra/tmp/authorized_key.json");
    let client = IAMGenerator::new(file_path).unwrap();

    match client.generate_iam_token().await {
        Ok(token) => println!("IAM Token: {}", token),
        Err(e) => eprintln!("Error: {}", e),
    }
}
