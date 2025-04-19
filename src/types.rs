use crate::serialize::{JsonObjectError, StructWrapper};
use serde::Serialize;
use serde_json::{Map, Value};
use snafu::{ResultExt, Snafu};

#[derive(Snafu, Debug)]
enum BuildError {
    ConvertJsonMapError { source: JsonObjectError },
}

#[derive(Serialize, Debug)]
enum Title {
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
enum Subtitle {
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
enum Body {
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
enum Alert {
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
enum Sound {
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
enum InterruptionLevel {
    Passive,
    Active,
    TimeSensitive,
    Critical,
}

#[derive(Serialize, Default, Debug)]
#[serde(rename_all = "kebab-case")]
struct APS {
    #[serde(skip_serializing_if = "Option::is_none")]
    alert: Option<Alert>,
    #[serde(skip_serializing_if = "Option::is_none")]
    badge: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sound: Option<Sound>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "crate::serialize::se_bool_as_u8")]
    content_available: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(serialize_with = "crate::serialize::se_bool_as_u8")]
    mutable_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    target_content_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    interruption_level: Option<InterruptionLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    relevance_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter_criteria: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stale_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_state: Option<Map<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dismissal_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<Map<String, Value>>,
}

impl APS {
    fn with_content_state<T: Serialize>(mut self, state: T) -> Result<Self, BuildError> {
        self.content_state = Some(
            StructWrapper(state)
                .try_into()
                .context(ConvertJsonMapSnafu)?,
        );
        Ok(self)
    }

    fn with_attributes<T: Serialize>(mut self, attributes: T) -> Result<Self, BuildError> {
        self.attributes = Some(
            StructWrapper(attributes)
                .try_into()
                .context(ConvertJsonMapSnafu)?,
        );
        Ok(self)
    }
}

#[derive(Serialize, Default, Debug)]
struct Notification {
    aps: APS,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(flatten)]
    custom: Option<Map<String, Value>>,
}

impl Notification {
    fn with_custom<T: Serialize>(mut self, custom: T) -> Result<Self, BuildError> {
        self.custom = Some(
            StructWrapper(custom)
                .try_into()
                .context(ConvertJsonMapSnafu)?,
        );
        Ok(self)
    }
}

mod tests {
    use crate::types::{APS, Alert, InterruptionLevel, Notification, Sound, Subtitle, Title};
    use serde::Serialize;

    #[test]
    fn test_empty() {
        let aps = APS::default();
        let json = serde_json::to_string(&aps).unwrap();
        assert_eq!("{}", json);
    }

    #[test]
    fn test_filled() {
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
