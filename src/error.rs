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
}

impl fmt::Display for GridSkipperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridSkipperError::FileNotFound(path) => write!(f, "File not found: {}", path),
            GridSkipperError::IdenticalPaths => write!(f, "Both file paths cannot be the same"),
            GridSkipperError::ImageProcessing(msg) => write!(f, "Image processing error: {}", msg),
            GridSkipperError::Io(e) => write!(f, "IO error: {}", e),
            GridSkipperError::Image(e) => write!(f, "Image error: {}", e),
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
