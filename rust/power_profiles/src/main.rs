use errors_with_context::*;
use process_utils::run;
use std::env;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let current_status = get_profile().await?;
    let current_profile = current_status.trim();

    if let Some(arg) = env::args().nth(1) {
        match arg.as_str() {
            "toggle" => {
                let next_profile = next_profile(current_profile)?;
                set_profile(next_profile).await?;
                print_profile(next_profile)?;
            }
            "power-saver" => set_profile("power-saver").await?,
            "balanced" => set_profile("balanced").await?,
            "performance" => set_profile("performance").await?,
            unknown_arg => ErrorMessage::err(format!("Unknown argument '{}'", unknown_arg))?,
        }
    } else {
        print_profile(current_profile)?;
    }

    Ok(())
}

fn print_profile(current_status: &str) -> Result<(), ErrorMessage> {
    match current_status {
        "performance" => power_high(),
        "balanced" => power_normal(),
        "power-saver" => power_low(),
        unknown_status => {
            ErrorMessage::err(format!("Got unknown power setting: {}", unknown_status))?
        }
    }
    Ok(())
}

async fn get_profile() -> Result<String, ErrorMessage> {
    run("powerprofilesctl", ["get"]).await.with_err_context("Failed to get current power profile")
}

async fn set_profile(current_profile: &str) -> Result<(), ErrorMessage> {
    run("powerprofilesctl", ["set", current_profile])
        .await
        .with_dyn_err_context(|| format!("Failed to set profile '{current_profile}'"))?;
    Ok(())
}

fn next_profile(current_status: &str) -> Result<&'static str, ErrorMessage> {
    match current_status {
        "power-saver" => Ok("balanced"),
        "balanced" => Ok("performance"),
        "performance" => Ok("power-saver"),
        unknown_status => {
            ErrorMessage::err(format!("Got unknown power setting: {}", unknown_status))
        }
    }
}

fn power_high() {
    // bolt icon
    // let icon = "\u{f0e7}";
    // cloud up
    let icon = "\u{f0ee}";
    println!(r#"{{"state": "Good", "text": "{icon}"}}"#);
}

fn power_normal() {
    // circle icon
    // let icon = "\u{f22d}";
    // cloud blank
    let icon = "\u{f0c2}";
    println!(r#"{{"state": "Info", "text": "{icon}"}}"#);
}

fn power_low() {
    // cogs icon
    // let icon = "\u{f085}";
    // cloud down
    let icon = "\u{f0ed}";
    println!(r#"{{"state": "Idle", "text": "{icon}"}}"#);
}
