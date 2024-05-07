use prost::Message;
use std::{error::Error, time::Duration};
use tokio::time::Instant;
use tonic::{service::interceptor::InterceptedService, transport::Channel};

use crate::{
    api::yandex::cloud::operation::{
        operation, operation_service_client::OperationServiceClient, GetOperationRequest,
    },
    iam_generator::IAMToken,
    iam_interceptor,
};

const OPERATION_GRPC_URL: &str = "https://operation.api.cloud.yandex.net";

pub struct CloudOperation {
    client: OperationServiceClient<InterceptedService<Channel, IAMToken>>,
    timeout: Duration,
}

impl CloudOperation {
    pub async fn new(iam: &IAMToken) -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            client: iam_interceptor!(OperationServiceClient<_>, iam, OPERATION_GRPC_URL),
            timeout: Duration::from_secs(60 * 2),
        })
    }

    pub fn set_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub async fn wait_done<R>(&mut self, operation_id: String) -> Result<R, Box<dyn Error>>
    where
        R: Message + Default,
    {
        let started = Instant::now();
        loop {
            let operation_status = self
                .client
                .get(GetOperationRequest {
                    operation_id: operation_id.clone(),
                })
                .await?
                .into_inner();

            println!("op {:?}", operation_status);

            if let Some(operation::Result::Error(err)) = operation_status.result {
                return Err(format!("Cloud operation: execution error: {:?}", err).into());
            }

            if !operation_status.done {
                if started.elapsed() > self.timeout {
                    return Err("Cloud operation: execution timeout".into());
                }

                tokio::time::sleep(Duration::from_millis(1000)).await;
                continue;
            }

            let op = operation_status
                .result
                .ok_or("Cloud operation: result None")?;
            return match op {
                operation::Result::Error(err) => {
                    Err(format!("Cloud operation: result error: {:?}", err).into())
                }
                operation::Result::Response(resp) => Ok(R::decode(resp.value.as_slice())?),
            };
        }
    }
}
