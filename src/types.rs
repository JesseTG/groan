use serde::{Deserialize, Serialize};
use std::fmt::Debug;

// Types based on descriptions given in https://docs.libretro.com/guides/ai-service/#for-developers

#[derive(Deserialize, Serialize, Debug, Clone)]
pub(crate) struct RequestParams {
    pub(crate) source_lang: Option<String>,
    pub(crate) target_lang: Option<String>,
    #[serde(with = "comma_separated_serialize")]
    pub(crate) output: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OutputFormat {
    Text,
    Sound(SoundOutputFormat),
    Image(Vec<ImageOutputFormat>),
}

#[derive(Deserialize, Serialize, Debug, Default)]
pub(crate) struct ResponseBody {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) image: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) sound: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) text_position: Option<TextPosition>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) press: Option<Vec<String>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) auto_request: Option<AutoRequest>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SoundOutputFormat {
    Wav,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ImageOutputFormat {
    Bmp,
    Png,
    #[serde(rename = "png-a")]
    PngA,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct RequestBody {
    pub(crate) image: String,
    pub(crate) format: Option<ImageOutputFormat>,
    pub(crate) coords: Option<(i32, i32, i32, i32)>,
    pub(crate) viewport: Option<(i32, i32)>,
    pub(crate) label: String,
    pub(crate) state: InputState,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum AutoRequest {
    Auto,
    Continue,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TextPosition {
    Bottom = 1,
    Top = 2,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct InputState {
    pub(crate) paused: u8,
    pub(crate) b: u8,
    pub(crate) y: u8,
    pub(crate) select: u8,
    pub(crate) start: u8,
    pub(crate) up: u8,
    pub(crate) down: u8,
    pub(crate) left: u8,
    pub(crate) right: u8,
    pub(crate) a: u8,
    pub(crate) x: u8,
    pub(crate) l: u8,
    pub(crate) r: u8,
    pub(crate) l2: u8,
    pub(crate) r2: u8,
    pub(crate) l3: u8,
    pub(crate) r3: u8,
}

#[derive(Debug)]
pub(crate) struct InvalidRequestBody;

impl warp::reject::Reject for InvalidRequestBody {}

impl ResponseBody {
    pub(crate) fn text<T>(text: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    pub(crate) fn error<T>(error: T) -> Self
    where
        T: Into<String>,
    {
        Self {
            error: Some(error.into()),
            ..Default::default()
        }
    }
}

impl Debug for RequestBody {
    // So that RequestBody can be printed in logs without an enormous base64 image.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RequestBody")
            .field("image", &"<base64-encoded image>")
            .field("format", &self.format)
            .field("coords", &self.coords)
            .field("viewport", &self.viewport)
            .field("label", &self.label)
            .field("state", &self.state)
            .finish()
    }
}
mod base64_serialize {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use serde::de::Error;
    use serde::{de, ser};
    use std::fmt;

    pub fn serialize<S>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(STANDARD.encode(data).as_str())
    }

    pub fn serialize_option<S>(data: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        match data {
            Some(data) => serializer.serialize_str(STANDARD.encode(data).as_str()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Vec<u8>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string that represents a base64-encoded image")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                STANDARD.decode(v).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

mod comma_separated_serialize {
    use serde::{de, ser};
    use std::fmt;

    pub fn serialize<S>(data: &Vec<String>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.collect_str(data.join(",").as_str())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = Vec<String>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a comma-separated list of strings")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(v.split(',').map(String::from).collect())
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
