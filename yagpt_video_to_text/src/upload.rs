use crate::config::Config;
use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3 as s3;
use s3::{config::Credentials, presigning::PresigningConfig, primitives::ByteStream};
use std::{error::Error, io, path::PathBuf, time::Duration};

static STORAGE_ENDPOINT_URL: &str = "https://storage.yandexcloud.net";
static STORAGE_REGION: &str = "ru-central1";

pub struct Uploader {
    s3_client: aws_sdk_s3::Client,
    bucket_name: String,
}

impl Uploader {
    pub fn new(config: &Config) -> Self {
        let builder = aws_sdk_s3::Config::builder()
            .behavior_version(BehaviorVersion::latest())
            .credentials_provider(Credentials::new(
                config.aws_access_key_id.clone(),
                config.aws_secret_access_key.clone(),
                None,
                None,
                "",
            ))
            .endpoint_url(STORAGE_ENDPOINT_URL)
            .region(Region::new(STORAGE_REGION))
            .build();

        Self {
            s3_client: aws_sdk_s3::Client::from_conf(builder),
            bucket_name: config.bucket_name.clone(),
        }
    }

    pub async fn upload(&self, local_file_path: PathBuf) -> Result<String, Box<dyn Error>> {
        println!("local {:?}", local_file_path);
        let filename = local_file_path
            .file_name()
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "filename error"))?;
        let body = ByteStream::from_path(local_file_path.clone()).await?;
        let expires_in = Duration::from_secs(3600 * 6);

        if self
            .s3_client
            .delete_object()
            .bucket(self.bucket_name.clone())
            .key(filename.to_string_lossy())
            .send()
            .await
            .is_ok()
        {
            println!("S3: deleted old file with the same name in the bucket")
        }

        self.s3_client
            .put_object()
            .bucket(self.bucket_name.clone())
            .key(filename.to_string_lossy())
            .body(body)
            .send()
            .await?;

        let presigned_request = self
            .s3_client
            .get_object()
            .bucket(self.bucket_name.clone())
            .key(filename.to_string_lossy())
            .presigned(PresigningConfig::expires_in(expires_in)?)
            .await?;

        Ok(String::from(presigned_request.uri()))
    }
}
