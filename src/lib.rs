//! # GridSkipper
//!
//! A library for comparing images and detecting antialiased pixels within an image.
//!
//! This library provides functionality to:
//! - Compare two images for differences
//! - Detect antialiased pixels within an image
//!
//! ```
pub mod base;
pub mod error;
pub mod results;
pub mod stats;
pub mod ui;
pub mod utils;

use crate::base::{ImageLocation, PixelNeighbors};
use crate::error::GridSkipperError;
use crate::results::{AntialiasDetectionResult, DiffResult};
use crate::stats::RawAAStatistics;
use crate::utils::{compute_brightness_delta_grayscale, has_at_least_three_equal_adjacent_pixels};
use image::{DynamicImage, GrayImage};
use std::path::Path;

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

/// Detect antialiased pixels using Vyšniauskas' algorithm (2009).
///
/// The original algorithm is described in the paper:
///   - Antialiased pixel and intensity slope detector, V. Vyšniauskas (2009).
///
/// This implementation takes a full color image, encoded as 32-bit (8 bit color channels) RGBA, and converts it to
/// grayscale using the `image.grayscale()` method, which is equivalent to `Luma` images. It then applies the algorithm
/// described in Vyšniauskas' paper and first implemented in `pixelmatch`, to detect antialiased pixels. These pixels
/// are then returned as an `AntialiasDetectionResult`.
///
/// Parameters:
/// - `image`: The input image to detect antialiased pixels in.
/// - `ground_truth_image_opt`: An optional ground truth image to compare against, for inclusion of statistics, such as
///    the number of false-positives and false-negatives.
///
/// Returns:
/// - `Result<AntialiasDetectionResult, GridSkipperError>`: The result of the antialias detection.
pub fn detect_antialiased_pixels_vysniauskas_rgba8(
    image: DynamicImage,
    ground_truth_image_opt: Option<DynamicImage>,
) -> Result<AntialiasDetectionResult, GridSkipperError> {
    // Convert to RGBA first to ensure 4 bytes per pixel
    let rgba_image = image.to_rgba8();
    let bytes = rgba_image.as_raw();

    // Convert the image to grayscale so that we can compare brightness values.
    let grayscale_image: GrayImage = image.grayscale().into();

    if ground_truth_image_opt.is_some() {}
    // Convert ground truth image once outside the loop
    let ground_truth_bytes = ground_truth_image_opt
        .as_ref()
        .map(|img| img.to_rgba8().into_raw());

    let mut statistics = RawAAStatistics::default();
    let mut result = AntialiasDetectionResult::default();

    for y in 0..image.height() {
        'center_pixel: for x in 0..image.width() {
            let pixel_idx: usize = ((image.width() * y + x) << 2) as usize;
            let mut considered_pixel_is_aa_in_gt = false;

            result.number_pixels += 1;

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
                        statistics.num_ground_truth_antialiased_pixels += 1;
                        considered_pixel_is_aa_in_gt = true;
                    }
                } else {
                    return Err(GridSkipperError::ImageProcessing(
                        "Failed to extract pixel data for ground truth image".to_string(),
                    ));
                }
            }

            let neighbors = get_neighbors_respect_image_bounds(
                x as i32,
                y as i32,
                image.width(),
                image.height(),
            );

            // TODO_jwir3: This is questionable. What if we are the top left corner or bottom right corner? Should we
            //             still consider the pixel as having only a single zero since we can't evaluate its neighbors?
            let mut zeroes = if x == neighbors.left
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
            let mut min_x = 0;
            let mut max_x = 0;
            let mut min_y = 0;
            let mut max_y = 0;

            // We now iterate through 8 adjacent pixels, looking for differences in brightness.
            for nx in neighbors.left..neighbors.right + 1 {
                for ny in neighbors.top..neighbors.bottom + 1 {
                    if nx == x && ny == y {
                        continue;
                    }

                    // Calculate the brightness difference between the current pixel and center pixel.
                    // This assumes that the image is grayscale and that the pixel data is originally in RGBA format.
                    let delta = compute_brightness_delta_grayscale(&grayscale_image, x, y, nx, ny);

                    if delta == 0 {
                        zeroes += 1;
                        if zeroes > 2 {
                            // This is not an antialiased pixel by the Vyšniauskas method.
                            if considered_pixel_is_aa_in_gt {
                                // We have a false negative because the pixel is actually antialiased in the
                                // ground truth image.
                                statistics.num_false_negatives += 1;
                            }

                            continue 'center_pixel;
                        }
                    } else if delta < min {
                        // We want to remember the darkest pixel.
                        // XXX_jwir3: What happens if we encounter a pixel that is _tied_ for the brightest pixel
                        // or darkest pixel?
                        min = delta;
                        min_x = nx;
                        min_y = ny;
                    } else if delta > max {
                        // We also want to remember the brightest pixel.
                        // XXX_jwir3: What happens if we encounter a pixel that is _tied_ for the brightest pixel
                        // or darkest pixel?
                        max = delta;
                        max_x = nx;
                        max_y = ny;
                    }
                }
            }

            // A pixel cannot be anti-aliasing via this method if there are not adjacent pixels that
            // are both darker and brighter than this center pixel.
            if min == 0 || max == 0 {
                continue 'center_pixel;
            }

            // Finally, we want to check if the darkest or brightest pixel has 3+ equal siblings. If it does,
            // then we know that these are at the top (brightest) or bottom (darkest) of the "brightness
            // landscape", according to Vyšniauskas. (Though, it's not clear if this is meant to be the global or
            // local brightness landscape).
            if has_at_least_three_equal_adjacent_pixels(&grayscale_image, min_x, min_y)
                || has_at_least_three_equal_adjacent_pixels(&grayscale_image, max_x, max_y)
            {
                result.antialiased_pixels.push(ImageLocation { x: x, y: y });
                result.num_antialiased_pixels += 1;

                if considered_pixel_is_aa_in_gt {
                    statistics.num_true_positives += 1;
                } else {
                    // We have a false positive because the pixel is NOT antialiased in the ground truth image.
                    statistics.num_false_positives += 1;
                }
            }
        }
    }

    if let Some(ref _gt_bytes) = ground_truth_bytes {
        result.statistics = Some(statistics);
    } else {
        result.statistics = None
    }

    Ok(result)
}

/// Retrieve the indices of neighboring pixels in an image, given a center pixel. Respects image boundaries so that
/// the indices are within the image dimensions.
///
/// # Arguments
///
/// Returns:
/// - `PixelNeighbors`: A structure containing the indices of the neighboring pixels, respecting image boundaries.
pub fn get_neighbors_respect_image_bounds(
    x: i32,
    y: i32,
    width: u32,
    height: u32,
) -> PixelNeighbors {
    let x0 = std::cmp::max(x - 1, 0);
    let y0 = std::cmp::max(y - 1, 0);
    let x2 = std::cmp::min(x + 1, width as i32 - 1);
    let y2 = std::cmp::min(y + 1, height as i32 - 1);

    PixelNeighbors {
        left: x0 as u32,
        right: x2 as u32,
        top: y0 as u32,
        bottom: y2 as u32,
    }
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
        let stats = unwrapped_result.statistics;
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().num_ground_truth_antialiased_pixels, 793);
    }
}
