#[derive(Debug)]
pub(crate) enum UcchError {
    FromStdIo(std::io::Error),
    FromStdStrUtf8(std::str::Utf8Error),
    FromMagick(magick_rust::MagickError),
    InvalidArgument(String),
}

impl From<std::io::Error> for UcchError {
    fn from(error: std::io::Error) -> UcchError {
        UcchError::FromStdIo(error)
    }
}

impl From<std::str::Utf8Error> for UcchError {
    fn from(error: std::str::Utf8Error) -> UcchError {
        UcchError::FromStdStrUtf8(error)
    }
}

impl From<magick_rust::MagickError> for UcchError {
    fn from(error: magick_rust::MagickError) -> UcchError {
        UcchError::FromMagick(error)
    }
}

pub(crate) type Result<T> = std::result::Result<T, UcchError>;

pub(crate) fn print_error(error: &UcchError) {
    match error {
        UcchError::FromStdIo(e) => println!("{}", e),
        UcchError::FromStdStrUtf8(e) => println!("{}", e),
        UcchError::FromMagick(e) => println!("{}", e),
        UcchError::InvalidArgument(e) => println!("{}", &e),
    };
}
