use std::collections::HashMap;
use crate::types::{
    ImageOutputFormat, InvalidRequestBody, RequestBody, RequestParams, ResponseBody,
};
use async_openai::config::OpenAIConfig;
use async_openai::types::{ChatCompletionRequestMessage, ChatCompletionRequestMessageContentPart, ChatCompletionRequestMessageContentPartImageArgs, ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequest, CreateChatCompletionRequestArgs, CreateChatCompletionResponse};
use async_openai::Client;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
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

#[derive(Debug)]
pub(crate) enum OpenAiRequest {
    CreateChatCompletionRequest(CreateChatCompletionRequest),
}

#[derive(Debug)]
pub(crate) enum OpenAiResponse {
    CreateChatCompletionResponse(CreateChatCompletionResponse),
}

#[derive(Debug)]
pub(crate) struct ServiceResponse {
    pub(crate) headers: HashMap<String, String>,
    pub(crate) body: Value,
}

#[derive(Debug)]
pub(crate) enum ServiceMessage {
    ClientRequest(HeaderMap, String, Bytes),
    OpenAiRequest(OpenAiRequest),
    OpenAiResponse(OpenAiResponse),
    ClientResponse(HeaderMap, Bytes),
}

impl From<CreateChatCompletionResponse> for ServiceMessage {
    fn from(response: CreateChatCompletionResponse) -> Self {
        ServiceMessage::OpenAiResponse(OpenAiResponse::CreateChatCompletionResponse(response))
    }
}

impl From<CreateChatCompletionRequest> for ServiceMessage {
    fn from(request: CreateChatCompletionRequest) -> Self {
        ServiceMessage::OpenAiRequest(OpenAiRequest::CreateChatCompletionRequest(request))
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
            .then(move |id, params, body, service| AiService::query_service(id, service, params, body))
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
    ) -> ResponseBody {
        match params
            .output
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .as_slice()
        {
            ["text", ..] => AiService::send_chat_request(id, service, params, body).await,
            ["sound", "wav", ..] => ResponseBody::error("Sound not implemented"),
            ["image", "png", "png-a", ..] => ResponseBody::error("Image not implemented"),
            _ => ResponseBody::error(format!("Unknown output format {:?}", params.output)),
        }
    }

    async fn send_chat_request(
        id: u64,
        service: Arc<AiService>,
        params: RequestParams,
        body: RequestBody,
    ) -> ResponseBody {
        let system = ChatCompletionRequestSystemMessageArgs::default()
            .content(
                "You are a narration service helping a visually impaired player \
            understand the scene for the game they're playing. \
            Describe the contents of the base64-encoded screenshots you will be given. \
            Your response will be read aloud by a text-to-speech system; \
            limit your response to at most two sentences. \
            Do not use headings or explicit section makers. \
            Do not speculate about the image's contents.",
            ) // TODO: Make customizable
            .build()
            .map(ChatCompletionRequestMessage::System)
            .unwrap();

        let message = ChatCompletionRequestMessageContentPartImageArgs::default()
            .image_url(format!(
                "data:image/{:?};base64,{}",
                body.format.unwrap_or(ImageOutputFormat::Png),
                body.image
            ))
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

        service.sender.send((id, request.clone().into())).await.expect("TODO: panic message");

        match service.client.chat().create(request).await.as_ref() {
            Ok(response) => {
                service.sender.send((id, response.clone().into())).await.expect("TODO: panic message");
                log::info!(target: "groan", "{:?}", response);
                ResponseBody::text(response.choices[0].message.content.as_ref().unwrap())
            }
            Err(error) => ResponseBody::error(format!("Error: {:?}", error)),
        }
    }
}
