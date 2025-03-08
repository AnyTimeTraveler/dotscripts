mod helpers;
use crate::helpers::run;
use serde_json::Value;
use std::error::Error;

/// ################################################
/// # Sway Monitor setup script by AnyTimeTraveler #
/// ################################################
///
/// This script sets up the monitor configurations based on which monitors are detected
/// It is meant to be used together with nwg-displays,
/// which outputs a chosen sway monitor configuration to the terminal and to a file in the path:
/// '~/.config/sway/outputs'
/// One is supposed to take that file and put it in the path, that is designated by the constant:
/// MONITOR_CONFIG_DIR
/// You can of course change that path in line 54 of this script.
/// You should name the file something meaningful, so you can later find the setup quickly
/// Afterward, you can come down to the MONITORS AND CONFIGURATIONS section,
/// specify the required monitors and apply the setup with the use_setup function.
/// It can also symlink the chosen configuration to the SWAY_CONFIG_DIR/outputs,
/// in case you want to start sway with

struct MonitorConfig {
    name_regex: Option<&'static str>,
    model_regex: Option<&'static str>,
    serial_regex: Option<&'static str>,
    make_regex: Option<&'static str>,
}
struct FoundMonitors {
    outputs: Vec<Value>,
}

impl FoundMonitors {
    async fn from_sway() -> Result<FoundMonitors, Box<dyn Error>> {
        let process_output = run("swaymsg", ["-t", "get_outputs"]).await?;
        let value: Value = serde_json::from_str(&process_output)?;
        if let Some(outputs) = value.as_array() {
            Ok(FoundMonitors { outputs: outputs.clone() })
        } else {
            Err("Output was not a JSON array".into())
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // ################################################
    // #         MONITORS AND CONFIGURATIONS          #
    // ################################################
    //

    // # Define your monitors
    // # Each monitor must be defined by at least 1 parameter, but more is better to avoid collisions
    // print("Detected Monitors:")
    // laptop_builtin = find_monitor(name_regex="eDP-1")
    // print(f"laptop_builtin: {laptop_builtin is not None}")
    // desk_center = find_monitor(model_regex="27GL650F", make_regex="LG Electronics")
    // print(f"desk_center: {desk_center is not None}")
    // desk_left = find_monitor(model_regex="LEN LT2452pwC", make_regex="Lenovo Group Limited")
    // print(f"desk_left: {desk_left is not None}")
    // desk_right = find_monitor(model_regex="S242HL", make_regex="Acer Technologies")
    // print(f"desk_right: {desk_right is not None}")
    //
    // dlr_left = find_monitor(make_regex="Dell Inc.", name_regex="DP-9")
    // print(f"dlr_left: {dlr_left is not None}")
    // dlr_right = find_monitor(make_regex="Dell Inc.", name_regex="DP-7")
    // print(f"dlr_right: {dlr_right is not None}")
    //
    // print("Choosing to use the following setup:")
    // # Define your setups based on which monitors were found
    // if laptop_builtin and desk_left and desk_center and desk_right:
    //     print("Home desk setup")
    //     use_setup(
    //         f"""output "{desk_center['name']}" {{
    //     mode  1920x1080@144.001Hz
    //     pos 1920 0
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // output "{desk_right['name']}" {{
    //     mode  1920x1080@60.0Hz
    //     pos 3840 0
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // output "{desk_left['name']}" {{
    //     mode  1920x1200@59.95Hz
    //     pos 0 0
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // output "{laptop_builtin['name']}" disable
    // """)
    // elif laptop_builtin and dlr_left and dlr_right:
    //     print("DLR desk setup")
    //     use_setup(
    //         f"""output "{laptop_builtin['name']}" {{
    //     mode  1920x1200@60.001Hz
    //     pos 1568 1440
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // output "{dlr_right['name']}" {{
    //     mode  2560x1440@59.951Hz
    //     pos 2560 0
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // output "{dlr_left['name']}" {{
    //     mode  2560x1440@59.951Hz
    //     pos 0 0
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // """)
    // elif laptop_builtin and desk_center:
    //     print("Laptop with screen above setup")
    //     use_setup(
    //             f"""output "{laptop_builtin['name']}" {{
    //     mode  1920x1200@60.001Hz
    //     pos 0 1080
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // output "{desk_center['name']}" {{
    //     mode  1920x1080@144.001Hz
    //     pos 0 0
    //     transform normal
    //     scale 1.0
    //     scale_filter nearest
    //     adaptive_sync off
    //     dpms on
    // }}
    // """)
    // elif laptop_builtin:
    //     print("Fallback laptop setup")
    //     x = 0
    //     setup_string = format_output(laptop_builtin, x)
    //     for output in outputs:
    //         if output != laptop_builtin:
    //             setup_string += format_output(output, x)
    //             x += output['modes'][0]['width']
    //     use_setup(setup_string)
    // else:
    //     print("Fallback desk setup")
    //     x = 0
    //     setup_string = ""
    //     for output in outputs:
    //         if output != laptop_builtin:
    //             setup_string += format_output(output, x)
    //             x += output['modes'][0]['width']
    //     use_setup(setup_string)
    Ok(())
}

macro_rules! check_monitor_parameter {
    ($expr: param_name, $expr: param) => {
        match ($param_name, output.get($param)) {
            (Some(param_regex), Some(param)) => {
                if !param.as_str()
                    .ok_or("Monitor $param is not a string!")?
                    .matches(param_regex) {
                    continue;
                }
            }
            (param_regex,param) => {
                Err(format!("Expected to match parameter '{:?}' with regex '{:?}', but either is missing",param, param_regex))
            }
        }
    };
}

impl FoundMonitors {
    async fn find_monitor(&self, monitor_config: MonitorConfig) -> Result<Option<String>, Box<dyn Error>> {
        for output in &self.outputs {
            let output = output.as_object().ok_or("Output is not an object")?;



            match (monitor_config.model_regex, output.get("model")) {
                (Some(model_regex), Some(model)) => {
                    if !model.as_str()
                        .ok_or("Monitor model is not a string!")?
                        .matches(model_regex) {
                        continue;
                    }
                }
                _ => {}
            }

            match (monitor_config.name_regex, output.get("name")) {
                (Some(name_regex), Some(name)) => {
                    if !name.as_str()
                        .ok_or("Monitor name is not a string!")?
                        .matches(name_regex) {
                        continue;
                    }
                }
                _ => {}
            }

            match (monitor_config.name_regex, output.get("name")) {
                (Some(name_regex), Some(name)) => {
                    if !name.as_str()
                        .ok_or("Monitor name is not a string!")?
                        .matches(name_regex) {
                        continue;
                    }
                }
                _ => {}
            }
        }
        for monitor in outputs:
        if name_regex
        and
        not
        re.search(name_regex, monitor['name']):
        continue;
        if model_regex
        and
        not
        re.search(model_regex, monitor['model']):
        continue;
        if serial_regex
        and
        not
        re.search(serial_regex, monitor['serial']):
        continue;
        if make_regex
        and
        not
        re.search(make_regex, monitor['make']):
        continue;
        return monitor;
        return None;
    }
}
// def find_monitor(
//         name_regex: Optional[str] = None,
//         model_regex: Optional[str] = None,
//         serial_regex: Optional[str] = None,
//         make_regex: Optional[str] = None,
// ) -> Optional[dict]:
//     for monitor in outputs:
//         if name_regex and not re.search(name_regex, monitor['name']):
//             continue
//         if model_regex and not re.search(model_regex, monitor['model']):
//             continue
//         if serial_regex and not re.search(serial_regex, monitor['serial']):
//             continue
//         if make_regex and not re.search(make_regex, monitor['make']):
//             continue
//         return monitor
//     return None
//
//
// def use_setup(monitor_config_commands: str):
//     monitor_config_commands = re.sub(r"^# .*$", "", monitor_config_commands, flags=re.MULTILINE)
//     monitor_config_commands = re.sub(r"[{}\n]", "", monitor_config_commands)
//     monitor_config_commands = monitor_config_commands.replace("output", ", output")
//     monitor_config_commands = monitor_config_commands.removeprefix(", ")
//
//     # print("Running:", monitor_config_commands)
//     # Apply the new config directly
//     command = run(["swaymsg", "--", monitor_config_commands], stdout=PIPE)
//     command.check_returncode()
//     command_output = json.loads(command.stdout)
//     for output in command_output:
//         if not output["success"]:
//             raise Exception("Error applying new monitor configuration!", command_output)
//     print("Monitor configuration successfully applied!")
//     exit(0)
//
//
// def format_output(output: dict, x_offset: int) -> str:
//     name = output['name']
//     rect = output['modes'][0]
//     return f"""output "{name}" {{
//     mode  {rect['width']}x{rect['height']}@{float(rect['refresh']) / 1000.0}Hz
//     pos {x_offset} 0
//     transform normal
//     scale 1.0
//     scale_filter nearest
//     adaptive_sync off
//     dpms on
// }}
// """
//
