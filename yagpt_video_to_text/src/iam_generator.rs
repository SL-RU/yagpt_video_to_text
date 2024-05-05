use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{fs, io};
use tonic::metadata::MetadataValue;
use tonic::transport::{Channel, ClientTlsConfig, Endpoint};

use crate::api::yandex::cloud::iam::v1 as api;

const IAM_GRPC_URL: &str = "https://iam.api.cloud.yandex.net";
const IAM_AUDIENCE_URL: &str = "https://iam.api.cloud.yandex.net/iam/v1/tokens";

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

#[derive(Debug, Clone, Serialize)]
pub struct IAMToken {
    token: String,
}

#[macro_export]
macro_rules! iam_interceptor {
    ($client:ty,$iam:expr,$endpoint:expr) => {{
        <$client>::with_interceptor($iam.connect($endpoint).await?, $iam.clone())
    }};
}

impl IAMToken {
    pub async fn connect(&self, endpoint_url: &str) -> Result<Channel, tonic::transport::Error> {
        Endpoint::new(String::from(endpoint_url))?
            .tls_config(ClientTlsConfig::new())?
            .connect()
            .await
    }
}

impl tonic::service::Interceptor for IAMToken {
    fn call(&mut self, req: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        let mut req = req;
        let token_value: MetadataValue<_> = format!("Bearer {}", self.token).parse().unwrap();
        req.metadata_mut()
            .insert("authorization", token_value.clone());
        Ok(req)
    }
}

/// A client capable of generating IAM tokens.
pub struct IAMGenerator {
    auth_info: AuthInfo,
}

impl IAMGenerator {
    /// Constructs a new `TokenClient` given a path to the authentication information file.
    pub fn new(auth_info_path: String) -> io::Result<Self> {
        let data = fs::read_to_string(auth_info_path)?;
        let auth_info = serde_json::from_str(&data).map_err(io::Error::from)?;
        Ok(Self { auth_info })
    }

    pub async fn generate_iam_token(&self) -> Result<IAMToken, Box<dyn std::error::Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let claims = Claims {
            aud: String::from(IAM_AUDIENCE_URL),
            iss: self.auth_info.service_account_id.clone(),
            iat: now,
            exp: now + 360 * 6,
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

        let mut client =
            api::iam_token_service_client::IamTokenServiceClient::connect(IAM_GRPC_URL).await?;
        let res = client
            .create(api::CreateIamTokenRequest {
                identity: Some(api::create_iam_token_request::Identity::Jwt(jwt.clone())),
            })
            .await?;

        Ok(IAMToken {
            token: res.get_ref().iam_token.clone(),
        })
    }
}
