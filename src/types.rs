use crate::serialize::{JsonObjectError, StructWrapper};
use serde::Serialize;
use serde_json::{Map, Value};
use snafu::{ResultExt, Snafu};

#[derive(Snafu, Debug)]
pub enum BuildError {
    ConvertJsonMapError { source: JsonObjectError },
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

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum Sound {
    Regular(String),
    Critical {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(serialize_with = "crate::serialize::se_bool_as_u8")]
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

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct APS {
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
    #[serde(serialize_with = "crate::serialize::se_bool_as_u8")]
    pub content_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "crate::serialize::se_bool_as_u8")]
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
    pub stale_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_state: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissal_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Map<String, Value>>,
}

impl APS {
    pub fn with_content_state<T: Serialize>(mut self, state: T) -> Result<Self, BuildError> {
        self.content_state = Some(
            StructWrapper(state)
                .try_into()
                .context(ConvertJsonMapSnafu)?,
        );
        Ok(self)
    }

    pub fn with_attributes<T: Serialize>(mut self, attributes: T) -> Result<Self, BuildError> {
        self.attributes = Some(
            StructWrapper(attributes)
                .try_into()
                .context(ConvertJsonMapSnafu)?,
        );
        Ok(self)
    }
}

#[derive(Serialize, Default, Debug)]
pub struct Notification {
    pub aps: APS,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    pub custom: Option<Map<String, Value>>,
}

impl Notification {
    pub fn with_custom<T: Serialize>(mut self, custom: T) -> Result<Self, BuildError> {
        self.custom = Some(
            StructWrapper(custom)
                .try_into()
                .context(ConvertJsonMapSnafu)?,
        );
        Ok(self)
    }
}

mod tests {
    #[test]
    fn test_empty() {
        use crate::types::APS;

        let aps = APS::default();
        let json = serde_json::to_string(&aps).unwrap();
        assert_eq!("{}", json);
    }

    #[test]
    fn test_filled() {
        use crate::types::{APS, Alert, InterruptionLevel, Sound, Subtitle, Title};

        let aps = APS {
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
            ..APS::default()
        };
        let json = serde_json::to_string(&aps).unwrap();
        assert_eq!(
            "{\"alert\":{\"title\":\"Title\",\"subtitle-loc-key\":\"SUBTITLE_KEY\"},\"sound\":{\"critical\":1},\"mutable-content\":1,\"interruption-level\":\"time-sensitive\"}",
            json
        )
    }

    #[test]
    fn test_custom_payload() {
        use crate::types::Notification;
        use serde::Serialize;

        #[derive(Serialize)]
        struct Custom {
            payload: String,
        }

        let custom = Custom {
            payload: String::from("payload"),
        };
        let notification = Notification {
            ..Notification::default()
        }
        .with_custom(custom)
        .unwrap();
        let json = serde_json::to_string(&notification).unwrap();
        assert_eq!("{\"aps\":{},\"payload\":\"payload\"}", json)
    }
}
