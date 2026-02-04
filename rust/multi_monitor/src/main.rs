//! ################################################
//! # Sway Monitor setup script by AnyTimeTraveler #
//! ################################################

mod output_filter;
mod swaymsg;

use errors_with_context::ErrorMessage;
use output_filter::{any_of, OutputFilter};
use crate::outputs::SwayOutputs;

mod outputs;

const BG_PATH: &'static str = "/home/nora/.config/sway";
const TRANS_CROPPED: &'static str = "trans_cropped.jpg fit";
const TRANS_LEFT: &'static str = "trans_left.jpg fit";
const TRANS_MIDDLE: &'static str = "trans_middle.jpg fit";
const TRANS_RIGHT: &'static str = "trans_right.jpg fit";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), ErrorMessage> {
    let outputs = SwayOutputs::get_outputs().await?;
    // ################################################
    // #         MONITORS and CONFIGURATIONS          #
    // ################################################

    // # Define your monitors
    // # Each monitor must be defined by at least 1 parameter, but more is better to avoid collisions
    println!("Detected Monitors:");
    let laptop_builtin = outputs.find_monitor(
        OutputFilter::new() //
            .name_regex("eDP-1"), //
    );
    println!("laptop_builtin: {}", laptop_builtin.is_some());
    let desk_center = outputs.find_monitor(
        OutputFilter::new() //
            .model_regex("27GL650F") //
            .make_regex("LG Electronics"), //
    );
    println!("desk_center: {}", desk_center.is_some());
    let desk_left = outputs.find_monitor(
        OutputFilter::new() //
            .model_regex("LEN LT2452pwC") //
            .make_regex("Lenovo Group Limited"), //
    );
    println!("desk_left: {}", desk_left.is_some());
    let desk_right = outputs.find_monitor(
        OutputFilter::new() //
            .model_regex("S242HL") //
            .make_regex("Acer Technologies"), //
    );
    println!("desk_right: {}", desk_right.is_some());

    let dlr_left = any_of(vec![
        outputs.find_monitor(
            OutputFilter::new()
                .make_regex("Dell Inc.")
                .model_regex("DELL P2423DE")
                .serial_regex("9D4M1L3"),
        ),
    ]);
    println!("dlr_left: {}", dlr_left.is_some());
    let dlr_right = any_of(vec![
        outputs.find_monitor(
            OutputFilter::new()
                .make_regex("Dell Inc.")
                .model_regex("DELL P2423DE")
                .serial_regex("5RYK1L3"),
        ),
    ]);
    println!("dlr_right: {}", dlr_right.is_some());

    print!("Choosing to use the following setup: ");
    // # Define your setups based on which monitors were found
    if let (Some(builtin), Some(left), Some(center), Some(right)) =
        (laptop_builtin, desk_left, desk_center, desk_right)
    {
        println!("Home desk setup");
        outputs.setup(|config| {
            config.config(builtin).disable();
            config.config(left).bg(TRANS_LEFT);
            config.config(center).x(left.width()).bg(TRANS_MIDDLE);
            config.config(right).x(left.width() + center.width()).bg(TRANS_RIGHT);
        })
        .await?;

    } else if let (Some(builtin), Some(left), Some(right)) = (laptop_builtin, dlr_left, dlr_right) {
        println!("DLR desk setup");
        outputs.setup(|config| {
            config.config(builtin).x(1568).y(1440).bg(TRANS_CROPPED);
            config.config(left).bg(TRANS_LEFT);
            config.config(right).x(left.width()).bg(TRANS_RIGHT);
        })
        .await?;

    } else if let (Some(builtin), Some(center)) = (laptop_builtin, desk_center) {
        println!("Laptop with screen above setup");
        outputs.setup(|config| {
            config.config(builtin).y(center.height()).bg(TRANS_CROPPED);
            config.config(center).bg(TRANS_CROPPED);
        })
        .await?;

    } else if let Some(builtin) = laptop_builtin {
        println!("Fallback laptop setup");
        outputs.setup(|config| {
            config.config(builtin).bg(decide_background(0, outputs.len()));

            let mut x = 0;
            for (i, output) in outputs.iter().enumerate() {
                if output != builtin {
                    config.config(&output).x(x).bg(decide_background(i, outputs.len()));
                    x += output.width();
                }
            }
        })
        .await?;

    } else {
        println!("Fallback setup");
        outputs.setup(|config| {
            let mut x = 0;
            for (i, output) in outputs.iter().enumerate() {
                config.config(&output).x(x).bg(decide_background(i, outputs.len()));
                x += output.width();
            }
        })
        .await?;
    }
    Ok(())
}

fn decide_background(index: usize, length: usize) -> &'static str {
    match length {
        l if l <= 1 => TRANS_CROPPED,
        2 => match index {
            0 => TRANS_LEFT,
            1 => TRANS_RIGHT,
            _ => TRANS_CROPPED,
        },
        3 => match index {
            0 => TRANS_LEFT,
            1 => TRANS_MIDDLE,
            _ => TRANS_RIGHT,
        },
        _ => match index {
            0 => TRANS_LEFT,
            l if l == length - 1 => TRANS_RIGHT,
            _ => TRANS_MIDDLE,
        },
    }
}
