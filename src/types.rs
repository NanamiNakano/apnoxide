use crate::serialize::{JsonObjectError, StructWrapper};
use reqwest::header::HeaderMap;
use serde::Serialize;
use serde_json::{Map, Value};
use snafu::{ResultExt, Snafu};
use serde_with::{serde_as, BoolFromInt};

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum BuildError {
    ConvertJsonObjectError { source: JsonObjectError },
}

#[derive(Serialize, Debug)]
pub enum Title {
    #[serde(rename = "title")]
    Normal(String),
    #[serde(untagged)]
    Localized {
        #[serde(rename = "title-loc-key")]
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "title-loc-args")]
        args: Option<Vec<String>>,
    },
}

#[derive(Serialize, Debug)]
pub enum Subtitle {
    #[serde(rename = "subtitle")]
    Normal(String),
    #[serde(untagged)]
    Localized {
        #[serde(rename = "subtitle-loc-key")]
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "subtitle-loc-args")]
        args: Option<Vec<String>>,
    },
}

#[derive(Serialize, Debug)]
pub enum Body {
    #[serde(rename = "body")]
    Normal(String),
    #[serde(untagged)]
    Localized {
        #[serde(rename = "loc-key")]
        key: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "loc-args")]
        args: Option<Vec<String>>,
    },
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Alert {
    Body(String),
    Full {
        #[serde(flatten)]
        title: Option<Title>,
        #[serde(flatten)]
        subtitle: Option<Subtitle>,
        #[serde(flatten)]
        body: Option<Body>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "launch-image")]
        launch_image: Option<String>,
    },
}

#[serde_as]
#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Sound {
    Regular(String),
    Critical {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde_as(as = "Option<BoolFromInt>")]
        critical: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        volume: Option<f64>,
    },
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum InterruptionLevel {
    Passive,
    Active,
    TimeSensitive,
    Critical,
}

#[serde_as]
#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Notification {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert: Option<Alert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<Sound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<BoolFromInt>")]
    pub content_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<BoolFromInt>")]
    pub mutable_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_content_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interruption_level: Option<InterruptionLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_criteria: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_state: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissal_date: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Map<String, Value>>,
}

impl Notification {
    pub fn with_content_state<T: Serialize>(mut self, state: T) -> Result<Self, BuildError> {
        self.content_state = Some(
            StructWrapper(state)
                .try_into()
                .context(ConvertJsonObjectSnafu)?,
        );
        Ok(self)
    }

    pub fn with_attributes<T: Serialize>(mut self, attributes: T) -> Result<Self, BuildError> {
        self.attributes = Some(
            StructWrapper(attributes)
                .try_into()
                .context(ConvertJsonObjectSnafu)?,
        );
        Ok(self)
    }
}

#[derive(Serialize, Default, Debug)]
pub struct Payload {
    pub aps: Notification,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub custom: Option<Map<String, Value>>,
}

impl Payload {
    pub fn with_custom<T: Serialize>(mut self, custom: T) -> Result<Self, BuildError> {
        self.custom = Some(
            StructWrapper(custom)
                .try_into()
                .context(ConvertJsonObjectSnafu)?,
        );
        Ok(self)
    }
}

pub struct Endpoint {
    endpoint: String,
    port: u16,
}

impl From<Endpoint> for String {
    fn from(value: Endpoint) -> Self {
            format!("https://{}:{}", value.endpoint, value.port)
    }
}

impl TryFrom<String> for Endpoint {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.split(":").collect::<Vec<_>>();
        if value.len() != 2 {
            return Err(());
        };
        Ok(Self {
            endpoint: value[0].to_string(),
            port: value[1].parse().map_err(|_| ())?,
        })
    }
}

impl Endpoint {
    pub fn development() -> Self {
        Self {
            endpoint: "api.sandbox.push.apple.com".to_string(),
            port: 443,
        }
    }

    pub fn development_alter() -> Self {
        Self {
            endpoint: "api.sandbox.push.apple.com".to_string(),
            port: 2197,
        }
    }

    pub fn production() -> Self {
        Self {
            endpoint: "api.push.apple.com".to_string(),
            port: 443,
        }
    }

    pub fn production_alter() -> Self {
        Self {
            endpoint: "api.push.apple.com".to_string(),
            port: 2197,
        }
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self::production()
    }
}

#[derive(Default)]
pub struct PushOption<'a> {
    pub push_type: Option<&'a str>,
    pub id: Option<&'a str>,
    pub expiration: Option<u128>,
    pub priority: Option<u8>,
    pub topic: &'a str,
    pub collapse_id: Option<&'a str>,
}

impl TryFrom<PushOption<'_>> for HeaderMap {
    type Error = ();

    fn try_from(value: PushOption) -> Result<Self, Self::Error> {
        let mut headers = Self::new();
        if let Some(push_type) = value.push_type {
            headers.insert("apns-push-type", push_type.parse().map_err(|_| ())?);
        }
        if let Some(id) = value.id {
            headers.insert("apns-id", id.parse().map_err(|_| ())?);
        }
        if let Some(expiration) = value.expiration {
            headers.insert(
                "apns-expiration",
                expiration.to_string().parse().map_err(|_| ())?,
            );
        }
        if let Some(priority) = value.priority {
            headers.insert(
                "apns-priority",
                priority.to_string().parse().map_err(|_| ())?,
            );
        }
        if let Some(collapse_id) = value.collapse_id {
            headers.insert("apns-collapse-id", collapse_id.parse().map_err(|_| ())?);
        }
        headers.insert("apns-topic", value.topic.parse().map_err(|_| ())?);
        Ok(headers)
    }
}

#[cfg(test)]
mod tests {
    use serde::Serialize;
    use crate::{Alert, InterruptionLevel, Notification, Payload, Sound, Subtitle, Title};

    #[test]
    fn test_empty() {
        let aps = Notification::default();
        let json = serde_json::to_string(&aps).unwrap();
        assert_eq!("{}", json);
    }

    #[test]
    fn test_filled() {
        #[derive(Serialize)]
        struct Attr {
            attr: String,
        }

        let attr = Attr {
            attr: "foo".to_string(),
        };

        let aps = Notification {
            alert: Some(Alert::Full {
                title: Some(Title::Normal("Title".to_string())),
                subtitle: Some(Subtitle::Localized {
                    key: "SUBTITLE_KEY".to_string(),
                    args: None,
                }),
                body: None,
                launch_image: None,
            }),
            sound: Some(Sound::Critical {
                critical: Some(true),
                name: None,
                volume: None,
            }),
            mutable_content: Some(true),
            interruption_level: Some(InterruptionLevel::TimeSensitive),
            ..Notification::default()
        }
        .with_attributes(attr)
        .unwrap();
        let json = serde_json::to_string(&aps).unwrap();
        assert_eq!(
            "{\"alert\":{\"title\":\"Title\",\"subtitle-loc-key\":\"SUBTITLE_KEY\"},\"sound\":{\"critical\":1},\"mutable-content\":1,\"interruption-level\":\"time-sensitive\",\"attributes\":{\"attr\":\"foo\"}}",
            json
        )
    }

    #[test]
    fn test_custom_payload() {
        #[derive(Serialize)]
        struct Custom {
            payload: String,
        }

        let custom = Custom {
            payload: String::from("payload"),
        };
        let notification = Payload {
            ..Payload::default()
        }
        .with_custom(custom)
        .unwrap();
        let json = serde_json::to_string(&notification).unwrap();
        assert_eq!("{\"aps\":{},\"payload\":\"payload\"}", json)
    }
}
