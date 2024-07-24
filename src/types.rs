use serde::{Deserialize, Serialize};

// Types based on descriptions given in https://docs.libretro.com/guides/ai-service/#for-developers

#[derive(Deserialize, Serialize, Debug)]
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

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub(crate) enum SoundOutputFormat {
    Wav,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) enum ImageOutputFormat {
    #[serde(rename = "png")]
    Png,
    #[serde(rename = "png-a")]
    PngA,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct RequestBody {
    #[serde(with = "base64_serialize")]
    pub(crate) image: Vec<u8>,
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

mod base64_serialize {
    use serde::{de, ser};
    use std::fmt;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;
    use serde::de::Error;

    pub fn serialize<S>(data: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(STANDARD.encode(data).as_str())
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