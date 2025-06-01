use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use crate::error_message::ErrorMessage;

impl Serialize for ErrorMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("ErrorMessage", 2)?;
        s.serialize_field("message", &self.message)?;

        if let Some(cause) = &self.cause {
            if let Some(cause) = cause.downcast_ref::<ErrorMessage>() {
                s.serialize_field("cause", cause)?;
            } else {
                s.serialize_field("cause", &Some(ErrorMessage::new(cause.to_string())))?;
            }
        } else {
            s.serialize_field("cause", &None::<ErrorMessage>)?;
        }

        s.end()
    }
}
