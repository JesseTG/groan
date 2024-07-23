use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct RequestParams {
    source_lang: Option<String>,
    target_lang: Option<String>,
    output: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) enum CapturedFrameFormat {
    Png,
    Bmp,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct RequestBody {
    image: String, // TODO: Byte array encoded as base64
    format: CapturedFrameFormat,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct ViewpointCoords {}
