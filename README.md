# `ucch` - Ugly Cropping Considered Harmful

`ucch` is a very simple tool using the MagickWand library of ImageMagick that
transforms images into a form that can be used as a Slack emoji.

It supports all the image formats that ImageMagick does. Every non-GIF image is
converted to PNG format if no background erasure will be done to them after
loading, but the output file format is ultimately decided by the file name
extension of the output file.

## Usage

```
$ ucch \
    [--interactive] \
    [--tolerance PERCENT] \
    [--crop-x X --crop-y Y --crop-size SIZE] \
    <INPUT_PATH> \
    <OUTPUT_PATH>
```

If a tolerance percentage is given by the user, `ucch` attempts to flood-fill
the background of the image with a transparent color, starting from the top-left
pixel, with the tolerance percentage set by the user. Otherwise, the background
of the image will be kept intact.

If any of the cropping-related parameters are specified, all of them need to be
specified. These flags enable the user to perform a custom (but always square)
cropping of the image.

In interactive mode, cropping parameters and the background erasure tolerance
are set from a GUI, which provides a live preview for the cropping.

## License

This software is under WTFPL. For more information, see the `LICENSE` file.
