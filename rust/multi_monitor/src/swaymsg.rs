use errors_with_context::{BooleanErrors, ErrorMessage, WithContext};
use serde::Deserialize;
use serde_json::Value;
use process_utils::{run, run_with_exit_status};
use crate::outputs::Mode;

#[derive(Clone, Deserialize)]
pub struct SwayOutput {
    pub(crate) name: String,
    pub(crate) make: String,
    pub(crate) model: String,
    pub(crate) serial: String,
    pub(crate) modes: Vec<Mode>,
}

pub(crate) async fn get_outputs() -> Result<Vec<SwayOutput>, ErrorMessage> {
    let process_output = run("swaymsg", ["-t", "get_outputs"]).await?;
    let outputs: Vec<SwayOutput> = serde_json::from_str(&process_output)
        .with_err_context("Failed to parse swaymsg outputs JSON")?;
    Ok(outputs)
}

pub(crate) async fn apply_setup(setup: String) -> Result<(), ErrorMessage> {
    // Remove all comments (lines starting with #)
    let mut setup: String = setup.lines().filter(|line| !line.starts_with("#")).collect();
    //     monitor_config_commands = re.sub(r"[{}\n]", "", monitor_config_commands)
    setup = setup
        .replace("{", "")
        .replace("}", "")
        .replace("\n", "")
        //     monitor_config_commands = monitor_config_commands.replace("output", ", output")
        .replace("output", ", output");
    //     monitor_config_commands = monitor_config_commands.removeprefix(", ")
    if setup.starts_with(", ") {
        setup = setup.replacen(", ", "", 1);
    }

    println!("Running: {setup}");
    // Apply the new config
    let (exit_status, command_output) = run_with_exit_status("swaymsg", ["--", &setup]).await?;

    exit_status
        .success()
        .error_if_false("Running the swaymsg command to apply the configuration failed")?;
    let results: Value =
        serde_json::from_str(&command_output).with_err_context("Expected json")?;
    for result in results.as_array().with_err_context("Expected array")? {
        result
            .get("success")
            .with_err_context("Expected success")?
            .as_bool()
            .with_err_context("Expected bool")?
            .error_if_false("Expected success to be true")?;
    }

    println!("Monitor configuration successfully applied!");
    Ok(())
}
