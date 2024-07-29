use std::sync::Arc;
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use crate::types::{RequestBody, RequestParams, ResponseBody};

pub(crate) async fn query_service(client: Arc<Client<OpenAIConfig>>, params: RequestParams, body: RequestBody) -> ResponseBody {
    match params.output.iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice() {
        ["text", ..] => ResponseBody::text("Not yet implemented."),
        ["sound", "wav", ..] => ResponseBody::error("Sound not implemented"),
        ["image", "png", "png-a", ..] => ResponseBody::error("Image not implemented"),
        _ => ResponseBody::error(format!("Unknown output format {:?}", params.output)),
    }
}