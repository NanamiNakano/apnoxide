use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use snafu::{ResultExt, Snafu};

#[allow(dead_code)]
pub(crate) fn se_bool_as_u8<S: Serializer>(x: &Option<bool>, s: S) -> Result<S::Ok, S::Error> {
    if let Some(x) = x {
        if *x {
            return s.serialize_u8(1);
        }
    }
    s.serialize_none()
}

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
