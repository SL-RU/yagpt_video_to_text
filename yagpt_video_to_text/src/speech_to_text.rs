use prost::Message;

use crate::{
    api::yandex::cloud::{
        ai::stt::v2::{
            self as api, recognition_spec::AudioEncoding, stt_service_client::SttServiceClient,
            LongRunningRecognitionRequest, LongRunningRecognitionResponse, RecognitionAudio,
            RecognitionConfig, RecognitionSpec,
        },
        operation::{operation_service_client::OperationServiceClient, GetOperationRequest},
    },
    iam_generator::IAMToken,
    iam_interceptor,
};
use std::{error::Error, time::Duration};

const STT_GRPC_URL: &str = "https://transcribe.api.cloud.yandex.net";
const OPERATION_GRPC_URL: &str = "https://operation.api.cloud.yandex.net";

pub async fn autio_to_text(iam: &IAMToken, uri: String) -> Result<String, Box<dyn Error>> {
    let config = RecognitionConfig {
        specification: Some(RecognitionSpec {
            audio_encoding: AudioEncoding::Mp3 as i32,
            sample_rate_hertz: 48000,
            language_code: String::from("ru-RU"),
            profanity_filter: false,
            model: String::from("general"),
            partial_results: false,
            single_utterance: false,
            audio_channel_count: 1,
            raw_results: false,
            literature_text: true,
        }),
        folder_id: String::from(""),
    };

    let audio = RecognitionAudio {
        audio_source: Some(api::recognition_audio::AudioSource::Uri(uri)),
    };

    let request = LongRunningRecognitionRequest {
        config: Some(config),
        audio: Some(audio),
    };

    let mut client = iam_interceptor!(SttServiceClient<_>, iam, STT_GRPC_URL);
    let res = client.long_running_recognize(request).await?;

    let mut operation_client = iam_interceptor!(OperationServiceClient<_>, iam, OPERATION_GRPC_URL);

    loop {
        let operation_status = operation_client
            .get(GetOperationRequest {
                operation_id: res.get_ref().id.clone(),
            })
            .await?
            .into_inner();

        if operation_status.done {
            if let Some(op) = operation_status.result {
                match op {
                    crate::api::yandex::cloud::operation::operation::Result::Error(_) => {
                        return Ok(String::new())
                    }
                    crate::api::yandex::cloud::operation::operation::Result::Response(resp) => {
                        let resp = LongRunningRecognitionResponse::decode(resp.value.as_slice())?;
                        let mut out = String::new();
                        for chunk in resp.chunks {
                            out += &chunk.alternatives[0].text;
                            out += "\r\n";
                        }
                        return Ok(out);
                    }
                }
            }

            break;
        }

        tokio::time::sleep(Duration::from_millis(1000)).await;
    }

    Ok("".to_string())
}
