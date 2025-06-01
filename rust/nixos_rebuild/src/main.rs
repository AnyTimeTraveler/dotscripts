use colored::Colorize;
use std::env::set_current_dir;
use std::fs::OpenOptions;
use std::io::{stdin, stdout, LineWriter, Write};
use std::time::Duration;
use tokio::time::sleep;

use clap::Parser;
use errors_with_context::{ErrorMessage, WithContext};
use process_utils::{run, run_with_exit_status, run_with_inherited_stdio, run_with_live_output};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(long, default_value = "subl")]
    editor: String,
    #[arg(long, default_value = "--wait .")]
    editor_args: String,
    #[arg(long, default_value = "/etc/nixos")]
    nix_dir: String,
    #[arg(short, long, action)]
    optimize_store: bool,
    #[arg(long, action)]
    dry_run: bool,
    #[arg(long, action)]
    boot: bool,
    #[arg(long, action)]
    debug: bool,
}
const NIX_REBUILD_ERROR_CONTEXT: i32 = 1;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let Args { editor, editor_args, nix_dir, optimize_store, dry_run, boot, debug } = Args::parse();
    set_current_dir(nix_dir).with_err_context("Failed to cd to nix config directory")?;

    run(&editor, editor_args.split(" ")).await?;
    println!("{}", "Waiting for sublime text to close...".green());

    wait_for_editor_exit().await;

    format_nix_config().await.with_err_context("Failed to format nix config")?;

    run("git", ["add", "-A"]).await?;

    let there_are_changes =
        check_for_git_changes().await.with_err_context("Failed to check for git changes")?;

    show_git_diff().await.with_err_context("Failed to display diff of changes")?;

    println!("{}", "NixOS Upgrade!".green());
    println!("{}", " 1. Rebuilding...".green());
    rebuild_nixos(dry_run, debug, boot).await.with_err_context("Nixos rebuild failed")?;
    print!("{}", " 2. Collecting garbage...".green());
    collect_garbage(dry_run, debug).await.with_err_context("Collecting garbage failed")?;
    if optimize_store {
        println!("{}", " 3. Optimizing nix-store...".green());
        if dry_run {
            println!("{}", "   Not running, has no dry-run option".red())
        } else {
            optimize_nix_store().await.with_err_context("Failed to optimize nix-store")?;
        }
    }
    let current_generation = show_new_generation().await?;

    if there_are_changes {
        println!("{}", " 3. Committing and pushing...".green());
        commit_and_push(&current_generation).await.with_err_context("Failed to commit and push")?;
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
    show_notification("NixOS Rebuilt OK!").await.with_err_context("Failed to show notification")?;
    Ok(())
}

async fn show_notification<S>(message: S) -> Result<(), ErrorMessage>
where
    S: AsRef<str>,
{
    run("notify-send", ["-e", message.as_ref(), "--icon=software-update-available"]).await?;
    Ok(())
}

async fn commit_and_push(current_generation: &str) -> Result<(), ErrorMessage> {
    run("git", ["commit", "-m", current_generation]).await?;
    run("git", ["push"]).await?;
    Ok(())
}

async fn show_new_generation() -> Result<String, ErrorMessage> {
    let output = run("nixos-rebuild", ["list-generations"]).await?;
    let current_generation = output
        .lines()
        .find(|line| line.contains("current"))
        .with_err_context("No current generation found")?
        .to_owned();
    let mut split = current_generation.split_whitespace();
    if let (Some(number), Some(date), Some(nix_version), Some(kernel_version)) =
        (split.next(), split.nth(1), split.nth(1), split.next())
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

async fn optimize_nix_store() -> Result<(), ErrorMessage> {
    run_with_inherited_stdio(
        "sudo",
        ["nix-store", "--optimise", "--log-format", "bar", "--cores", "0"],
    )
    .await?;
    Ok(())
}

async fn collect_garbage(dry_run: bool, debug: bool) -> Result<(), ErrorMessage> {
    let mut args = vec!["nix-collect-garbage", "--delete-older-than", "30d"];
    if dry_run {
        args.push("--dry-run");
    }
    let (status, output) = run_with_live_output("sudo", args, |line| {
        if line.starts_with("deleting '") || line.starts_with("removing stale link ") {
            if debug { Some(format!("\n{}", line.trim_end().cyan())) } else { Some(".".into()) }
        } else {
            Some(format!("\n{}", line.trim_end().purple()))
        }
    })
    .await?;
    println!();
    if status.success() {
        Ok(())
    } else {
        println!("Error:\n{}", output.trim());
        println!("{}", "Aborting!".bright_purple());
        ErrorMessage::err("nix-collect-garbage failed.".to_owned())
    }
}

const NIX_LOG_FILENAME: &str = "nixos-rebuild.log";

async fn rebuild_nixos(dry_run: bool, debug: bool, on_boot: bool) -> Result<(), ErrorMessage> {
    let log = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(NIX_LOG_FILENAME)
        .with_dyn_err_context(|| format!("Failed to open '{NIX_LOG_FILENAME}'"))?;
    let mut log = LineWriter::new(log);
    let (status, output) = run_with_live_output(
        "sudo",
        if dry_run {
            ["nixos-rebuild", "dry-build", "--upgrade-all", "--show-trace"]
        } else if on_boot {
            ["nixos-rebuild", "boot", "--upgrade-all", "--show-trace"]
        } else {
            ["nixos-rebuild", "switch", "--upgrade-all", "--show-trace"]
        },
        |line| {
            log.write_all(line.as_bytes()).unwrap();
            if line.contains("error:") {
                Some(format!("\n{}", line.trim_end().bright_red()))
            } else if line.contains("activating the configuration...")
                || (line.starts_with("building ") && !line.starts_with("building '"))
                || line.starts_with("building...")
                || line.contains(" paths will be fetched (")
            {
                Some(format!("\n{}", line.trim_end().purple()))
            } else if debug {
                Some(format!("\n{}", line.trim_end().cyan()))
            } else {
                Some(".".into())
            }
        },
    )
    .await
    .with_err_context("Command nixos-rebuild exited non-cleanly")?;
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
        ErrorMessage::err("Rebuilding failed.".to_owned())
    }
}

async fn show_git_diff() -> Result<(), ErrorMessage> {
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

async fn check_for_git_changes() -> Result<bool, ErrorMessage> {
    let output = run("git", ["diff", "--staged"]).await?;
    let there_are_changes = !output.trim().is_empty();
    if !there_are_changes {
        print!("{}", "No changes. Want to run anyway? [Yn] ".yellow());
        stdout().flush().with_err_context("Failed to flush stdout")?;
        let mut user_input = String::new();
        stdin().read_line(&mut user_input).with_err_context("Failed to read stdin")?;
        let user_input = user_input.trim();
        if !user_input.is_empty() || user_input.to_lowercase() == "n" {
            return ErrorMessage::err("User aborted program.".to_owned());
        }
    }
    Ok(there_are_changes)
}

async fn format_nix_config() -> Result<(), ErrorMessage> {
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
        ErrorMessage::err("Formatting nix config failed!".to_owned())
    } else {
        Ok(())
    }
}

async fn wait_for_editor_exit() {
    while let Ok(process_list) = run("pgrep", ["sublime"]).await {
        if process_list.trim().is_empty() {
            break;
        }
        sleep(Duration::from_millis(250)).await;
    }
}
