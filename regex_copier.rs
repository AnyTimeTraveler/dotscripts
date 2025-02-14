use colored::Colorize;
use regex::Regex;
use std::error::Error;
use std::fs::copy;
use std::path::Path;

mod helpers;
use clap::Parser;

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
async fn main() -> Result<(), Box<dyn Error>> {
    let Args {
        src_path,
        src_file_regex: src_regex,
        target_formatstring,
        target_path,
        dry_run,
        debug,
    } = Args::parse();

    let src_path = Path::new(&src_path);
    if !src_path.exists() {
        Err("Source path does not exist")?;
    }
    let re = Regex::new(&src_regex)?;
    let target_path = Path::new(&target_path);
    if !target_path.exists() {
        Err("Target path does not exist")?;
    }

    for dir in Path::new(src_path).read_dir()? {
        let dir = dir?;
        if let Some(file_name) = dir.file_name().to_str() {
            if debug {
                println!("Consider: {}", file_name);
            }
            if let Some(captures) = re.captures(file_name) {
                if debug {
                    println!("Matches:  {}", captures.get(0).unwrap().as_str());
                }
                let mut target_formatted_string = target_formatstring.clone();
                for capture in captures.iter() {
                    if let Some(capture) = capture {
                        target_formatted_string =
                            target_formatstring.replacen("{}", capture.as_str(), 1);
                    }
                }
                if target_formatted_string.contains("{}") {
                    Err(format!(
                        "Target Formatstring still contains {{}}!
                    File: {}
                    Partially filled in formatstring: {}
                    ",
                        file_name, target_formatted_string
                    ))?;
                }

                let source = Path::new(&target_formatted_string);
                let target = target_path.join(source.file_name().ok_or("WTF")?);
                if source.exists() {
                    if debug {
                        println!("Copying:  {}\tFrom: {:?}\tTo: {:?}", file_name, &source, &target);
                    }
                    if !dry_run {
                        copy(source, target)?;
                    }
                } else {
                    println!("{}", "Warning: Source does not exist".yellow());
                }
            }
        }
    }
    Ok(())
}
