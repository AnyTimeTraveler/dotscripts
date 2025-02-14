#![allow(dead_code)]
use std::error::Error;
use std::ffi::OsStr;
use std::fmt::Display;
use std::io::{stdout, Write};
use std::process::{ExitStatus, Output, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::select;

pub async fn run_simple(cmd: &str) -> Result<String, Box<dyn Error>> {
    let mut split = cmd.split(" ");
    let cmd = split.next().ok_or("No command supplied".to_owned())?;
    run(cmd, split).await
}

pub async fn run<I, S>(cmd: &str, args: I) -> Result<String, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let (status, output) = run_with_exit_status(cmd, args).await?;
    if status.success() {
        Ok(output)
    } else {
        Err(format!("Exited with output: '{}'", output).into())
    }
}

pub async fn run_with_exit_status<I, S>(
    cmd: &str,
    args: I,
) -> Result<(ExitStatus, String), Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let process_output = create_process(cmd, args).await?.wait_with_output().await?;
    let mut buffer = String::new();
    let status = capture_output(&mut buffer, process_output).await?;
    Ok((status, buffer))
}

pub async fn run_with_live_output<I, S, F, L>(
    cmd: &str,
    args: I,
    mut output_filter: F,
) -> Result<(ExitStatus, String), Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    F: FnMut(String) -> Option<L>,
    L: Display,
{
    let mut child = create_process(cmd, args).await?;

    let child_stdout = child.stdout.take().expect("Internal error, could not take stdout");
    let child_stderr = child.stderr.take().expect("Internal error, could not take stderr");

    let mut stdout_reader = BufReader::new(child_stdout).lines();
    let mut stderr_reader = BufReader::new(child_stderr).lines();

    let mut consumed = false;
    let mut buffer = String::new();
    while !consumed {
        let next_line = select! {
            line = stdout_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        Some(line)
                    }
                    Ok(None) => {
                        consumed = true;
                        None
                    }
                    Err(error) => { return Err(Box::new(error)); }
                }
            },
            line = stderr_reader.next_line() => {
                 match line {
                    Ok(Some(line)) => {
                        Some(line)
                    }
                    Ok(None) => {
                        consumed = true;
                        None
                    }
                    Err(error) => { return Err(Box::new(error)); }
                }
            },
        };
        if let Some(line) = next_line {
            buffer.push_str(&line);
            buffer.push('\n');
            if let Some(line) = output_filter(format!("{}\n", line)) {
                print!("{}", line);
                stdout().flush()?;
            }
        }
        if child.try_wait()?.is_some() {
            break;
        }
    }

    let process_output = child.wait_with_output().await?;

    let status = capture_output(&mut buffer, process_output).await?;
    Ok((status, buffer))
}


pub async fn run_with_inherited_stdio<I, S>(cmd: &str, args: I) -> Result<ExitStatus, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    Ok(child.wait().await?)
}

async fn capture_output(
    buffer: &mut String,
    process_output: Output,
) -> Result<ExitStatus, Box<dyn Error>> {
    let status = process_output.status;
    buffer.extend(process_output.stdout.into_iter().map(char::from));
    buffer.extend(process_output.stderr.into_iter().map(char::from));

    Ok(status)
}

async fn create_process<I, S>(cmd: &str, args: I) -> Result<Child, Box<dyn Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    Ok(child)
}

#[cfg(test)]
mod test {
    use crate::helpers::{run, run_with_live_output};

    #[tokio::test]
    async fn run_stdout() {
        let string = run("sh", ["-c", "echo test"]).await.unwrap();
        assert_eq!(string.trim(), "test");
    }

    #[tokio::test]
    async fn run_stderr() {
        let string = run("sh", ["-c", "echo test 1>&2"]).await.unwrap();
        assert_eq!(string.trim(), "test");
    }

    #[tokio::test]
    async fn run_stdout_live() {
        let (_status, string) =
            run_with_live_output("sh", ["-c", "echo test"], |_| None::<&str>).await.unwrap();
        assert_eq!(string.trim(), "test");
    }

    #[tokio::test]
    async fn run_stderr_live() {
        let (_status, string) =
            run_with_live_output("sh", ["-c", "echo test 1>&2"], |_| None::<&str>).await.unwrap();
        assert_eq!(string.trim(), "test");
    }
}
