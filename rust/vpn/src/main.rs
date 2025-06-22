use errors_with_context::{ErrorMessage, WithContext};
use process_utils::{run, run_simple, run_with_exit_status};
use std::env;

const TIMEOUT_SECONDS: u64 = 1;
const MEGADRIVE_IP: &str = "172.16.0.1";
const VPN_IP: &str = "172.16.0.1";
const LOCAL_IP_SUBSTRING: &str = "192.168.1.";
const WIREGUARD_LOCAL_PROFILE: &str = "wg_local";
const WIREGUARD_GLOBAL_PROFILE: &str = "wg_global";

fn service_name(profile_name: &str) -> String {
    format!("wireguard-{}.service", profile_name)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let vpn_ping_time = tokio::spawn(ping(VPN_IP));
    let megadrive_reachable = tokio::spawn(ping(MEGADRIVE_IP));
    let home_ip_range = tokio::spawn(is_in_home_ip_range());
    let service_local = tokio::spawn(is_vpn_interface_active(WIREGUARD_LOCAL_PROFILE));
    let service_global = tokio::spawn(is_vpn_interface_active(WIREGUARD_GLOBAL_PROFILE));

    let vpn_ping_time =
        dbg!(vpn_ping_time.await.with_err_context("Failed to get ping time for VPN")??);
    let megadrive_reachable =
        dbg!(megadrive_reachable.await.with_err_context("Failed to get ping time for megadrive")??).is_some();
    let home_ip_range = home_ip_range.await.with_err_context("Failed to check for home IP range")??;
    let service_local = service_local
        .await
        .with_err_context("Failed to check if local wireguard service is active")??;
    let service_global = service_global
        .await
        .with_err_context("Failed to check if global wireguard service is active")??;

    let use_local_profile = use_local_profile(home_ip_range, megadrive_reachable);

    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "status" => {
                print_status(
                    megadrive_reachable,
                    home_ip_range,
                    service_local,
                    service_global,
                    vpn_ping_time,
                );
            }
            "toggle" => {
                if vpn_ping_time.is_some() {
                    stop_wg().await?;
                } else {
                    restart_wg(use_local_profile).await?;
                }
            }
            "start" => {
                restart_wg(use_local_profile).await?;
            }
            "stop" => {
                stop_wg().await?;
            }
            "global" => {
                stop_wg().await?;
                restart_wg(false).await?;
            }
            "local" => {
                stop_wg().await?;
                restart_wg(true).await?;
            }
            unknown_arg => ErrorMessage::err(format!("Unknown argument '{}'", unknown_arg))?,
        }
    } else {
        print_status(megadrive_reachable, home_ip_range, service_local, service_global, vpn_ping_time);
    }

    Ok(())
}

async fn stop_wg() -> Result<(), ErrorMessage> {
    run("sudo", ["systemctl", "stop", &service_name(WIREGUARD_LOCAL_PROFILE)]).await?;
    run("sudo", ["systemctl", "stop", &service_name(WIREGUARD_GLOBAL_PROFILE)]).await?;
    Ok(())
}

async fn restart_wg(use_local_profile: bool) -> Result<(), ErrorMessage> {
    if use_local_profile {
        run("sudo", ["systemctl", "restart", &service_name(WIREGUARD_LOCAL_PROFILE)]).await?;
    } else {
        run("sudo", ["systemctl", "restart", &service_name(WIREGUARD_GLOBAL_PROFILE)]).await?;
    }
    Ok(())
}

fn print_status(
    megadrive_reachable: bool,
    home_ip_range: bool,
    service_local: bool,
    service_global: bool,
    ping_time: Option<String>,
) {
    print!(r#"{{"state": ""#);
    if ping_time.is_some() {
        print!("Good")
    } else if service_global || service_local {
        print!("Warning")
    } else {
        print!("Critical")
    }
    // print!(r#"", "text": "C "#);
    print!(r#"", "text": ""#);
    if megadrive_reachable {
        print!(" ")
    } else {
        print!(" ")
    }
    if home_ip_range {
        print!(" ")
    } else {
        print!("")
    }
    print!("|");

    // print!("P");
    if service_local && service_global {
        print!("  ")
    } else if service_local {
        print!(" ")
    } else if service_global {
        print!(" ")
    }else {
        print!(" ")
    }
    print!(" ");

    if let Some(ping_time) = ping_time {
        print!("{ping_time} ms");
    } else {
        print!(" ");
    }
    println!(r#""}}"#);
}

/// Some: ping success with time
/// None: ping failed
async fn ping(target: &str) -> Result<Option<String>, ErrorMessage> {
    let (status, output) =
        run_with_exit_status("ping", ["-c", "1", "-w", &TIMEOUT_SECONDS.to_string(), target])
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

fn use_local_profile(is_in_home_ip_range: bool, megadrive_reachable: bool) -> bool {
    is_in_home_ip_range && megadrive_reachable
}

async fn is_in_home_ip_range() -> Result<bool, ErrorMessage> {
    let output = run_simple("ip a").await?;

    Ok(output.contains(LOCAL_IP_SUBSTRING))
}

async fn is_vpn_interface_active(interface: &'static str) -> Result<bool, ErrorMessage> {
    let output = run_simple("sudo wg").await?;

    Ok(output.contains(interface))
}
