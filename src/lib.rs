//! # GridSkipper
//!
//! A library for comparing images and detecting antialiased pixels within an image.
//!
//! This library provides functionality to:
//! - Compare two images for differences
//! - Detect antialiased pixels within an image
//!
//! ## Usage
//!
//! ```rust
//! use gridskipper::{diff_images, detect_antialiased_pixels, GridSkipperError};
//!
//! // Compare two images
//! match diff_images("image1.png", "image2.png") {
//!     Ok(result) => println!("Diff result: {:?}", result),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//!
//! // Detect antialiased pixels
//! match detect_antialiased_pixels("image.png") {
//!     Ok(result) => println!("Detection result: {:?}", result),
//!     Err(e) => eprintln!("Error: {}", e),
//! }
//! ```

use std::error::Error;
use std::fmt;
use std::path::Path;

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
}

impl fmt::Display for GridSkipperError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GridSkipperError::FileNotFound(path) => write!(f, "File not found: {}", path),
            GridSkipperError::IdenticalPaths => write!(f, "Both file paths cannot be the same"),
            GridSkipperError::ImageProcessing(msg) => write!(f, "Image processing error: {}", msg),
            GridSkipperError::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl Error for GridSkipperError {}

impl From<std::io::Error> for GridSkipperError {
    fn from(error: std::io::Error) -> Self {
        GridSkipperError::Io(error)
    }
}

/// Result of comparing two images
#[derive(Debug)]
pub struct DiffResult {
    /// Whether the images are identical
    pub identical: bool,
    /// Number of different pixels found
    pub diff_pixels: u32,
    /// Percentage of different pixels
    pub diff_percentage: f64,
}

/// Result of antialiasing detection
#[derive(Debug)]
pub struct AntialiasDetectionResult {
    /// Number of antialiased pixels detected
    pub antialiased_pixels: u32,
    /// Percentage of antialiased pixels
    pub antialiased_percentage: f64,
}

/// Validates that file paths are different and both files exist
pub fn validate_file_paths(left_side: &str, right_side: &str) -> Result<(), GridSkipperError> {
    // Check if both file paths are the same
    if left_side == right_side {
        return Err(GridSkipperError::IdenticalPaths);
    }

    // Check if files exist
    if !Path::new(left_side).exists() {
        return Err(GridSkipperError::FileNotFound(left_side.to_string()));
    }

    if !Path::new(right_side).exists() {
        return Err(GridSkipperError::FileNotFound(right_side.to_string()));
    }

    Ok(())
}

/// Validates that a single file path exists
pub fn validate_single_file_path(path: &str) -> Result<(), GridSkipperError> {
    if !Path::new(path).exists() {
        return Err(GridSkipperError::FileNotFound(path.to_string()));
    }
    Ok(())
}

/// Compare two images for differences
///
/// # Arguments
///
/// * `left_side` - Path to the first image file
/// * `right_side` - Path to the second image file
///
/// # Returns
///
/// Returns a `DiffResult` containing information about the differences between the images,
/// or a `GridSkipperError` if validation fails or image processing encounters an error.
///
/// # Example
///
/// ```rust
/// use gridskipper::diff_images;
///
/// match diff_images("image1.png", "image2.png") {
///     Ok(result) => {
///         if result.identical {
///             println!("Images are identical");
///         } else {
///             println!("Found {} different pixels ({}%)",
///                     result.diff_pixels, result.diff_percentage);
///         }
///     },
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn diff_images(left_side: &str, right_side: &str) -> Result<DiffResult, GridSkipperError> {
    // Validate input
    validate_file_paths(left_side, right_side)?;

    // TODO: Implement actual image comparison logic
    // This is a placeholder implementation
    Ok(DiffResult {
        identical: false,
        diff_pixels: 0,
        diff_percentage: 0.0,
    })
}

/// Detect antialiased pixels within an image
///
/// # Arguments
///
/// * `path` - Path to the image file to analyze
///
/// # Returns
///
/// Returns an `AntialiasResult` containing information about antialiased pixels found,
/// or a `GridSkipperError` if validation fails or image processing encounters an error.
///
/// # Example
///
/// ```rust
/// use gridskipper::detect_antialiased_pixels;
///
/// match detect_antialiased_pixels("image.png") {
///     Ok(result) => {
///         println!("Found {} antialiased pixels ({}%)",
///                 result.antialiased_pixels, result.antialiased_percentage);
///     },
///     Err(e) => eprintln!("Error: {}", e),
/// }
/// ```
pub fn detect_antialiased_pixels(path: &str) -> Result<AntialiasDetectionResult, GridSkipperError> {
    // Validate input
    validate_single_file_path(path)?;

    // TODO: Implement actual antialiasing detection logic
    // This is a placeholder implementation
    Ok(AntialiasDetectionResult {
        antialiased_pixels: 0,
        antialiased_percentage: 0.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identical_paths_error() {
        let result = validate_file_paths("same.png", "same.png");
        assert!(matches!(result, Err(GridSkipperError::IdenticalPaths)));
    }

    #[test]
    fn test_file_not_found_error() {
        let result = validate_single_file_path("nonexistent.png");
        assert!(matches!(result, Err(GridSkipperError::FileNotFound(_))));
    }
}
