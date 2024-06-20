use crate::result::{print_error, Result};
use clap::Parser;
use magick_rust::{magick_wand_genesis, magick_wand_terminus, MagickWand};

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
}

fn main() -> Result<()> {
    let arguments = CommandLineArguments::parse();

    magick_wand_genesis();

    let result: Result<()> = {
        let crop_x = arguments.crop_x;
        let crop_y = arguments.crop_y;
        let crop_size = arguments.crop_size;
        let blob = std::fs::read(&arguments.input)?;
        let mut wand = MagickWand::new();
        wand.read_image_blob(blob)?;
        println!("(input) Original image read from '{}'.", &arguments.input);
        operations::convert(&mut wand, arguments.tolerance)?;
        operations::squarify(&mut wand, crop_x, crop_y, crop_size)?;
        operations::filter(&mut wand, arguments.tolerance)?;
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
