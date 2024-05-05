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
    operation_id: String,
    client: OperationServiceClient<InterceptedService<Channel, IAMToken>>,
}

impl CloudOperation {
    pub async fn new(
        iam: &IAMToken,
        operation_id: String,
    ) -> Result<Self, tonic::transport::Error> {
        Ok(Self {
            operation_id,
            client: iam_interceptor!(OperationServiceClient<_>, iam, OPERATION_GRPC_URL),
        })
    }

    pub async fn wait_done<R>(&mut self) -> Result<R, Box<dyn Error>>
    where
        R: Message + Default,
    {
        let started = Instant::now();
        loop {
            let operation_status = self
                .client
                .get(GetOperationRequest {
                    operation_id: self.operation_id.clone(),
                })
                .await?
                .into_inner();

            if let Some(operation::Result::Error(err)) = operation_status.result {
                return Err(format!("Cloud operation: execution error: {:?}", err).into());
            }

            if !operation_status.done {
                if started.elapsed() > Duration::from_secs(300) {
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
