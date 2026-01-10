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

use image::buffer::ConvertBuffer;
use image::{self, ColorType, ConvertColorOptions, DynamicImage, GrayImage, ImageError, RgbaImage};
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

#[derive(Debug)]
pub struct ImageLocation {
    pub x: u32,
    pub y: u32,
}

/// Result of antialiasing detection
#[derive(Debug)]
pub struct AntialiasDetectionResult {
    /// Number of pixels in the image without antialiasing that are different in the image with antialiasing
    pub num_ground_truth_antialiased_pixels: u64,

    /// Number of antialiased pixels detected
    pub num_antialiased_pixels: u64,

    /// Number of true positives
    pub num_true_positives: u64,

    /// Number of false positives. This is the number of pixels that were identified as antialiased but are not antialiased
    /// in the ground truth image
    pub num_false_positives: u64,

    /// Number of false negatives. This is the number of pixels that are antialiased in the ground truth image but were not
    /// identified as antialiased
    pub num_false_negatives: u64,

    /// A list of antialiased pixel locations
    pub antialiased_pixels: Vec<ImageLocation>,
}

impl Default for AntialiasDetectionResult {
    fn default() -> Self {
        Self {
            num_ground_truth_antialiased_pixels: 0,
            num_antialiased_pixels: 0,
            num_true_positives: 0,
            num_false_positives: 0,
            num_false_negatives: 0,
            antialiased_pixels: Vec::new(),
        }
    }
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

pub fn detect_antialiased_pixels_vysniauskas_rgba8(
    image: DynamicImage,
    ground_truth_image_opt: Option<DynamicImage>,
) -> Result<AntialiasDetectionResult, GridSkipperError> {
    // Convert to RGBA first to ensure 4 bytes per pixel
    let rgba_image = image.to_rgba8();
    let bytes = rgba_image.as_raw();

    // Convert ground truth image once outside the loop
    let ground_truth_bytes = ground_truth_image_opt
        .as_ref()
        .map(|img| img.to_rgba8().into_raw());

    let mut result = AntialiasDetectionResult::default();

    for y in 0..image.height() {
        for x in 0..image.width() {
            let pixel_idx: usize = ((image.width() * y + x) << 2) as usize;
            let mut considered_pixel_is_aa_in_gt = false;

            // if we have a ground truth image, and the pixels are different
            // at this location between the ground truth and the image under consideration
            // then we have an AA pixel in the image under consideration
            if let Some(ref gt_bytes) = ground_truth_bytes {
                let ground_truth_pixel_res: Result<[u8; 4], _> =
                    gt_bytes[pixel_idx..pixel_idx + 4].try_into();
                if let Ok(ground_truth_pixel) = ground_truth_pixel_res {
                    let pixel: [u8; 4] = bytes[pixel_idx..pixel_idx + 4].try_into().unwrap();
                    if ground_truth_pixel != pixel {
                        // This is an AA pixel in the image under consideration
                        result.num_ground_truth_antialiased_pixels += 1;
                        considered_pixel_is_aa_in_gt = true;
                    }
                } else {
                    return Err(GridSkipperError::ImageProcessing(
                        "Failed to extract pixel data for ground truth image".to_string(),
                    ));
                }
            }

            let neighbors = get_neighbors_respect_image_bounds(x as i32, y as i32, &image);

            // TODO_jwir3: This is questionable. What if we are the top left corner or bottom right corner? Should we
            //             still consider the pixel as having only a single zero since we can't evaluate its neighbors?
            let zeroes = if x == neighbors.left
                || x == neighbors.right
                || y == neighbors.top
                || y == neighbors.bottom
            {
                1
            } else {
                0
            };

            let mut min = 0;
            let mut max = 0;
            let mut minX = 0;
            let mut maxX = 0;
            let mut minY = 0;
            let mut maxY = 0;

            // We now iterate through 8 adjacent pixels, looking for differences in brightness.
            for nx in neighbors.left..neighbors.right + 1 {
                for ny in neighbors.top..neighbors.bottom + 1 {
                    if nx == x && ny == y {
                        continue;
                    }

                    // Calculate the brightness difference between the current pixel and center pixel.
                    // This assumes that the image is grayscale and that the pixel data is originally in RGBA format.
                    // const delta = compute_brightness_delta_grayscale(image, pos * 4, (ny * width + nx) * 4, true);
                }
            }
        }
    }

    Ok(result)
}

/// A structure indicating the indices of neighboring pixels in an image.
pub struct PixelNeighbors {
    left: u32,
    right: u32,
    top: u32,
    bottom: u32,
}

/// Retrieve the indices of neighboring pixels in an image, given a center pixel. Respects image boundaries so that
/// the indices are within the image dimensions.
///
/// Parameters:
/// - `x`: The x-coordinate of the center pixel.
/// - `y`: The y-coordinate of the center pixel.
/// - `image`: The image to retrieve the neighbors from.
///
/// Returns:
/// - `PixelNeighbors`: A structure containing the indices of the neighboring pixels, respecting image boundaries.
pub fn get_neighbors_respect_image_bounds(x: i32, y: i32, image: &DynamicImage) -> PixelNeighbors {
    let x0 = std::cmp::max(x - 1, 0);
    let y0 = std::cmp::max(y - 1, 0);
    let x2 = std::cmp::min(x + 1, image.width() as i32 - 1);
    let y2 = std::cmp::min(y + 1, image.height() as i32 - 1);

    PixelNeighbors {
        left: x0 as u32,
        right: x2 as u32,
        top: y0 as u32,
        bottom: y2 as u32,
    }
}

pub fn copy_image_to_grayscale(image: &RgbaImage) -> GrayImage {
    // Note this isn't _quite_ correct. The color transform in the image crate is not perfect, and the definition of
    // this function gives a warning that this will be disabled in future releases. We should update this to fix the
    // problem in the future.
    let grayscale_image: GrayImage = image.convert();
    grayscale_image
}

/// Compute the brightness delta between two pixels in a grayscale image and encode whether the difference is brighter
/// or darker based on the sign.
///
/// Parameters:
/// - `grayscale_image`: The grayscale image.
/// - `center_x`: The x-coordinate of the center pixel.
/// - `center_y`: The y-coordinate of the center pixel.
/// - `adjacent_x`: The x-coordinate of the adjacent pixel.
/// - `adjacent_y`: The y-coordinate of the adjacent pixel.
///
/// Returns:
/// - The brightness delta between the two pixels, with the adjacent pixel being brighter encoded as a positive value
/// and the adjacent pixel being darker encoded as a negative value.
pub fn compute_brightness_delta_grayscale(
    grayscale_image: &GrayImage,
    center_x: u32,
    center_y: u32,
    adjacent_x: u32,
    adjacent_y: u32,
) -> i16 {
    let center_pixel = grayscale_image.get_pixel(center_x, center_y);
    let adjacent_pixel = grayscale_image.get_pixel(adjacent_x, adjacent_y);
    let delta = adjacent_pixel[0] as i16 - center_pixel[0] as i16;
    delta
}

#[cfg(test)]
mod tests {
    use image::Luma;

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

