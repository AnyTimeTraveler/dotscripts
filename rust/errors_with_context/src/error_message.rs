use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};

pub struct ErrorMessage {
    pub(crate) message: String,
    pub(crate) cause: Option<Box<dyn Error>>,
}

impl ErrorMessage {
    pub fn new(message: String) -> ErrorMessage {
        ErrorMessage { message, cause: None }
    }
    pub fn err<T>(message: String) -> Result<T, ErrorMessage> {
        Err(ErrorMessage { message, cause: None })
    }
    pub fn with_context<E: Error + 'static>(message: String, cause: E) -> ErrorMessage {
        ErrorMessage { message, cause: Some(Box::new(cause)) }
    }
}

impl Error for ErrorMessage {}

impl Display for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)?;
        if let Some(cause) = &self.cause {
            fmt_cause(cause, f)?;
        }
        Ok(())
    }
}

impl Debug for ErrorMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut s = f.debug_struct("ErrorMessage");
        s.field("message", &self.message);

        if let Some(cause) = &self.cause {
            if let Some(cause) = cause.downcast_ref::<ErrorMessage>() {
                s.field("cause", cause);
            } else {
                s.field("cause", &Some(cause));
            }
        } else {
            s.field("cause", &None::<ErrorMessage>);
        }

        s.finish()
    }
}

fn fmt_cause(error: &Box<dyn Error>, f: &mut Formatter) -> fmt::Result {
    f.write_str("\n  caused by: ")?;
    if let Some(cause) = error.downcast_ref::<ErrorMessage>() {
        f.write_str(&cause.message)?;
        if let Some(cause) = &cause.cause {
            fmt_cause(cause, f)?;
        }
    } else {
        Debug::fmt(error, f)?;
    }
    Ok(())
}
