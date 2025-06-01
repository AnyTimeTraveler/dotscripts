use colored::Colorize;
use errors_with_context::{ErrorMessage, WithContext};
use process_utils::run;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let all_sources = list_all_sources().await?;
    let current_source = get_default_source().await?;
    let mut next_source = find_next_source(&all_sources, &current_source)?;
    set_default_source(&next_source).await?;
    let mut new_source = get_default_source().await?;

    // sometimes, setting a new source fails
    // then try the next source in line
    while new_source != next_source {
        println!("{}: {}", "Setting source failed".red(), next_source);
        next_source = find_next_source(&all_sources, &next_source)?;
        set_default_source(&next_source).await?;
        new_source = get_default_source().await?;
    }
    Ok(())
}

async fn list_all_sources() -> Result<Vec<String>, ErrorMessage> {
    let output = run("pactl", ["list", "short", "sources"]).await?;
    let mut all_sources = output
        .lines()
        .filter(|l| !l.contains(".monitor"))
        .filter_map(|line| line.split("\t").nth(1))
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    all_sources.sort();
    println!("Available sources:\n{:#?}", all_sources);
    Ok(all_sources)
}

fn find_next_source(all_sources: &[String], current_source: &str) -> Result<String, ErrorMessage> {
    let mut source_iter = all_sources.iter();
    source_iter.find(|source| *source == current_source);
    let next_source = if let Some(source) = source_iter.next() {
        source
    } else {
        all_sources.first().with_err_context("No sources found!")?
    };

    println!("Next source: {:?}", next_source);
    Ok(next_source.to_owned())
}

async fn get_default_source() -> Result<String, ErrorMessage> {
    let output = run("pactl", ["get-default-source"]).await?;
    let current_source = output.trim().to_owned();
    println!("Current source: {}", current_source);
    Ok(current_source)
}

async fn set_default_source(next_source: &str) -> Result<(), ErrorMessage> {
    run("pactl", ["set-default-source", next_source]).await?;
    Ok(())
}
