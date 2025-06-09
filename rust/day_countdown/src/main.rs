use chrono::{Local, NaiveDate};
use std::env;
use std::env::home_dir;
use std::fs::OpenOptions;
use std::io::Read;
use errors_with_context::{ErrorMessage, WithContext};

fn main() -> Result<(), ErrorMessage> {
    let home = home_dir()
        .with_err_context("Could not get home dir")?;
    let target_date_path = home
        .join(".data")
        .join("day_countdown_target_date");

    let mut target_date = String::new();
    OpenOptions::new()
        .read(true)
        .open(&target_date_path)
        .with_dyn_err_context(|| format!("Could not find target date at: {:?}", target_date_path))?
        .read_to_string(&mut target_date)
        .with_err_context("Could not read target date")?;

    let target_date = NaiveDate::parse_from_str(target_date.trim(), "%d.%m.%Y")
        .with_err_context("Unexpected date format. Expected format: DD.MM.YYYY")?;
    let current_date = Local::now().naive_local().date();
    let difference = target_date - current_date;

    match difference.num_days() {
        number if number <= 0 => println!("In my arms! 🥰️"),
        1 => println!("Tomorrow! ♥️"),
        number => println!("{} Days ♥️", number),
    }

    Ok(())
}
