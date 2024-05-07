use crate::{
    api::yandex::cloud::ai::stt::v2::{
        recognition_audio::AudioSource, recognition_spec::AudioEncoding,
        stt_service_client::SttServiceClient, LongRunningRecognitionRequest,
        LongRunningRecognitionResponse, RecognitionAudio, RecognitionConfig, RecognitionSpec,
    },
    cloud_operation::CloudOperation,
    iam_generator::IAMToken,
    iam_interceptor,
};
use std::{error::Error, time::Duration};

const STT_GRPC_URL: &str = "https://transcribe.api.cloud.yandex.net";

pub async fn autio_to_text(iam: &IAMToken, uri: String) -> Result<Vec<String>, Box<dyn Error>> {
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
        audio_source: Some(AudioSource::Uri(uri)),
    };

    let request = LongRunningRecognitionRequest {
        config: Some(config),
        audio: Some(audio),
    };

    let mut client = iam_interceptor!(SttServiceClient<_>, iam, STT_GRPC_URL);
    let res = client.long_running_recognize(request).await?;

    let mut op = CloudOperation::new(iam)
        .await?
        .set_timeout(Duration::from_secs((4 * 60) * 10 * 2));
    let resp: LongRunningRecognitionResponse = op.wait_done(res.get_ref().id.clone()).await?;

    let mut out: Vec<String> = Vec::new();
    for chunk in resp.chunks {
        out.push(
            chunk
                .alternatives
                .first()
                .and_then(|c| Some(c.text.to_string()))
                .get_or_insert("".to_string())
                .to_string(),
        );
    }

    Ok(out)
}
