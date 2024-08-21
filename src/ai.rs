use std::collections::HashMap;
use std::error::Error;
use crate::types::{
    ImageOutputFormat, InvalidRequestBody, RequestBody, RequestParams, ResponseBody,
};
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPart, ChatCompletionRequestMessageContentPartImageArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, ChatCompletionResponseMessage, CreateChatCompletionRequest, CreateChatCompletionRequestArgs, CreateChatCompletionResponse, CreateSpeechRequest, CreateSpeechRequestArgs, CreateSpeechResponse};
use async_openai::Client;
use bytes::{Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use async_openai::error::OpenAIError;
use async_openai::types::SpeechModel::Tts1;
use async_openai::types::SpeechResponseFormat::Wav;
use async_openai::types::Voice::Fable;
use serde_json::Value;
use tokio::sync::mpsc::{Receiver, Sender};
use warp::Filter;
use warp::hyper::HeaderMap;

pub(crate) type MessageSender = Sender<(u64, ServiceMessage)>;
pub(crate) type MessageReceiver = Receiver<(u64, ServiceMessage)>;

pub(crate) struct AiService {
    client: Arc<Client<OpenAIConfig>>,
    sender: MessageSender,
    next_id: AtomicU64,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ServiceRequest {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) params: String,
    pub(crate) body: Value,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) enum OpenAiMessage {
    CreateChatCompletionRequest(CreateChatCompletionRequest),
    CreateSpeechRequest(CreateSpeechRequest),
    CreateChatCompletionResponse(CreateChatCompletionResponse),
    CreateSpeechResponse(Bytes),
}

#[derive(Debug, Serialize)]
pub(crate) struct ServiceResponse {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Value,
}

#[derive(Debug)]
pub(crate) enum ServiceMessage {
    ClientRequest(HeaderMap, String, Bytes),
    OpenAiMessage(OpenAiMessage),
    ClientResponse(HeaderMap, Bytes),
}

impl From<CreateChatCompletionResponse> for ServiceMessage {
    fn from(response: CreateChatCompletionResponse) -> Self {
        ServiceMessage::OpenAiMessage(OpenAiMessage::CreateChatCompletionResponse(response))
    }
}

impl From<CreateSpeechResponse> for ServiceMessage {
    fn from(response: CreateSpeechResponse) -> Self {
        ServiceMessage::OpenAiMessage(OpenAiMessage::CreateSpeechResponse(response.bytes))
    }
}

impl From<CreateChatCompletionRequest> for ServiceMessage {
    fn from(request: CreateChatCompletionRequest) -> Self {
        ServiceMessage::OpenAiMessage(OpenAiMessage::CreateChatCompletionRequest(request))
    }
}

impl From<CreateSpeechRequest> for ServiceMessage {
    fn from(request: CreateSpeechRequest) -> Self {
        ServiceMessage::OpenAiMessage(OpenAiMessage::CreateSpeechRequest(request))
    }
}

impl AiService {
    pub(crate) fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }
    pub(crate) fn service(
        client: Arc<Client<OpenAIConfig>>,
        sender: MessageSender,
    ) -> impl Filter<Extract=(impl warp::Reply,), Error=warp::Rejection> + Clone {
        let service = Arc::new(Self { client, sender, next_id: AtomicU64::new(0) });

        warp::post() // Accept only POST requests...
            // ...at the root path...
            .and(warp::path::end())
            // ...with query parameters that suit RequestParams...
            .and(warp::query::<RequestParams>())
            .and(warp::query::raw())
            // ...and including the HTTP headers...
            .and(warp::header::headers_cloned())
            // ...regardless of the declared content type.
            .and(warp::body::bytes())
            // ...and pass along the service object itself.
            // (Necessary so that the closure in `and_then` can implement `Fn`.)
            .and(warp::any().map(move || service.clone()))
            // RetroArch declares application/x-www-form-urlencoded for its AI service requests,
            // but the body is actually JSON;
            // hence we deserialize explicitly because warp doesn't know how to handle this discrepancy.
            .and_then(|params: RequestParams, raw_params: String, headers: HeaderMap, body: Bytes, service: Arc<AiService>| async move {
                let request_id = service.next_id();
                log::info!(target: "groan", "{:?}", raw_params);

                if let Ok(request_body) = serde_json::from_slice::<RequestBody>(body.iter().as_slice()) {
                    log::info!(target: "groan", "{:?}", request_body);

                    let request = ServiceMessage::ClientRequest(headers, raw_params, body);
                    service.sender.send((request_id, request)).await.expect("TODO: panic message");

                    Ok((request_id, params, request_body, service))
                } else {
                    let request = ServiceMessage::ClientRequest(headers, raw_params, body);
                    service.sender.send((request_id, request)).await.expect("TODO: panic message");

                    Err(warp::reject::custom(InvalidRequestBody))
                }
            })
            // Then we untuple the parameters and body...
            .untuple_one()
            // query_service may run on another thread, possibly with multiple instances;
            // therefore we create the client in an `Arc` and clone it for each call to this endpoint
            .then(move |id, params, body, service| async move {
                AiService::query_service(id, service, params, body).await.unwrap_or_else(|e| {
                    log::error!(target: "groan", "{:?}", e);
                    ResponseBody::error(format!("{:?}", e))
                })
            })
            // Now that we've got the response, convert it to JSON...
            .map(|response| {
                warp::reply::json(&response)
            })
            .with(warp::trace::named("groan"))
    }

    async fn query_service(
        id: u64,
        service: Arc<AiService>,
        params: RequestParams,
        body: RequestBody,
    ) -> Result<ResponseBody, Box<dyn Error>> {
        match params
            .output
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .as_slice()
        {
            ["text", ..] => AiService::send_chat_request(id, service, params, body).await,
            ["sound", "wav", ..] => AiService::send_sound_request(id, service, params, body).await,
            ["image", "png", "png-a", ..] => Ok(ResponseBody::error("Not implemented")),
            _ => Ok(ResponseBody::error(format!("Unknown output format {:?}", params.output))),
        }
    }

    async fn chat_completion(
        id: u64,
        service: &Arc<AiService>,
        params: RequestParams,
        body: RequestBody,
    ) -> Result<CreateChatCompletionResponse, Box<dyn Error>> {
        let system = ChatCompletionRequestSystemMessageArgs::default()
            .content(
                "You are a narration service helping a visually impaired player \
                understand the scene for the game they're playing. \
                Describe the contents of the screenshots you will be given. \
                Limit your response to one sentence. \
                Do not use headings or explicit section makers. \
                Do not speculate about the image's contents. \
                Use video game terminology if appropriate.",
            ) // TODO: Make customizable
            .build()
            .map(ChatCompletionRequestMessage::System)?;

        let message = ChatCompletionRequestMessageContentPartImageArgs::default()
            .image_url(format!(
                "data:image/{:?};base64,{}",
                body.format.unwrap_or(ImageOutputFormat::Png),
                body.image
            ))
            .build()
            .map(ChatCompletionRequestMessageContentPart::ImageUrl)?;

        let user = ChatCompletionRequestUserMessageArgs::default()
            .content(vec![message])
            .build()
            .map(ChatCompletionRequestMessage::User)?;

        let request = CreateChatCompletionRequestArgs::default()
            .model("gpt-4o-mini") // TODO: Make customizable
            .max_tokens(300u32) // TODO: Make customizable
            .messages(vec![system, user])
            .build()?;

        service.sender.send((id, request.clone().into())).await?;
        service.client.chat().create(request).await.or_else(|e| Err(Box::new(e))?)
    }

    async fn send_chat_request(
        id: u64,
        service: Arc<AiService>,
        params: RequestParams,
        body: RequestBody,
    ) -> Result<ResponseBody, Box<dyn Error>> {
        let response = Self::chat_completion(id, &service, params, body).await?;
        service.sender.send((id, response.clone().into())).await?;
        log::info!(target: "groan", "{:?}", response);
        Ok(ResponseBody::text(response.choices[0].message.content.as_ref().ok_or("No content in response")?))
    }

    async fn send_sound_request(
        id: u64,
        service: Arc<AiService>,
        params: RequestParams,
        body: RequestBody,
    ) -> Result<ResponseBody, Box<dyn Error>> {
        let chat_response = Self::chat_completion(id, &service, params, body).await?;
        let text = chat_response.choices[0].message.content.as_ref().ok_or("No content in response")?;

        let request = CreateSpeechRequestArgs::default()
            .input(text)
            .model(Tts1) // TODO: Make customizable
            .voice(Fable) // TODO: Make customizable
            .response_format(Wav)
            .speed(1.1)
            .build()?;

        // OpenAI returns a WAV file with a subchunk2 size of -1
        // RetroArch's built-in WAV parser treats subchunks with a negative length as invalid
        // So we need to compute the length and fix the file
        let response = service.client.audio().speech(request).await.or_else(|e| Err(Box::new(e)))?;

        // This memory is already allocated;
        // ideally we can use it, but if not then we need to make our own copy
        let mut sound = response.bytes.try_into_mut().unwrap_or_else(BytesMut::from);
        let bytes_length = sound.len();

        // First subchunk2 size is at bytes 40-43
        let subchunk2size = sound.get_mut(40..44).ok_or("WAV file is too short")?;
        if i32::from_le_bytes(subchunk2size.try_into()?) == -1 {
            let length = (bytes_length - 44) as i32;
            log::debug!(target: "groan", "Returned audio's subchunk2size is -1; computed size is {}", length);
            subchunk2size.copy_from_slice(&length.to_le_bytes());
        }

        let bytes = sound.freeze();

        let response = ResponseBody::sound(&bytes);
        service.sender.send((id, ServiceMessage::OpenAiMessage(OpenAiMessage::CreateSpeechResponse(bytes)))).await?;
        Ok(response)
    }
}
