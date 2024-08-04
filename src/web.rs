use crate::ai::MessageReceiver;
use warp::http::Response;
use warp::{Filter, Rejection};

pub(crate) struct WebConsoleService {
    receiver: MessageReceiver, // TODO: Store requests and responses
}

const INDEX_HTML: &str = include_str!("../assets/index.html");
const INDEX_JS: &str = include_str!(concat!(env!("OUT_DIR"), "/index.js"));
const STYLE_CSS: &str = include_str!("../node_modules/eternium/eternium.css");

impl WebConsoleService {
    pub(crate) fn service(
        receiver: MessageReceiver,
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
}
