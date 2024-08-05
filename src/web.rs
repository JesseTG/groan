use crate::ai::ServiceMessage::*;
use crate::ai::{MessageReceiver, ServiceRequest, ServiceResponse};
use std::collections::HashMap;
use warp::http::Response;
use warp::{Filter, Rejection};

pub(crate) struct WebConsoleService {
    client_requests: HashMap<u64, ServiceRequest>,
    client_responses: HashMap<u64, ServiceResponse>,
}

const INDEX_HTML: &str = include_str!("../assets/index.html");
const INDEX_JS: &str = include_str!(concat!(env!("OUT_DIR"), "/index.js"));
const STYLE_CSS: &str = include_str!("../node_modules/eternium/eternium.css");

impl WebConsoleService {
    pub(crate) fn new() -> Self {
        Self {
            client_requests: Default::default(),
            client_responses: Default::default(),
        }
    }

    pub(crate) fn server_filter(
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = Rejection> + Clone {
        let index_html = warp::get()
            .and(warp::path::end())
            .map(move || warp::reply::html(INDEX_HTML));

        let style_css = warp::get().and(warp::path("style.css")).map(|| {
            Response::builder()
                .header("Content-Type", "text/css; charset=utf-8")
                .body(STYLE_CSS)
                .unwrap()
        });

        let index_js = warp::get().and(warp::path("index.js")).map(|| {
            Response::builder()
                .header("Content-Type", "text/javascript; charset=utf-8")
                .body(INDEX_JS)
                .unwrap()
        });

        warp::any().and(index_html.or(index_js).or(style_css))
    }

    pub(crate) async fn poll_task(&mut self, mut receiver: MessageReceiver) {
        while let Some((id, message)) = receiver.recv().await {
            log::info!(target: "groan", "{:?}", message);
            match message {
                ClientRequest(request) => {
                    assert!(!self.client_requests.contains_key(&id));
                    self.client_requests.insert(id, request);
                }
                OpenAiRequest => {}
                ClientResponse(response) => {
                    assert!(self.client_requests.contains_key(&id));
                    assert!(!self.client_responses.contains_key(&id));
                    self.client_responses.insert(id, response);
                }
                OpenAiResponse => {}
            }
        }
    }
}
