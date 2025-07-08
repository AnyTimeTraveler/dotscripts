//! ################################################
//! # Sway Monitor setup script by AnyTimeTraveler #
//! ################################################
//!
//! This script sets up the monitor configurations based on which monitors are detected
//! It is meant to be used together with nwg-displays,
//! which outputs a chosen sway monitor configuration to the terminal.is_some() && to a file in the path:
//! '~/.config/sway/outputs'
//! One is supposed to take that file.is_some() && put it in the path, that is designated by the constant:
//! MONITOR_CONFIG_DIR
//! You can of course change that path in line 54 of this script.
//! You should name the file something meaningful, so you can later find the setup quickly
//! Afterward, you can come down to the MONITORS.is_some() && CONFIGURATIONS section,
//! specify the required monitors.is_some() && apply the setup with the use_setup function.
//! It can also symlink the chosen configuration to the SWAY_CONFIG_DIR/outputs,
//! in case you want to start sway with

use errors_with_context::prelude::BooleanErrors;
use errors_with_context::{ErrorMessage, WithContext};
use process_utils::{run, run_with_exit_status};
use serde::Deserialize;
use serde_json::Value;

#[derive(Eq, PartialEq, Deserialize)]
struct Output {
    name: String,
    make: String,
    model: String,
    serial: String,
    modes: Vec<Mode>,
}

#[derive(Eq, PartialEq, Deserialize)]
struct Mode {
    width: u32,
    height: u32,
    refresh: u32,
}

#[derive(Deserialize)]
struct Outputs {
    outputs: Vec<Output>,
}

