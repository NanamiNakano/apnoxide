use serde::Serialize;
use serde_json::{Map, Value};
use snafu::{ResultExt, Snafu};

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum JsonObjectError {
    SerializationError { source: serde_json::Error },
    NotAnObjectError,
}

pub(crate) struct StructWrapper<T>(pub T);

impl<T: Serialize> TryFrom<StructWrapper<T>> for Map<String, Value> {
    type Error = JsonObjectError;

    fn try_from(converter: StructWrapper<T>) -> Result<Self, Self::Error> {
        let value = serde_json::to_value(converter.0).context(SerializationSnafu)?;

        match value {
            Value::Object(object) => Ok(object),
            _ => Err(JsonObjectError::NotAnObjectError),
        }
    }
}
