use crate::client::APNClientError::{HeaderError, InitializeError, SignError};
use crate::APNClientError::{APNError, InvalidResponseError};
use crate::{Endpoint, Payload, PushOption};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::header::ToStrError;
use serde::{Deserialize, Serialize};
use snafu::{OptionExt, ResultExt, Snafu};
use std::time;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum APNClientError {
    #[snafu(display("Error when initialize client: {}", msg))]
    InitializeError {
        msg: String,
    },
    #[snafu(display("Error when signing token: {}", msg))]
    SignError {
        msg: String,
    },
    SystemTimeError {
        source: time::SystemTimeError,
    },
    HTTPError {
        source: reqwest::Error,
    },
    #[snafu(display("Unable to parse header"))]
    HeaderError,
    #[snafu(display("Can not parse APN server response"))]
    InvalidResponseError,
    #[snafu(display("Error from APN server: {}", error.reason))]
    APNError {
        response: APNResponse,
        status: u16,
        error: APNErrorResponse,
    },
    ToStrError {
        source: ToStrError,
    },
}

#[derive(Debug)]
pub struct APNResponse {
    pub id: String,
    pub unique_id: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct APNErrorResponse {
    pub reason: String,
    pub timestamp: Option<u64>,
}

pub struct APNClientConfig {
    team_id: String,
    key_id: String,
    key: EncodingKey,
    endpoint: String,
}

#[derive(Serialize)]
pub struct APNTokenClaims {
    #[serde(rename = "iss")]
    pub issuer_team_id: String,
    #[serde(rename = "iat")]
    pub issued_at: u64,
}

impl APNClientConfig {
    pub fn new(
        team_id: &str,
        key_id: &str,
        key: &str,
        endpoint: Endpoint,
    ) -> Result<Self, APNClientError> {
        let key = EncodingKey::from_ec_pem(key.as_bytes()).map_err(|_| InitializeError {
            msg: "Unable to parse private key".to_string(),
        })?;
        Ok(Self {
            team_id: team_id.to_string(),
            key_id: key_id.to_string(),
            key,
            endpoint: endpoint.into(),
        })
    }
}

pub struct APNClient {
    config: APNClientConfig,
    token: Option<String>,
    signed_time: SystemTime,
    http_client: reqwest::Client,
}

impl APNClient {
    pub fn new(config: APNClientConfig) -> Result<Self, APNClientError> {
        Ok(Self {
            config,
            token: None,
            signed_time: SystemTime::now(),
            http_client: reqwest::Client::builder()
                .use_rustls_tls()
                .build()
                .map_err(|_| InitializeError {
                    msg: "Unable to initialize http client".to_string(),
                })?,
        })
    }

    fn sign(&mut self) -> Result<String, APNClientError> {
        if let Some(token) = self.token.clone() {
            let now = SystemTime::now();
            let duration = now
                .duration_since(self.signed_time)
                .context(SystemTimeSnafu)?;
            if duration < Duration::from_secs(60 * 20) {
                return Ok(token);
            }
        }

        let mut header = Header::new(Algorithm::ES256);
        header.kid = Some(self.config.key_id.clone());
        header.typ = None;
        let claims = APNTokenClaims {
            issuer_team_id: self.config.team_id.clone(),
            issued_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .context(SystemTimeSnafu)?
                .as_secs(),
        };
        let token = encode(&header, &claims, &self.config.key).map_err(|_| SignError {
            msg: "Unable to sign token".to_string(),
        })?;
        self.token = Some(token.clone());
        Ok(token)
    }

    pub async fn push(
        &mut self,
        payload: &Payload,
        device_token: &str,
        option: PushOption<'_>,
    ) -> Result<APNResponse, APNClientError> {
        let path = format!("{}/3/device/{}", &self.config.endpoint, device_token);
        let token = self.sign()?;
        let req = self
            .http_client
            .post(path)
            .bearer_auth(token)
            .headers(option.try_into().map_err(|_| HeaderError)?)
            .json(payload);
        let res = req.send().await.context(HTTPSnafu)?;
        let headers = res.headers();
        let id = String::from(
            headers
                .get("apns-id")
                .context(InvalidResponseSnafu)?
                .to_str()
                .context(ToStrSnafu)?,
        );
        let unique_id = match headers.get("apns-unique-id") {
            None => None,
            Some(value) => Some(value.to_str().context(ToStrSnafu)?.to_string()),
        };
        let apn_response = APNResponse { id, unique_id };
        let status = res.status().as_u16();
        match status {
            200 => Ok(apn_response),
            _ => {
                let error_response = res
                    .json::<APNErrorResponse>()
                    .await
                    .map_err(|_| InvalidResponseError)?;
                Err(APNError {
                    response: apn_response,
                    status,
                    error: error_response,
                })
            }
        }
    }
}
