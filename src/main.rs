use crate::result::{print_error, Result};
use clap::Parser;
use magick_rust::{magick_wand_genesis, magick_wand_terminus, MagickWand};

mod operations;
mod result;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct CommandLineArguments {
    /// Path to the input file
    #[arg()]
    input: String,

    /// Path to the output file
    #[arg()]
    output: String,

    /// Tolerance percent for flood-fill based background erasure
    #[arg(long, default_value_t = 0f64)]
    tolerance: f64,
}

fn main() -> Result<()> {
    let arguments = CommandLineArguments::parse();

    magick_wand_genesis();

    let result: Result<()> = {
        let blob = std::fs::read(&arguments.input)?;
        let mut wand = MagickWand::new();
        wand.read_image_blob(blob)?;
        println!("(input) Original image read from '{}'.", &arguments.input);
        operations::convert(&mut wand)?;
        operations::squarify(&mut wand)?;
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
