use std::sync::Arc;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPart, ChatCompletionRequestMessageContentPartImageArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs};
use crate::types::{ImageOutputFormat, RequestBody, RequestParams, ResponseBody};

async fn send_chat_request(client: Arc<Client<OpenAIConfig>>, params: RequestParams, body: RequestBody) -> ResponseBody {
    let system = ChatCompletionRequestSystemMessageArgs::default()
        .content(
            "You are a narration service helping a visually impaired player \
            understand the scene for the game they're playing. \
            Describe the contents of the base64-encoded screenshots you will be given. \
            Your response will be read aloud by a text-to-speech system; \
            limit your response to at most two sentences. \
            Do not use headings or explicit section makers. \
            Do not speculate about the image's contents."
        ) // TODO: Make customizable
        .build()
        .map(ChatCompletionRequestMessage::System)
        .unwrap();

    let message = ChatCompletionRequestMessageContentPartImageArgs::default()
        .image_url(format!("data:image/{:?};base64,{}", body.format.unwrap_or(ImageOutputFormat::Png), body.image))
        .build()
        .map(ChatCompletionRequestMessageContentPart::ImageUrl)
        .unwrap();

    let user = ChatCompletionRequestUserMessageArgs::default()
        .content(vec![message])
        .build()
        .map(ChatCompletionRequestMessage::User)
        .unwrap();

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini") // TODO: Make customizable
        .max_tokens(300u32) // TODO: Make customizable
        .messages(vec![system, user])
        .build()
        .unwrap();

    match client.chat().create(request).await.as_ref() {
        Ok(response) => {
            log::info!(target: "groan", "{:?}", response);
            ResponseBody::text(response.choices[0].message.content.as_ref().unwrap())
        }
        Err(error) => ResponseBody::error(format!("Error: {:?}", error)),
    }
}

pub(crate) async fn query_service(client: Arc<Client<OpenAIConfig>>, params: RequestParams, body: RequestBody) -> ResponseBody {
    match params.output.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice() {
        ["text", ..] => send_chat_request(client, params, body).await,
        ["sound", "wav", ..] => ResponseBody::error("Sound not implemented"),
        ["image", "png", "png-a", ..] => ResponseBody::error("Image not implemented"),
        _ => ResponseBody::error(format!("Unknown output format {:?}", params.output)),
    }
}