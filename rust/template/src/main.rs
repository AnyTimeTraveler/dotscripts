use env_logger::{Env, Target};
use errors_with_context::ErrorMessage;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    env_logger::builder()
        .parse_env(Env::default().filter_or("RUST_LOG", "info"))
        .target(Target::Stdout)
        .format_timestamp_secs()
        .init();


    Ok(())
}
