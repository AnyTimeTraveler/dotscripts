use chrono::{Local, NaiveDate};
use std::env;
use std::error::Error;
use std::fs::OpenOptions;
use std::io::Read;

mod helpers;

fn main() -> Result<(), Box<dyn Error>> {
    let exe_path = env::current_exe()?;
    let exe_dir_path = exe_path
        .parent()
        .ok_or("Could not get parent dir of this exe".to_owned())?;
    let target_date_path = exe_dir_path.join("day_countdown_target_date");

    let mut target_date = String::new();
    OpenOptions::new()
        .read(true)
        .open(&target_date_path)
        .map_err(|error| {
            format!(
                "{:?}\nLooked for target date at: {:?}",
                error, target_date_path
            )
        })?
        .read_to_string(&mut target_date)?;

    let target_date = NaiveDate::parse_from_str(target_date.trim(), "%d.%m.%Y")
        .map_err(|error| format!("{:?}\nExpected format: DD.MM.YYYY", error))?;
    let current_date = Local::now().naive_local().date();
    let difference = target_date - current_date;

    match difference.num_days() {
        number if number <= 0 => println!("In my arms! 🥰️"),
        1 => println!("Tomorrow! ♥️"),
        number => println!("{} Days ♥️", number),
    }

    Ok(())
}
