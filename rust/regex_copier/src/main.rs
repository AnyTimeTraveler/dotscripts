use colored::Colorize;
use regex::Regex;
use std::path::Path;
use tokio::fs::copy;

use clap::Parser;
use errors_with_context::prelude::BooleanErrors;
use errors_with_context::{ErrorMessage, WithContext};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    src_path: String,
    src_file_regex: String,
    target_formatstring: String,
    target_path: String,
    #[arg(long, action)]
    dry_run: bool,
    #[arg(long, action)]
    debug: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let Args {
        src_path,
        src_file_regex: src_regex,
        target_formatstring,
        target_path,
        dry_run,
        debug,
    } = Args::parse();

    let src_path = Path::new(&src_path);
    src_path.exists().error_if_false("Source path does not exist")?;

    let re = Regex::new(&src_regex).with_err_context("src_file_regex invalid")?;
    let target_path = Path::new(&target_path);
    target_path.exists().error_if_false("Target path does not exist")?;

    for dir in Path::new(src_path).read_dir().with_err_context("Failed to read directory")? {
        let dir = dir.with_err_context("Failed to read directory entry")?;
        if let Some(file_name) = dir.file_name().to_str() {
            if debug {
                println!("Consider: {}", file_name);
            }
            if let Some(captures) = re.captures(file_name) {
                if debug {
                    println!("Matches:  {}", captures.get(0).unwrap().as_str());
                }
                let mut target_formatted_string = target_formatstring.clone();
                for capture in captures.iter().flatten() {
                    target_formatted_string =
                        target_formatstring.replacen("{}", capture.as_str(), 1);
                }
                if target_formatted_string.contains("{}") {
                    ErrorMessage::err(format!(
                        "Target Formatstring still contains {{}}!
                    File: {file_name}
                    Partially filled in formatstring: {target_formatted_string}
                    "
                    ))?;
                }

                let source = Path::new(&target_formatted_string);
                let target = target_path.join(source.file_name().with_dyn_err_context(|| {
                    format!("Invalid file name '{:?}'", source.file_name())
                })?);
                if source.exists() {
                    if debug {
                        println!(
                            "Copying:  {}\tFrom: {}\tTo: {}",
                            file_name,
                            source.display(),
                            target.display()
                        );
                    }
                    if !dry_run {
                        copy(source, target).await.with_err_context("Copying failed")?;
                    }
                } else {
                    println!("{}", "Warning: Source does not exist".yellow());
                }
            }
        }
    }
    Ok(())
}
