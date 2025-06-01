use crate::error_message::ErrorMessage;
use crate::WithContext;
use std::convert::Infallible;
use std::fs::File;
use std::io;
use std::io::ErrorKind;
use std::path::Path;

#[test]
fn test_no_reason() {
    let message_text = "I/O Error";
    let message = ErrorMessage::new(message_text.to_owned());
    let message_string = message.to_string();

    println!("test_no_reason():\n{}\n", message_string);
    assert_eq!(message_string, message_text);
}

#[test]
fn test_result_multiple_reasons() {
    let message_text = "I/O Error";
    let result: Result<Infallible, _> = ErrorMessage::err(message_text.to_owned())
        .with_err_context("Failed to read file")
        .with_err_context("Failed to load configuration")
        .with_err_context("Failed to start the program");
    let message = result.expect_err("Created an error and didn't get an error");
    let message_string = message.to_string();

    println!("test_result_multiple_reasons():\n{}\n", message_string);
    assert_eq!(message_string, r#"Failed to start the program
  caused by: Failed to load configuration
  caused by: Failed to read file
  caused by: I/O Error"#)
}

#[test]
fn test_option_single_reason() {
    let result: Result<Infallible, _> = None
        .with_err_context("Configuration value not found");
    let message = result.expect_err("Created an error and didn't get an error");
    let message_string = message.to_string();

    println!("test_option_single_reason():\n{}\n", message_string);
    assert_eq!(message_string, "Configuration value not found")
}



#[test]
fn test_functions() {
    fn innermost() -> io::Result<()> {
        Err(io::Error::new(ErrorKind::NotFound, "Test message"))
    }
    fn load_config() -> Result<(), ErrorMessage> {
        innermost()
            .with_err_context("Failed to load config")
    }
    fn run_program() -> Result<(), ErrorMessage> {
        let file = Path::new("./config");
        load_config()
            .with_err_context(format!("Failed to start program with config {:?}", file))?;
        Ok(())
    }

    let message = run_program()
        .expect_err("Created an error and didn't get an error");
    let message_string = message.to_string();

    println!("test_functions():\n{}\n", message_string);
    assert_eq!(message_string, r#"Failed to start program with config "./config"
  caused by: Failed to load config
  caused by: Custom { kind: NotFound, error: "Test message" }"#)
}

#[test]
#[cfg(feature = "serde")]
fn test_serialize_custom_base_error() {
    let message_text = "I/O Error";
    let result: Result<Infallible, _> = ErrorMessage::err(message_text.to_owned())
        .with_err_context("Failed to read file")
        .with_err_context("Failed to load configuration")
        .with_err_context("Failed to start the program");
    let message = result.expect_err("Created an error and didn't get an error");
    let json = serde_json::to_string_pretty(&message)
        .expect("Conversion to json failed");

    println!("test_serialize():\n{}\n", json);
    assert_eq!(json, r#"{
  "message": "Failed to start the program",
  "cause": {
    "message": "Failed to load configuration",
    "cause": {
      "message": "Failed to read file",
      "cause": {
        "message": "I/O Error",
        "cause": null
      }
    }
  }
}"#)
}

#[test]
#[cfg(feature = "serde")]
fn test_serialize() {
    let result: Result<Infallible, _> = Err(io::Error::from(ErrorKind::NotFound))
        .with_err_context("Failed to read file")
        .with_err_context("Failed to load configuration")
        .with_err_context("Failed to start the program");
    let message = result.expect_err("Created an error and didn't get an error");
    let json = serde_json::to_string_pretty(&message)
        .expect("Conversion to json failed");

    println!("test_serialize():\n{}\n", json);
    assert_eq!(json, r#"{
  "message": "Failed to start the program",
  "cause": {
    "message": "Failed to load configuration",
    "cause": {
      "message": "Failed to read file",
      "cause": {
        "message": "I/O Error",
        "cause": null
      }
    }
  }
}"#)
}
