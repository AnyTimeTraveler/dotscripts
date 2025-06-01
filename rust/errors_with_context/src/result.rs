use crate::error_message::ErrorMessage;
use crate::WithContext;
use std::error::Error;

impl<T, E: Error + 'static> WithContext<T, E> for Result<T, E> {
    fn with_err_context(self, context: impl AsRef<str>) -> Result<T, ErrorMessage> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                let message = context.as_ref().to_owned();
                Err(ErrorMessage::with_context(message, error))
            }
        }
    }

    fn with_dyn_err_context(self, context: impl FnOnce() -> String) -> Result<T, ErrorMessage> {
        match self {
            Ok(value) => Ok(value),
            Err(error) => {
                let message = context();
                Err(ErrorMessage::with_context(message, error))
            }
        }
    }
}
