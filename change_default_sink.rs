use crate::helpers::run;
use std::error::Error;

mod helpers;


#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn Error>> {
    let all_sinks = list_all_sinks().await?;
    let current_sink = get_default_sink().await?;
    let next_sink = find_next_sink(all_sinks, current_sink)?;
    set_default_sink(&next_sink).await?;
    get_default_sink().await?;
    Ok(())
}

async fn list_all_sinks() -> Result<Vec<String>, Box<dyn Error>> {
    let output = run("pactl", ["list", "short", "sinks"]).await?;
    let mut all_sinks = output
        .lines()
        .filter_map(|line| line.split("\t").nth(1))
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    all_sinks.sort();
    println!("SINKS:\n{:#?}", all_sinks);
    Ok(all_sinks)
}

fn find_next_sink(all_sinks: Vec<String>, current_sink: String) -> Result<String, Box<dyn Error>> {
    let mut sink_iter = all_sinks.iter();
    sink_iter.find(|sink| **sink == current_sink);
    let next_sink = if let Some(sink) = sink_iter.next() {
        sink
    } else {
        all_sinks.first().ok_or("No sinks found!".to_owned())?
    };

    println!("NEXT: {:?}", next_sink);
    Ok(next_sink.to_owned())
}

async fn get_default_sink() -> Result<String, Box<dyn Error>> {
    let output = run("pactl", ["get-default-sink"]).await?;
    let current_sink = output.trim().to_owned();
    println!("CURRENT SINK: {}", current_sink);
    Ok(current_sink)
}

async fn set_default_sink(next_sink: &str) -> Result<(), Box<dyn Error>> {
    run("pactl", ["set-default-sink", next_sink]).await?;
    Ok(())
}
