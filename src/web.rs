use crate::ai::ServiceMessage::*;
use crate::ai::{MessageReceiver, ServiceRequest, ServiceResponse};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;
use warp::http::{HeaderMap, Response};
use warp::{Filter, Rejection};
use crate::types::{RequestBody, RequestParams, ResponseBody};

#[derive(Clone)]
pub(crate) struct WebConsoleService {
    cache: Arc<Mutex<MessageCache>>
}

#[derive(Default)]
pub(crate) struct MessageCache {
    client_requests: HashMap<u64, ServiceRequest>,
    openai_requests: HashMap<u64, crate::ai::OpenAiRequest>,
    openai_responses: HashMap<u64, crate::ai::OpenAiResponse>,
    client_responses: HashMap<u64, ServiceResponse>,
    request_images: HashMap<u64, Vec<u8>>,
}

const HTML: &str = include_str!(concat!(env!("OUT_DIR"), "/index.html"));
const JS: &str = include_str!(concat!(env!("OUT_DIR"), "/app.js"));
const CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/app.css"));

impl WebConsoleService {
    pub(crate) fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(MessageCache::default()))
        }
    }

    pub(crate) fn server_filter(self) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        let index_html = warp::get()
            .and(warp::path::end())
            .map(move || warp::reply::html(HTML));

        let style_css = warp::get().and(warp::path("app.css")).map(|| {
            Response::builder()
                .header("Content-Type", "text/css; charset=utf-8")
                .body(CSS)
                .unwrap()
        });

        let index_js = warp::get().and(warp::path("app.js")).map(|| {
            Response::builder()
                .header("Content-Type", "text/javascript; charset=utf-8")
                .body(JS)
                .unwrap()
        });

        let me = self.clone();
        let request = warp::path!("api" / "request" / u64)
            .and(warp::get())
            .and_then(move |id: u64| {
                let me = me.clone();
                async move {
                    if let Some(request) = me.cache.lock().await.client_requests.get(&id) {
                        let response = Response::builder()
                            .header("Content-Type", "application/json")
                            .body(serde_json::to_string(request).unwrap())
                            .unwrap();

                        return Ok(response);
                    }
                    else {
                        Err(warp::reject::not_found())
                    }
                }
            });

        let image = warp::path!("api" / "request" / u64 / "image")
            .and(warp::get())
            .and_then(move |id: u64| {
                let me = self.clone();
                async move {
                    if let Some(image) = me.cache.lock().await.request_images.get(&id) {
                        let image = image.clone();
                        let response = Response::builder()
                            .header("Content-Type", "image/png")
                            .body(image)
                            .unwrap();

                        return Ok(response);
                    }
                    else {
                        Err(warp::reject::not_found())
                    }
                }

            });

        // /api/request/:id/request.json
        // /api/request/:id/image
        // /api/request/:id/openai-request.json
        // /api/request/:id/openai-response.json
        // /api/request/:id/response.json

        let api = request
            .or(image);


        // TODO: Route for getting an image
        // TODO: Route for getting a sound clip
        // TODO: Route for getting a JSON blob

        warp::any()
            .and(index_html.or(index_js).or(style_css).or(api))
            .with(warp::trace::named("groan"))
    }

    async fn handle_client_request(&mut self, id: u64, headers: HeaderMap, params: String, body: Bytes) -> Result<(), Box<dyn Error>> {
        let body = serde_json::from_slice::<Value>(body.iter().as_slice())?;
        let image = body.get("image").and_then(|i| i.as_str()).ok_or("No image in body")?;
        let image = BASE64_STANDARD.decode(image.as_bytes())?;
        let headers = HashMap::from_iter(headers.iter().map(|h| (h.0.to_string(), h.1.to_str().unwrap().to_string())));

        let mut cache = self.cache.lock().await;
        assert!(!cache.client_requests.contains_key(&id));
        cache.client_requests.insert(id, ServiceRequest {headers, params, body});
        cache.request_images.insert(id, image);

        Ok(())
    }

    async fn handle_client_response(&mut self, id: u64, headers: HeaderMap, body: Bytes) -> Result<(), Box<dyn Error>> {
        let body = serde_json::from_slice::<Value>(body.iter().as_slice())?;
        let headers = HashMap::from_iter(headers.iter().map(|h| (h.0.to_string(), h.1.to_str().unwrap().to_string())));

        let mut cache = self.cache.lock().await;
        assert!(cache.client_requests.contains_key(&id));
        cache.client_responses.insert(id, ServiceResponse {headers, body});

        Ok(())
    }

    pub(crate) async fn poll_task(&mut self, mut receiver: MessageReceiver) {
        while let Some((id, message)) = receiver.recv().await {
            match message {
                ClientRequest(headers, params, body) => {
                    if let Err(e) = self.handle_client_request(id, headers, params, body).await {
                        log::error!("Error handling client request: {}", e);
                    }
                }
                OpenAiRequest(request) => {
                    let mut cache = self.cache.lock().await;
                    assert!(cache.client_requests.contains_key(&id));
                    cache.openai_requests.insert(id, request);
                }
                OpenAiResponse(response) => {
                    let mut cache = self.cache.lock().await;
                    assert!(cache.client_requests.contains_key(&id));
                    cache.openai_responses.insert(id, response);
                }
                ClientResponse(headers, body) => {
                    if let Err(e) = self.handle_client_response(id, headers, body).await {
                        log::error!("Error handling client response: {}", e);
                    }
                }
            }
        }
    }
}
