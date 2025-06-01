use errors_with_context::{ErrorMessage, WithContext};
use process_utils::{run, run_with_exit_status};
use std::env;

const TIMEOUT_SECONDS: u64 = 1;
const VPN_IP: &str = "172.16.0.1";
const WIREGUARD_SERVICE: &str = "wireguard-wg0.service";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let ping_time = ping().await?;

    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "status" => {
                print_status(ping_time);
            }
            "toggle" => {
                if ping_time.is_some() {
                    run("sudo", ["systemctl", "stop", WIREGUARD_SERVICE]).await?;
                } else {
                    run("sudo", ["systemctl", "restart", WIREGUARD_SERVICE]).await?;
                }
                println!(r#"{{"state": "Warning", "text": " "}}"#);
            }
            "start" => {
                run("sudo", ["systemctl", "restart", WIREGUARD_SERVICE]).await?;
                println!(r#"{{"state": "Warning", "text": " "}}"#);
            }
            "stop" => {
                run("sudo", ["systemctl", "stop", WIREGUARD_SERVICE]).await?;
                println!(r#"{{"state": "Warning", "text": " "}}"#);
            }
            unknown_arg => ErrorMessage::err(format!("Unknown argument '{}'", unknown_arg))?,
        }
    } else {
        print_status(ping_time);
    }

    Ok(())
}

fn print_status(ping_time: Option<String>) {
    if let Some(ping_time) = ping_time {
        println!(r#"{{"state": "Good", "text": "  {ping_time} ms"}}"#);
    } else {
        println!(r#"{{"state": "Critical", "text": " "}}"#);
    }
}

/// Some: ping success with time
/// None: ping failed
async fn ping() -> Result<Option<String>, ErrorMessage> {
    let (status, output) =
        run_with_exit_status("ping", ["-c", "1", "-w", &TIMEOUT_SECONDS.to_string(), VPN_IP])
            .await?;
    if status.success() {
        let (_, time_with_suffix) = output.split_once("time=")
            .with_dyn_err_context(|| format!("Expected output of successful ping command to contain string 'time='. Instead got:\n{}", output))?;
        let (time, _) = time_with_suffix.split_once(' ')
            .with_dyn_err_context(|| format!("Expected output of successful ping command to contain 'time=<TIME> ms'. Instead got:\n{}", output))?;
        Ok(Some(time.to_owned()))
    } else {
        Ok(None)
    }
}
