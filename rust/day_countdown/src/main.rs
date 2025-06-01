use chrono::{Local, NaiveDate};
use std::env;
use std::fs::OpenOptions;
use std::io::Read;
use errors_with_context::{ErrorMessage, WithContext};

fn main() -> Result<(), ErrorMessage> {
    let exe_path = env::current_exe()
        .with_err_context("Could not get path to current exe")?;
    let exe_dir_path =
        exe_path.parent()
            .with_err_context("Could not get parent dir of this exe")?;
    let target_date_path = exe_dir_path.join("day_countdown_target_date");

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
