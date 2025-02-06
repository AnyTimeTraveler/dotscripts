use crate::helpers::{run, run_with_exit_status, run_with_live_output};
use colored::Colorize;
use std::env::set_current_dir;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, LineWriter, Write};
use std::time::Duration;
use tokio::time::sleep;

mod helpers;

const NIX_REBUILD_ERROR_CONTEXT: i32 = 1;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    set_current_dir("/etc/nixos")?;

    run("subl", ["--wait", "."]).await?;
    println!("{}", "Waiting for sublime text to close...".green());

    wait_for_sublime_exit().await;

    format_nix_config().await?;
    run("git", ["add", "-A"]).await?;

    let there_are_changes = check_for_git_changes().await?;

    show_git_diff().await?;

    println!("{}", "NixOS Upgrade!".green());
    println!("{}", " 1. Rebuilding...".green());
    rebuild_nixos().await?;
    print!("{}", " 2. Collecting garbage...".green());
    collect_garbage().await?;
    // println!("{}", " 3. Optimizing nix-store...".green());
    // optimize_nix_store().await?;

    let current_generation = show_new_generation().await?;

    if there_are_changes {
        println!("{}", " 3. Committing and pushing...".green());
        commit_and_push(&current_generation).await?;
    }

    println!(
        "{}",
        "
    =====================
    = NixOS Rebuilt OK! =
    =====================
    "
        .green()
    );
    show_notification("NixOS Rebuilt OK!").await?;
    Ok(())
}

async fn show_notification<S>(message: S) -> Result<(), Box<dyn Error>>
where
    S: AsRef<str>,
{
    run("notify-send", ["-e", message.as_ref(), "--icon=software-update-available"]).await?;
    Ok(())
}

async fn commit_and_push(current_generation: &str) -> Result<(), Box<dyn Error>> {
    run("git", ["commit", "-m", current_generation]).await?;
    run("git", ["push"]).await?;
    Ok(())
}

async fn show_new_generation() -> Result<String, Box<dyn Error>> {
    let output = run("nixos-rebuild", ["list-generations"]).await?;
    let current_generation = output
        .lines()
        .find(|line| line.contains("current"))
        .ok_or("No current generation found".to_owned())?
        .to_owned();
    let mut split = current_generation.split_whitespace();
    if let (Some(number), Some(date), Some(nix_version), Some(kernel_version)) =
        (split.nth(0), split.nth(1), split.nth(1), split.nth(0))
    {
        println!(
            "{} {} ({}) Nix: {} Kernel: {}",
            "New generation:".green(),
            number,
            date,
            nix_version,
            kernel_version
        );
    } else {
        println!("{} {}", "New generation:".green(), current_generation);
    }
    Ok(current_generation)
}

#[allow(dead_code)]
async fn optimize_nix_store() -> Result<(), Box<dyn Error>> {
    run_with_live_output(
        "sudo",
        ["nix-store", "--optimise", "--log-format", "bar", "--cores", "0"],
        |line| Some(line),
    )
    .await?;
    Ok(())
}

async fn collect_garbage() -> Result<(), Box<dyn Error>> {
    let (status, output) = run_with_live_output(
        "sudo",
        ["nix-collect-garbage", "--delete-older-than", "30d"], //
        |line| {
            if line.starts_with("deleting '") || line.starts_with("removing stale link ") {
                Some(".".into())
            } else {
                Some(format!("\n{}", line.trim_end().purple()))
            }
        },
    )
    .await?;
    println!();
    if status.success() {
        Ok(())
    } else {
        println!("Error:\n{}", output.trim());
        println!("{}", "Aborting!".bright_purple());
        Err("Collecting garbage failed.".to_owned().into())
    }
}

async fn rebuild_nixos() -> Result<(), Box<dyn Error>> {
    let log = OpenOptions::new() //
        .create(true) //
        .truncate(true) //
        .write(true) //
        .open("nixos-rebuild.log")?;
    let mut log = LineWriter::new(log);
    let (status, output) = run_with_live_output(
        "sudo",
        ["nixos-rebuild", "switch", "--upgrade-all", "--show-trace"],
        |line| {
            (&mut log).write(line.as_bytes()).unwrap();
            if line.contains("error:") {
                Some(format!("\n{}", line.trim_end().bright_red()))
            } else if line.contains("activating the configuration...")
                || (line.starts_with("building ") && !line.starts_with("building '"))
                || line.starts_with("building...")
                || line.contains(" paths will be fetched (")
            {
                Some(format!("\n{}", line.trim_end().purple()))
            } else {
                Some(".".into())
            }
        },
    )
    .await?;
    println!(); // to finish up the inverted newline-pattern
    if status.success() {
        Ok(())
    } else {
        let mut print_context = 0;
        for line in output.lines() {
            if line.contains("error:") {
                println!("{}", line.bright_red());
                print_context = NIX_REBUILD_ERROR_CONTEXT;
            } else if print_context > 0 {
                print_context -= 1;
                println!("{}", line.red());
            }
        }
        println!("{}", "Aborting!".bright_purple());
        Err("Rebuilding failed.".to_owned().into())
    }
}

async fn show_git_diff() -> Result<(), Box<dyn Error>> {
    let output = run("git", ["diff", "--staged", "-U0", "--color=always"]).await?;

    print!("{}", "Diff:".green());
    if output.trim().is_empty() {
        println!(" <EMPTY>");
    } else {
        let border = "=====================".blue();
        println!("\n{}", border);
        println!("{}", &output);
        println!("{}", border);
    }
    Ok(())
}

async fn check_for_git_changes() -> Result<bool, Box<dyn Error>> {
    let output = run("git", ["diff", "--staged"]).await?;
    let there_are_changes = !output.trim().is_empty();
    if !there_are_changes {
        print!("{}", "No changes. Want to run anyway? [Yn] ".yellow());
        stdout().flush()?;
        let mut user_input = String::new();
        stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();
        if !user_input.is_empty() || user_input.to_lowercase() == "n" {
            return Err("Failed to check for git changes!".to_owned().into());
        }
    }
    Ok(there_are_changes)
}

async fn format_nix_config() -> Result<(), Box<dyn Error>> {
    let (status, output) = run_with_exit_status("alejandra", ["."]).await?;
    if !status.success() {
        for line in output.lines() {
            if line.contains("Failed!") || line.contains(" at ") {
                println!("{}", line.bright_red())
            } else {
                println!("{}", line)
            }
        }
        println!("{}", "Aborting!".bright_purple());
        Err("Formatting nix config failed!".to_owned().into())
    } else {
        Ok(())
    }
}

async fn wait_for_sublime_exit() {
    while let Ok(process_list) = run("pgrep", ["sublime"]).await {
        if process_list.trim().is_empty() {
            break;
        }
        sleep(Duration::from_millis(250)).await;
    }
}
