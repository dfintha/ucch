use crate::interactive::perform_interactive_setup;
use crate::operations::OperationOptions;
use crate::result::{print_error, Result};
use clap::Parser;
use magick_rust::{magick_wand_genesis, magick_wand_terminus, MagickWand};

mod interactive;
mod operations;
mod result;

#[derive(Debug, Parser)]
#[command(name = "ucch", version, about, long_about = None)]
#[command(about = "Image to Slack emoji converter")]
#[command(
    after_help = "All crop settings shall be given at once, or the squarify operation will fail."
)]
struct CommandLineArguments {
    /// Path of the input file
    #[arg()]
    input: String,

    /// Path of the output file
    #[arg()]
    output: String,

    /// Tolerance percent for flood-fill based background erasure
    #[arg(long, default_value_t = 0f64, value_name = "PERCENT")]
    tolerance: f64,

    /// Starting X coordinate for cropping
    #[arg(long, value_name = "X")]
    crop_x: Option<isize>,

    /// Starting Y coordinate for cropping
    #[arg(long, value_name = "Y")]
    crop_y: Option<isize>,

    /// Size of the cropped area
    #[arg(long, value_name = "SIZE")]
    crop_size: Option<isize>,

    /// When set, cropping and background erasure is performed through a GUI
    #[arg(long, default_value_t = false)]
    interactive: bool,
}

fn main() -> Result<()> {
    let arguments = CommandLineArguments::parse();

    magick_wand_genesis();

    let options = if arguments.interactive {
        perform_interactive_setup(&arguments.input)?
    } else {
        OperationOptions {
            tolerance: arguments.tolerance,
            crop_x: arguments.crop_x,
            crop_y: arguments.crop_y,
            crop_size: arguments.crop_size,
        }
    };

    let result: Result<()> = {
        let mut wand = MagickWand::new();
        let blob = std::fs::read(&arguments.input)?;
        wand.read_image_blob(blob)?;
        println!("(input) Original image read from '{}'.", &arguments.input);
        operations::convert(&mut wand, &options)?;
        operations::squarify(&mut wand, &options)?;
        operations::filter(&mut wand, &options)?;
        operations::downscale(&mut wand)?;
        wand.write_images(&arguments.output, true)?;
        println!("(output) Final image written to '{}'.", &arguments.output);
        Ok(())
    };

    magick_wand_terminus();

    if let Err(error) = &result {
        print_error(error);
    }
    Ok(())
}
