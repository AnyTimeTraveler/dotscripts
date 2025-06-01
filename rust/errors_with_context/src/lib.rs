//! # Why did my program crash?
//! I want a simple answer and would love kind of a stack trace.
//! I tried some popular crates and found that they are either too complex or too much work.
//! If the error is fatal, I just want to pass it up the stack and eventually either print or serialize it.
//! And that's exactly what this library does, while remaining as small as possible.
//!
//! It implements a single type [ErrorMessage] and a single trait [WithContext] implemented for [Result] and [Option].
//! It provides two functions:
//!  - [WithContext::with_err_context] which takes anything that can be converted to [&str]
//!  - [WithContext::with_dyn_err_context] which instead takes a closure that is only run in the error case
//!
//! No macros and no learning curve.
//! Wherever you use Rust's [?] operator, you now add `.with_context("What are you trying to do")?`
//!
//! The output is neatly formatted, almost like a stacktrace.
//!
//! ```rust
//! # use std::convert::Infallible;
//! # use errors_with_context::prelude::ErrorMessage;
//! # use errors_with_context::WithContext;
//! let result: Result<Infallible, _> = ErrorMessage::err("File not found".to_owned())
//!         .with_err_context("Failed to read file")
//!         .with_err_context("Failed to load configuration")
//!         .with_err_context("Failed to start the program");
//! ```
//! prints
//! ```text
//! Failed to start the program
//!   caused by: Failed to load configuration
//!   caused by: Failed to read file
//!   caused by: File not found
//! ```
//!
//! Real world example for a config script of mine:
//! ```rust,compile_fail
//! let process_output = run("swaymsg", ["-t", "get_outputs"]).await?;
//! let value: Value = serde_json::from_str(&process_output)
//!     .with_context("Failed to parse swaymsg outputs JSON")?;
//! ```
//! prints
//! ```text
//! Error: Failed to parse swaymsg outputs JSON
//!   caused by: Error("EOF while parsing a value", line: 41, column: 24)
//! ```
//!
//! Much nicer to understand what the program was doing and where to search :)
//!
//! # Features
//!
//! This crate includes two optional features.
//! `serde` enables serialization of [ErrorMessage]s with serde.
//! `boolean` enables the trait [boolean::BooleanErrors], which allows emitting [ErrorMessage]s based on boolean values.
//!
//! See the function `tests::test_serialize` for an example how an error is turned into json.


mod error_message;
mod result;
mod option;
#[cfg(feature = "serde")]
mod serde;
#[cfg(feature = "boolean")]
mod boolean;
#[cfg(test)]
mod tests;


pub use crate::error_message::ErrorMessage;

pub mod prelude {
    pub use super::error_message::ErrorMessage;
    pub use super::WithContext;
    #[cfg(feature = "boolean")]
    pub use super::boolean::BooleanErrors;
}

pub trait WithContext<T, E> {
    fn with_err_context(self, reason: impl AsRef<str>) -> Result<T, ErrorMessage>;
    fn with_dyn_err_context(self, reason: impl FnOnce() -> String) -> Result<T, ErrorMessage>;
}
