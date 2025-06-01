use errors_with_context::{ErrorMessage, WithContext};
use std::ffi::OsStr;
use std::fmt::Display;
use std::io::{stdout, Write};
use std::process::{ExitStatus, Output, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::select;

pub async fn run_simple(cmd: &str) -> Result<String, ErrorMessage> {
    let mut split = cmd.split(" ");
    let cmd = split.next().with_err_context("No command supplied")?;
    run(cmd, split).await
}

pub async fn run<I, S>(cmd: &str, args: I) -> Result<String, ErrorMessage>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let (status, output) = run_with_exit_status(cmd, args).await?;
    if status.success() {
        Ok(output)
    } else {
        ErrorMessage::err(format!("Exited with output: '{}'", output))
    }
}

pub async fn run_with_exit_status<I, S>(
    cmd: &str,
    args: I,
) -> Result<(ExitStatus, String), ErrorMessage>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let process_output = create_process(cmd, args)
        .await?
        .wait_with_output()
        .await
        .with_dyn_err_context(|| format!("Failed to waiting for command '{cmd}' to complete"))?;
    let mut buffer = String::new();
    let status = capture_output(&mut buffer, process_output).await?;
    Ok((status, buffer))
}

pub async fn run_with_live_output<I, S, F, L>(
    cmd: &str,
    args: I,
    mut output_filter: F,
) -> Result<(ExitStatus, String), ErrorMessage>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    F: FnMut(String) -> Option<L>,
    L: Display,
{
    let mut child = create_process(cmd, args).await?;

    let child_stdout = child
        .stdout
        .take()
        .with_dyn_err_context(|| format!("Could not take stdout for process '{cmd}' to exit"))?;
    let child_stderr = child
        .stderr
        .take()
        .with_dyn_err_context(|| format!("Could not take stderr for process '{cmd}' to exit"))?;

    let mut stdout_reader = BufReader::new(child_stdout).lines();
    let mut stderr_reader = BufReader::new(child_stderr).lines();

    let mut consumed = false;
    let mut buffer = String::new();
    while !consumed {
        let next_line = select! {
            line = stdout_reader.next_line() => {
                match line.with_dyn_err_context(|| format!("Could not read next line from stdout for process '{cmd}'"))? {
                    Some(line) => {
                        Some(line)
                    }
                    None => {
                        consumed = true;
                        None
                    }
                }
            },
            line = stderr_reader.next_line() => {
                 match line.with_dyn_err_context(|| format!("Could not read next line from stderr for process '{cmd}'"))? {
                    Some(line) => {
                        Some(line)
                    }
                    None => {
                        consumed = true;
                        None
                    }
                }
            },
        };
        if let Some(line) = next_line {
            buffer.push_str(&line);
            buffer.push('\n');
            if let Some(line) = output_filter(format!("{}\n", line)) {
                print!("{}", line);
                stdout()
                    .flush()
                    .with_dyn_err_context(|| format!("Failed to flush output buffer of '{cmd}'"))?;
            }
        }
        if child
            .try_wait()
            .with_dyn_err_context(|| format!("Failed to wait for process '{cmd}' to exit"))?
            .is_some()
        {
            break;
        }
    }

    let process_output = child
        .wait_with_output()
        .await
        .with_dyn_err_context(|| format!("Failed to wait for process '{cmd}' to exit"))?;

    let status = capture_output(&mut buffer, process_output).await?;
    Ok((status, buffer))
}

pub async fn run_with_inherited_stdio<I, S>(cmd: &str, args: I) -> Result<ExitStatus, ErrorMessage>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_dyn_err_context(|| format!("Failed to spawn command '{cmd}'"))?;

    child
        .wait()
        .await
        .with_dyn_err_context(|| format!("Failed to waiting for command '{cmd}' to complete"))
}

async fn capture_output(
    buffer: &mut String,
    process_output: Output,
) -> Result<ExitStatus, ErrorMessage> {
    let status = process_output.status;
    buffer.extend(process_output.stdout.into_iter().map(char::from));
    buffer.extend(process_output.stderr.into_iter().map(char::from));

    Ok(status)
}

async fn create_process<I, S>(cmd: &str, args: I) -> Result<Child, ErrorMessage>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_dyn_err_context(|| format!("Failed to spawn command '{cmd}'"))?;
    Ok(child)
}

#[cfg(test)]
mod test {
    use crate::{run, run_with_live_output};

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
