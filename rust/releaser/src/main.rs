use errors_with_context::prelude::BooleanErrors;
use errors_with_context::{ErrorMessage, WithContext};
use log::{debug, info};
use std::env::{current_dir, home_dir};
use std::fs;
use std::path::Path;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    env_logger::init();
    let src_path = current_dir() //
        .with_err_context("Could not get current directory")? //
        .join("target/release");
    src_path
        .exists()
        .error_dyn_if_false(|| format!("Source path {} does not exist", src_path.display()))?;
    debug!("Starting in {}", src_path.display());

    let dest_path = home_dir() //
        .with_err_context("Unable to get home directory")? //
        .join(".local/bin");
    dest_path.exists().error_dyn_if_false(|| {
        format!("Destination path {} does not exist", dest_path.display())
    })?;
    debug!("Will copy to {}", dest_path.display());

    for dir in Path::new(&src_path)
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

        if !metadata.is_file() {
            // only consider files
            debug!("\tis not a file");
            continue;
        }

        if file_name.to_string_lossy().starts_with(".") {
            // skip dotfiles
            debug!("\tis a dotfile");
            continue;
        }
        if file_name.to_string_lossy().ends_with(".d") {
            // skip .d files
            debug!("\tis a .d file");
            continue;
        }
        debug!("\taccepted");

        let target_exe_path = dest_path.join(file_name);

        info!("Copying {} to {}", path.display(), target_exe_path.display());
        fs::copy(path, target_exe_path).with_err_context("Error copying file")?;
    }
    Ok(())
}
