use crate::{
    api::yandex::{
        cloud::ai::foundation_models::v1::{
            message, text_generation_async_service_client::TextGenerationAsyncServiceClient,
            CompletionOptions, CompletionRequest, CompletionResponse, Message,
        },
    },
    cloud_operation::CloudOperation,
    config::Config,
    iam_generator::IAMToken,
    iam_interceptor,
};
use std::error::Error;

const LLM_ENDPOINT_URL: &str = "https://llm.api.cloud.yandex.net";

const SYSTEM_INSTRUCTION: &str = "Это траскрибирование лекции в университете, сделай текст читаемым, исправь знаки, повторы и ошибки и сформируй абзацы.";
const ASSISTANT_INSTRUCTION: &str = "Переформулируй, чтобы это были не обрывистые высказывания преподавателя, а текст, который удобно читать, а большие связанные абзацы, без обращений к аудитории только суть лекции. Не сокращай, исходный и конечный объём должен быть примерно одинаковый. Если можешь, то разбей на логические блоки. Так же можешь добавить Markdown форматирование";

pub async fn proceed(
    iam: &IAMToken,
    config: &Config,
    input: String,
) -> Result<String, Box<dyn Error>> {
    let mut client = iam_interceptor!(TextGenerationAsyncServiceClient<_>, iam, LLM_ENDPOINT_URL);

    let req = CompletionRequest {
        model_uri: config.model_uri.clone(),
        completion_options: Some(CompletionOptions {
            temperature: Some(0.6),
            max_tokens: Some(2000),
            stream: false,
        }),
        messages: vec![
            Message {
                role: String::from("system"),
                content: Some(message::Content::Text(String::from(SYSTEM_INSTRUCTION))),
            },
            Message {
                role: String::from("assistant"),
                content: Some(message::Content::Text(String::from(ASSISTANT_INSTRUCTION))),
            },
            Message {
                role: String::from("system"),
                content: Some(message::Content::Text(input)),
            },
        ],
    };

    let resp = client.completion(req).await?;
    let mut op = CloudOperation::new(iam, resp.into_inner().id).await?;
    let resp: CompletionResponse = op.wait_done().await?;
    println!("tokens: {:?}", resp.usage);
    let resp = &resp
        .alternatives
        .first()
        .ok_or("Gpt processor: response None")?
        .message;
    let msg = resp
        .clone()
        .ok_or("Gpt processor: response message None")?
        .content
        .ok_or("Gpt processor: response message content None")?;

    Ok(match msg {
        message::Content::Text(text) => text,
    })
}
