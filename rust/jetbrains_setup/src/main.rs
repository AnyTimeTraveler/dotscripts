use errors_with_context::{BooleanErrors, ErrorMessage, WithContext};
use process_utils::run;
use std::path::{Path, PathBuf};
use env_logger::{Env, Target};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::{debug, info, trace};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
        env_logger::builder()
        .parse_env(Env::default().filter_or("RUST_LOG", "info"))
        .target(Target::Stdout)
        .format_timestamp_secs()
        .init();
    let current_path =
        std::env::current_dir().with_err_context("Current directory somehow not found")?;
    debug!("Current directory: {}", current_path.display());
    let workspace_xml = find_dot_idea_folder(current_path)?.join("workspace.xml");
    debug!("Location of .idea/workspace.xml: {}", workspace_xml.display());
    let mut workspace_xml_content = String::new();

    OpenOptions::new()
        .read(true)
        .open(workspace_xml.as_path())
        .await
        .with_err_context("Could not open .idea/workspace.xml for reading")?
        .read_to_string(&mut workspace_xml_content)
        .await
        .with_err_context("Could not read .idea/workspace.xml")?;

    let old_element_start = workspace_xml_content.find(r#"<component name="RustProjectSettings">"#);

    match old_element_start {
        Some(start) => replace_config(&mut workspace_xml_content, start).await?,
        None => insert_new_config(&mut workspace_xml_content).await?,
    }

    OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(workspace_xml.as_path())
        .await
        .with_dyn_err_context(|| format!("Could not open {} for writing", workspace_xml.display()))?
        .write_all(workspace_xml_content.as_bytes())
        .await
        .with_dyn_err_context(|| format!("Could not write {}", workspace_xml.display()))?;

    Ok(())
}

async fn insert_new_config(content: &mut String) -> Result<(), ErrorMessage> {
    let end = content.find("</project>").unwrap();

    let range = end..end;
    let string = create_config().await?;
    content.replace_range(range, &format!("  {string}\n"));
    Ok(())
}

async fn replace_config(content: &mut String, start: usize) -> Result<(), ErrorMessage> {
    debug!("Replacing old config with new values");
    let component_end = "</component>";
    let end = content[start..].find(component_end).unwrap();

    let range = start..start + end + component_end.len();
    trace!("Replacing:\n{}", &content[range.clone()]);
    let new_config = create_config().await?;
    trace!("With:\n{}", new_config);
    content.replace_range(range, &new_config);
    info!("Updated jetbrains configuration to use the currently enabled rust-toolchain!");
    Ok(())
}

async fn create_config() -> Result<String, ErrorMessage> {
    let rust_root = run("rustc", &["--print", "sysroot"])
        .await
        .with_err_context("Executing 'rustc --print sysroot' failed")?;
    let rust_root = rust_root.trim();
    info!("Found rust at path: {}", &rust_root);
    let rust_bin = format!("{rust_root}/bin");
    Path::new(&rust_bin).exists().error_dyn_if_false(||format!("Rust binaries not found at path: {}", &rust_bin))?;
    let rust_lib = format!("{rust_root}/lib/rustlib/src/rust/library");
    Path::new(&rust_lib).exists().error_dyn_if_false(||format!("Rust stdlib not found at path: {}", &rust_lib))?;
    debug!("Rust bin: {}", rust_bin);
    debug!("Rust lib: {}", rust_lib);
    Ok(format!(
        r#"<component name="RustProjectSettings">
    <option name="toolchainHomeDirectory" value="{rust_bin}" />
    <option name="explicitPathToStdlib" value="{rust_lib}" />
  </component>"#
    ))
}

fn find_dot_idea_folder(mut path: PathBuf) -> Result<PathBuf, ErrorMessage> {
    loop {
        let idea_path = path.join(".idea");
        if idea_path.is_dir() {
            return Ok(idea_path);
        }

        let Some(parent) = path.parent() else {
            return ErrorMessage::err(
                "Could not find .idea/ directory when searching all parents.\
                 Root path reached when trying to get parent directory."
                    .to_owned(),
            );
        };
        path = parent.to_path_buf();
    }
}
