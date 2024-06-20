use crate::result::{Result, UcchError};
use magick_rust::bindings::MagickFloodfillPaintImage;
use magick_rust::{FilterType, MagickError, MagickWand, PixelWand};

fn sizeof(wand: &MagickWand) -> Result<usize> {
    let blob = wand.write_images_blob(&wand.get_image_format()?)?;
    Ok(blob.len())
}

pub(crate) fn convert(wand: &mut MagickWand, tolerance: f64) -> Result<()> {
    let format = wand.get_image_format()?;
    if format != "GIF" && tolerance == 0f64 {
        println!(
            "(convert) Image format is not GIF and no background erasure will \
            be done, skipping conversion."
        );
    } else if format != "GIF" {
        println!("(convert) Image format is not GIF, converting to PNG.");
        wand.set_image_format("PNG")?;
    } else {
        println!("(convert) Image format is GIF, skipping conversion.");
    }
    Ok(())
}

pub(crate) fn squarify(
    wand: &mut MagickWand,
    x: Option<isize>,
    y: Option<isize>,
    size: Option<isize>,
) -> Result<()> {
    let any = x.is_some() || y.is_some() || size.is_some();
    let all = x.is_some() && y.is_some() && size.is_some();
    if any && !all {
        let message = String::from(
            "If either the X or Y coordinate is or the size specified for the \
            crop operation, all of them need to be present.",
        );
        return Err(UcchError::InvalidArgument(message));
    }

    let width = wand.get_image_width() as isize;
    let height = wand.get_image_height() as isize;
    if width == height && !any {
        println!("(squarify) Image is a square, skipping squarification.");
    } else {
        let smaller = std::cmp::min(width, height);
        let size = if let Some(size) = size { size } else { smaller };

        let x_offset = if let Some(x) = x {
            x
        } else {
            (width - size) / 2
        };

        let y_offset = if let Some(y) = y {
            y
        } else {
            (height - size) / 2
        };

        let x_big = size > (width - x_offset);
        let y_big = size > (height - y_offset);
        if x_big || y_big {
            let message = String::from(
                "Invalid crop size, cropping would go outside the boundary of \
                the image.",
            );
            return Err(UcchError::InvalidArgument(message));
        }

        let repage = format!("{}x{}+{}+{}", size, size, 0, 0);

        println!(
            "(squarify) Squarifying image to {}x{} resolution.",
            size, size
        );

        wand.set_first_iterator();
        wand.crop_image(size as usize, size as usize, x_offset, y_offset)?;
        wand.reset_image_page(&repage)?;
        while wand.next_image() {
            wand.crop_image(size as usize, size as usize, x_offset, y_offset)?;
            wand.reset_image_page(&repage)?;
        }
    }

    Ok(())
}

pub(crate) fn filter_frame(wand: &mut MagickWand, fuzz: f64) -> Result<()> {
    let mut pixel = PixelWand::new();
    pixel.set_color("rgba(255, 0, 0,0)")?;
    pixel.set_fuzz(fuzz);

    let status = unsafe {
        MagickFloodfillPaintImage(
            wand.wand,
            pixel.wand,
            fuzz,
            std::ptr::null(),
            0isize,
            0isize,
            0,
        )
    };

    if status != 0 {
        Ok(())
    } else {
        let message = String::from("Failed to flood paint image.");
        Err(UcchError::FromMagick(MagickError(message)))
    }
}

pub(crate) fn filter(wand: &mut MagickWand, tolerance: f64) -> Result<()> {
    if tolerance == 0f64 {
        println!("(filter) Zero or no tolerance specified, skipping background removal.");
        return Ok(());
    }

    println!("(filter) Attempting to flood-fill and remove background.");

    // Fuzz is computed as the root mean squared difference between two colors.
    // As such, fuzz in percent is 100 * fuzz / UINT16_MAX, and one percent
    // of tolerance is 0.01 * UINT16_MAX.
    let fuzz = tolerance * (0.01 * 65535f64);
    wand.set_first_iterator();
    filter_frame(wand, fuzz)?;
    while wand.next_image() {
        filter_frame(wand, tolerance)?;
    }

    Ok(())
}

pub(crate) fn downscale(wand: &mut MagickWand) -> Result<()> {
    let mut size = sizeof(wand)?;
    if size <= 128_000 {
        println!("(downscale) Image is under 128k, skipping downscaling.");
        return Ok(());
    }

    println!("(downscale) Downscaling image until it is under 128k.");

    while size > 128_000 {
        let scale = match size {
            1..=255_999 => 0.95,
            256_000..=999_999 => 0.90,
            _ => 0.50,
        };

        wand.set_first_iterator();
        wand.scale_image(scale, scale, FilterType::Kaiser)?;
        while wand.next_image() {
            wand.scale_image(scale, scale, FilterType::Kaiser)?;
        }
        size = sizeof(wand)?;
        println!(
            "(downscale) Performed a {}% downscale, size is {}k.",
            (100.0 - scale * 100.0) as u32,
            size / 1000
        );
    }

    println!(
        "(downscale) Final image size is {}x{}.",
        wand.get_image_width(),
        wand.get_image_height()
    );

    Ok(())
}
