use thiserror::Error;

/// Custom error type for GridSkipper operations
#[derive(Error, Debug)]
pub enum GridSkipperError {
    /// Error when the byte alignment is not appropriate for u32 integers.
    #[error("Image byte buffer was not properly aligned for u32")]
    ByteAlignmentIncorrect,

    /// Error when byte arrays (for fast checking) are incorrect sizes.
    #[error("Computed byte array is an incorrect size")]
    ByteArrayIncorrectSize,

    /// Error when file does not exist
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Error when both file paths are identical
    #[error("Both file paths cannot be the same")]
    IdenticalPaths,
    
    /// Image error
    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),

    /// An error caused by an image being in an invalid format
    #[error("Image in invalid format: {0}")]
    InvalidImageFormat(String),

    /// Error during image processing
    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("Images have different heights: {0} vs {1}")]
    ImageSizesDifferentHeight(u32, u32),

    #[error("Images have different widths: {0} vs {1}")]
    ImageSizesDifferentWidth(u32, u32),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// An error found during overlay compositing
    #[error("Error while compositing overlay: {0}")]
    OverlayCompositingError(String),
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
