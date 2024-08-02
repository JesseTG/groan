use std::sync::Arc;
use warp::Filter;
use crate::ai::{MessageReceiver};

pub(crate) struct WebConsoleService {
    receiver: MessageReceiver
    // TODO: Store requests and responses
}

const INDEX_HTML: &str = include_str!("../assets/index.html");

impl WebConsoleService {
    pub(crate) fn service(
        receiver: MessageReceiver,
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::path::end()
            .map(move || warp::reply::html(INDEX_HTML))
    }
}