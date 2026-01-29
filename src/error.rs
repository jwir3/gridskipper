use std::error::Error;
use std::fmt;

/// Custom error type for GridSkipper operations
#[derive(Debug)]
pub enum GridSkipperError {
    /// Error when file does not exist
    FileNotFound(String),
    /// Error when both file paths are identical
    IdenticalPaths,
    /// Error during image processing
    ImageProcessing(String),
    /// IO error
    Io(std::io::Error),
    /// Image error
    Image(image::ImageError),
    /// An error found during overlay compositing
    OverlayCompositingError(String),
}

impl fmt::Display for GridSkipperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridSkipperError::FileNotFound(path) => write!(f, "File not found: {}", path),
            GridSkipperError::IdenticalPaths => write!(f, "Both file paths cannot be the same"),
            GridSkipperError::ImageProcessing(msg) => write!(f, "Image processing error: {}", msg),
            GridSkipperError::Io(e) => write!(f, "IO error: {}", e),
            GridSkipperError::Image(e) => write!(f, "Image error: {}", e),
            GridSkipperError::OverlayCompositingError(e) => {
                write!(f, "Error while compositing overlay: {}", e)
            }
        }
    }
}

impl Error for GridSkipperError {}

impl From<std::io::Error> for GridSkipperError {
    fn from(error: std::io::Error) -> Self {
        GridSkipperError::Io(error)
    }
}

impl From<image::ImageError> for GridSkipperError {
    fn from(error: image::ImageError) -> Self {
        GridSkipperError::Image(error)
    }
}

#[cfg(test)]
mod tests {
    use image::ImageError;
    use image::error::LimitError;

    use super::GridSkipperError;
    use std::io::Error;

    #[test]
    pub fn test_gridskipper_error_display() {
        let error1 = GridSkipperError::ImageProcessing("image incorrect".to_string());
        let error1_display = format!("{}", error1);

        let error2 = GridSkipperError::FileNotFound("invalid file".to_string());
        let error2_display = format!("{}", error2);

        let error3 = GridSkipperError::IdenticalPaths;
        let error3_display = format!("{}", error3);

        let original_error4 = Error::new(std::io::ErrorKind::NotFound, "something happened");
        let error4 = GridSkipperError::Io(original_error4);
        let error4_display = format!("{}", error4);

        let original_error5 = ImageError::Limits(LimitError::from_kind(
            image::error::LimitErrorKind::InsufficientMemory,
        ));
        let error5 = GridSkipperError::from(original_error5);
        let error5_display = format!("{}", error5);

        let error6 =
            GridSkipperError::OverlayCompositingError("image sizes do not match".to_string());
        let error6_display = format!("{}", error6);

        assert_eq!(error1_display, "Image processing error: image incorrect");
        assert_eq!(error2_display, "File not found: invalid file");
        assert_eq!(error3_display, "Both file paths cannot be the same");
        assert_eq!(error4_display, "IO error: something happened");
        assert_eq!(error5_display, "Image error: Memory limit exceeded");
        assert_eq!(
            error6_display,
            "Error while compositing overlay: image sizes do not match"
        );
    }
}