    #[test]
    fn test_ground_truth_detection() {
        // Open the files
        let ground_truth_image =
            image::open("data/images/no-aa/lowercase-b-noto-serif-noAa.png").unwrap();
        let image_under_consideration =
            image::open("data/images/analytic-aa/lowercase-b-noto-serif-aa.png").unwrap();

        // Detect ground truth antialiasing
        let result = detect_antialiased_pixels_vysniauskas_rgba8(
            ground_truth_image,
            Some(image_under_consideration),
        );
        assert!(matches!(result, Ok(_)));

        let unwrapped_result = result.unwrap();
        assert_eq!(unwrapped_result.num_ground_truth_antialiased_pixels, 793);
    }

    #[test]
    fn test_compute_brightness_delta_grayscale() {
        // NOTE_jwir3: This example is taken from pg 109 of Vyšniauskas' paper. The image is a 3x3 grid of pixels. Note
        // that the delta values should be a subtraction of the adjacent pixel from the current pixel luminance value.
        // Further, the example is taken from the upper left corner of the first image on that page. In the image, a
        // simple subtraction error seems to be present in most of the examples. This test corrects that error (namely
        // in the subtraction of the lower right pixel from the example image).
        let mut grayscale_image = GrayImage::new(3, 3);
        grayscale_image.put_pixel(0, 0, Luma([50]));
        grayscale_image.put_pixel(1, 0, Luma([60]));
        grayscale_image.put_pixel(2, 0, Luma([222]));
        grayscale_image.put_pixel(0, 1, Luma([50]));
        grayscale_image.put_pixel(1, 1, Luma([128]));
        grayscale_image.put_pixel(2, 1, Luma([232]));
        grayscale_image.put_pixel(0, 2, Luma([50]));
        grayscale_image.put_pixel(1, 2, Luma([141]));
        grayscale_image.put_pixel(2, 2, Luma([232]));

        let brightness_delta1 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 0, 0);
        assert_eq!(brightness_delta1, -78);

        let brightness_delta2 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 1, 0);
        assert_eq!(brightness_delta2, -68);

        let brightness_delta3 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 2, 0);
        assert_eq!(brightness_delta3, 94);

        let brightness_delta4 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 0, 1);
        assert_eq!(brightness_delta4, -78);

        let brightness_delta5 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 1, 1);
        assert_eq!(brightness_delta5, 0);

        let brightness_delta6 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 2, 1);
        assert_eq!(brightness_delta6, 104);

        let brightness_delta7 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 0, 2);
        assert_eq!(brightness_delta7, -78);

        let brightness_delta8 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 1, 2);
        assert_eq!(brightness_delta8, 13);

        let brightness_delta9 = compute_brightness_delta_grayscale(&grayscale_image, 1, 1, 2, 2);
        assert_eq!(brightness_delta9, 104);
    }
}
