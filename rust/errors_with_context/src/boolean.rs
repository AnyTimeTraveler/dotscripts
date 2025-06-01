use crate::ErrorMessage;

pub trait BooleanErrors {
    fn error_if_false(self, context: impl AsRef<str>) -> Result<bool, ErrorMessage>;
    fn error_if_true(self, context: impl AsRef<str>) -> Result<bool, ErrorMessage>;
    fn error_dyn_if_false(self, context: impl FnOnce() -> String) -> Result<bool, ErrorMessage>;
    fn error_dyn_if_true(self, context: impl FnOnce() -> String) -> Result<bool, ErrorMessage>;
}

impl BooleanErrors for bool {
    fn error_if_false(self, context: impl AsRef<str>) -> Result<bool, ErrorMessage> {
        if self {
            Ok(self)
        } else {
            let message = context.as_ref().to_owned();
            Err(ErrorMessage::new(message))
        }
    }

    fn error_if_true(self, context: impl AsRef<str>) -> Result<bool, ErrorMessage> {
        if self {
            let message = context.as_ref().to_owned();
            Err(ErrorMessage::new(message))
        } else {
            Ok(self)
        }
    }

    fn error_dyn_if_false(self, context: impl FnOnce() -> String) -> Result<bool, ErrorMessage> {
        if self {
            Ok(self)
        } else {
            let message = context();
            Err(ErrorMessage::new(message))
        }
    }

    fn error_dyn_if_true(self, context: impl FnOnce() -> String) -> Result<bool, ErrorMessage> {
        if self {
            let message = context();
            Err(ErrorMessage::new(message))
        } else {
            Ok(self)
        }
    }
}