impl Outputs {
    async fn from_sway() -> Result<Outputs, ErrorMessage> {
        let process_output = run("swaymsg", ["-t", "get_outputs"]).await?;
        let outputs: Vec<Output> = serde_json::from_str(&process_output)
            .with_err_context("Failed to parse swaymsg outputs JSON")?;
        Ok(Outputs { outputs })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let monitors = Outputs::from_sway().await?;
    // ################################################
    // #         MONITORS.is_some() && CONFIGURATIONS          #
    // ################################################

    // # Define your monitors
    // # Each monitor must be defined by at least 1 parameter, but more is better to avoid collisions
    println!("Detected Monitors:");
    let laptop_builtin = monitors.find_monitor(
        OutputConfig::new() //
            .name_regex("eDP-1"), //
    );
    println!("laptop_builtin: {}", laptop_builtin.is_some());
    let desk_center = monitors.find_monitor(
        OutputConfig::new() //
            .model_regex("27GL650F") //
            .make_regex("LG Electronics"), //
    );
    println!("desk_center: {}", desk_center.is_some());
    let desk_left = monitors.find_monitor(
        OutputConfig::new() //
            .model_regex("LEN LT2452pwC") //
            .make_regex("Lenovo Group Limited"), //
    );
    println!("desk_left: {}", desk_left.is_some());
    let desk_right = monitors.find_monitor(
        OutputConfig::new() //
            .model_regex("S242HL") //
            .make_regex("Acer Technologies"), //
    );
    println!("desk_right: {}", desk_right.is_some());

    let dlr_left = any_of(vec![
        monitors.find_monitor(
            OutputConfig::new()
                .make_regex("Dell Inc.")
                .model_regex("DELL P2423DE")
                .serial_regex("J1VK1L3"),
        ),
    ]);
    println!("dlr_left: {}", dlr_left.is_some());
    let dlr_right = any_of(vec![
        monitors.find_monitor(
            OutputConfig::new()
                .make_regex("Dell Inc.")
                .model_regex("DELL P2423DE")
                .serial_regex("FLFL1L3"),
        ),
    ]);
    println!("dlr_right: {}", dlr_right.is_some());

    print!("Choosing to use the following setup: ");
    // # Define your setups based on which monitors were found
    let mut setup_string;
    if let (Some(builtin), Some(left), Some(center), Some(right)) =
        (laptop_builtin, desk_left, desk_center, desk_right)
    {
        println!("Home desk setup");
        let builtin_name = &builtin.name;
        let left_name = &left.name;
        let center_name = &center.name;
        let right_name = &right.name;

        setup_string = format!(
            r#"output "{center_name}" {{
        mode  1920x1080@144.001Hz
        pos 1920 0
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    output "{right_name}" {{
        mode  1920x1080@60.0Hz
        pos 3840 0
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    output "{left_name}" {{
        mode  1920x1200@59.95Hz
        pos 0 0
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    output "{builtin_name}" disable
    "#
        );
    } else if let (Some(builtin), Some(left), Some(right)) = (laptop_builtin, dlr_left, dlr_right) {
        println!("DLR desk setup");
        let builtin_name = &builtin.name;
        let right_name = &right.name;
        let left_name = &left.name;

        setup_string = format!(
            r#"output "{builtin_name}" {{
        mode  1920x1200@60.001Hz
        pos 1568 1440
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    output "{right_name}" {{
        mode  2560x1440@59.951Hz
        pos 2560 0
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    output "{left_name}" {{
        mode  2560x1440@59.951Hz
        pos 0 0
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    "#
        );
    } else if let (Some(builtin), Some(center)) = (laptop_builtin, desk_center) {
        println!("Laptop with screen above setup");
        let builtin_name = &builtin.name;
        let center_name = &center.name;

        setup_string = format!(
            r#"output "{builtin_name}" {{
        mode  1920x1200@60.001Hz
        pos 0 1080
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    output "{center_name}" {{
        mode  1920x1080@144.001Hz
        pos 0 0
        transform normal
        scale 1.0
        scale_filter nearest
        adaptive_sync off
        dpms on
    }}
    "#
        );
    } else if let Some(builtin) = laptop_builtin {
        println!("Fallback laptop setup");

        let mut x = 0;
        setup_string = builtin.format_output(x);

        for output in &monitors.outputs {
            if output != builtin {
                setup_string += &output.format_output(x);
                x += output.modes[0].width;
            }
        }
    } else {
        println!("Fallback setup");
        let mut x = 0;
        setup_string = String::new();
        for output in &monitors.outputs {
            setup_string += &output.format_output(x);
            x += output.modes[0].width;
        }
    }
    apply_setup(&setup_string)
        .await
        .with_err_context("Error applying new monitor configuration")?;
    Ok(())
}

fn any_of(outputs: Vec<Option<&Output>>) -> Option<&Output> {
    outputs.into_iter().filter_map(|a| a).next()
}

struct OutputConfig {
    name_regex: Option<&'static str>,
    model_regex: Option<&'static str>,
    make_regex: Option<&'static str>,
    serial_regex: Option<&'static str>,
}

impl OutputConfig {
    fn new() -> Self {
        Self { name_regex: None, model_regex: None, make_regex: None, serial_regex: None }
    }
    fn name_regex(mut self, name_regex: &'static str) -> Self {
        self.name_regex = Some(name_regex);
        self
    }
    fn model_regex(mut self, model_regex: &'static str) -> Self {
        self.model_regex = Some(model_regex);
        self
    }
    fn make_regex(mut self, make_regex: &'static str) -> Self {
        self.make_regex = Some(make_regex);
        self
    }
    #[allow(unused)]
    fn serial_regex(mut self, serial_regex: &'static str) -> Self {
        self.serial_regex = Some(serial_regex);
        self
    }
}

impl Outputs {
    fn find_monitor(&self, monitor_config: OutputConfig) -> Option<&Output> {
        for output in &self.outputs {
            if let Some(model_regex) = monitor_config.model_regex {
                if output.model.matches(model_regex).next().is_none() {
                    continue;
                }
            }
            if let Some(make_regex) = monitor_config.make_regex {
                if output.make.matches(make_regex).next().is_none() {
                    continue;
                }
            }
            if let Some(name_regex) = monitor_config.name_regex {
                if output.name.matches(name_regex).next().is_none() {
                    continue;
                }
            }
            if let Some(serial_regex) = monitor_config.serial_regex {
                if output.serial.matches(serial_regex).next().is_none() {
                    continue;
                }
            }
            return Some(output);
        }
        None
    }
}

impl Output {
    fn format_output(&self, x_offset: u32) -> String {
        let name = &self.name;
        let Mode { width, height, refresh } = &self.modes[0];
        let refresh = *refresh as f32 / 1000.0;

        format!(
            r#"output "{name}" {{
    mode  {width}x{height}@{refresh}Hz
    pos {x_offset} 0
    transform normal
    scale 1.0
    scale_filter nearest
    adaptive_sync off
    dpms on
}}
"#
        )
    }
}

async fn apply_setup(setup: &str) -> Result<(), ErrorMessage> {
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
    let results: Value = serde_json::from_str(&command_output).with_err_context("Expected json")?;
    for result in results.as_array().with_err_context("Expected array")? {
        result
            .get("success")
            .with_err_context("Exptected success")?
            .as_bool()
            .with_err_context("Expected bool")?
            .error_if_false("Expected success to be true")?;
    }

    println!("Monitor configuration successfully applied!");
    Ok(())
}
