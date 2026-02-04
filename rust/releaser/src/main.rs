use env_logger::{Env, Target};
use errors_with_context::prelude::BooleanErrors;
use errors_with_context::{ErrorMessage, WithContext};
use log::{debug, info};
use std::env::{current_dir, home_dir};
use std::ffi::OsString;
use std::fs;
use std::fs::Metadata;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    env_logger::builder()
        .parse_env(Env::default().filter_or("RUST_LOG", "info"))
        .target(Target::Stdout)
        .format_timestamp_secs()
        .init();
    let dest_path = home_dir() //
        .with_err_context("Unable to get home directory")? //
        .join(".local")
        .join("bin");
    dest_path.exists().error_dyn_if_false(|| {
        format!("Destination path {} does not exist", dest_path.display())
    })?;
    debug!("Will copy to {}", dest_path.display());

    rust(&dest_path)?;
    python(&dest_path)?;
    shell(&dest_path)?;
    Ok(())
}

fn rust(dest_path: &PathBuf) -> Result<(), ErrorMessage> {
    let src_path = current_dir() //
        .with_err_context("Could not get current directory")? //
        .join("target")
        .join("release");
    src_path
        .exists()
        .error_dyn_if_false(|| format!("Source path {} does not exist", src_path.display()))?;
    debug!("Rust: Starting in {}", src_path.display());

    copy_files(&src_path, &dest_path, |metadata: Metadata, file_name: &OsString| {
        if !metadata.is_file() {
            return Err("is not a file");
        }
        if file_name.to_string_lossy().starts_with(".") {
            return Err("is a dotfile");
        }
        if file_name.to_string_lossy().ends_with(".d") {
            return Err("is a .d file");
        }
        Ok(())
    })?;
    Ok(())
}

fn python(dest_path: &PathBuf) -> Result<(), ErrorMessage> {
    let src_path = current_dir() //
        .with_err_context("Could not get current directory")? //
        .join("python");
    src_path
        .exists()
        .error_dyn_if_false(|| format!("Source path {} does not exist", src_path.display()))?;
    debug!("Python: Starting in {}", src_path.display());

    copy_files(&src_path, &dest_path, |metadata: Metadata, file_name: &OsString| {
        if !metadata.is_file() {
            return Err("is not a file");
        }
        if !file_name.to_string_lossy().ends_with(".py") {
            return Err("is not a .py file");
        }
        Ok(())
    })?;
    Ok(())
}

fn shell(dest_path: &PathBuf) -> Result<(), ErrorMessage> {
    let src_path = current_dir() //
        .with_err_context("Could not get current directory")? //
        .join("shell");
    src_path
        .exists()
        .error_dyn_if_false(|| format!("Source path {} does not exist", src_path.display()))?;
    debug!("Shell: Starting in {}", src_path.display());

    copy_files(&src_path, &dest_path, |metadata: Metadata, file_name: &OsString| {
        if !metadata.is_file() {
            return Err("is not a file");
        }
        if !file_name.to_string_lossy().ends_with(".sh") {
            return Err("is not a .sh file");
        }
        Ok(())
    })?;
    Ok(())
}

const COPY_RETRY_COUNT: usize = 3;

fn copy_files(
    src_path: &Path,
    dest_path: &Path,
    accepted: fn(Metadata, &OsString) -> Result<(), &str>,
) -> Result<(), ErrorMessage> {
    'outer: for dir in Path::new(&src_path)
        .read_dir()
        .with_dyn_err_context(|| format!("Failed to read directory {}", src_path.display()))?
    {
        let dir = dir.with_err_context("Failed to read directory entry")?;
        let path = dir.path();
        let file_name = dir.file_name();
        let metadata = dir
            .metadata()
            .with_dyn_err_context(|| format!("Could not get metadata {}", path.display()))?;
        debug!("Considering: {}", file_name.display());

        if let Err(message) = accepted(metadata, &file_name) {
            debug!("\trejected: {}", message);
            continue;
        }
        debug!("\taccepted");

        let target_exe_path = dest_path.join(file_name);

        info!("Copy {:60} -> {}", path.display(), target_exe_path.display());
        for _ in 0..COPY_RETRY_COUNT {
            match fs::copy(&path, &target_exe_path).with_err_context("Error copying file") {
                Ok(_) => continue 'outer,
                Err(error) => {
                    eprintln!(
                        "{}",
                        ErrorMessage::with_context("Error, retrying in half a second...", error)
                    );
                    sleep(Duration::from_millis(500))
                }
            }
        }
        ErrorMessage::err(format!("Error copying {path:?}, giving up"))?;
    }
    Ok(())
}
