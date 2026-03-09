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
pub mod color;
pub mod error;
pub mod overlay;
pub mod results;
pub mod stats;
pub mod ui;
pub mod utils;

use crate::base::{ImageLocation, PixelNeighbors};
use crate::error::GridSkipperError;
use crate::results::{AntialiasDetectionResult, DiffResult};
use crate::stats::RawAAStatistics;
use crate::utils::has_at_least_three_equal_adjacent_pixels;
use image::DynamicImage;
use std::path::Path;

use self::utils::{compute_brightness_delta_alpha, compute_brightness_delta_rgba};

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
    Ok(DiffResult::default())
}

/// Detect antialiased pixels using Vyšniauskas' algorithm (2009) using a naive conversion from RGB to grayscale.
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
/// - `image`: The input image to detect antialiased pixels in. Should be in RGBA format.
/// - `ground_truth_image_opt`: An optional ground truth image to compare against, for inclusion of statistics, such as
///   the number of false-positives and false-negatives.
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
    // let grayscale_image: GrayImage = image.grayscale().into();

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
                    // This assumes that the image is in RGBA format.
                    let delta = compute_brightness_delta_rgba(&rgba_image, x, y, nx, ny);

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
            if has_at_least_three_equal_adjacent_pixels(&rgba_image, min_x, min_y)
                || has_at_least_three_equal_adjacent_pixels(&rgba_image, max_x, max_y)
            {
                result.antialiased_pixels.push(ImageLocation { x, y });
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

/// Detect antialiased pixels using Vyšniauskas' algorithm (2009) using alpha values of pixels as intensity.
///
/// The original algorithm is described in the paper:
///   - Antialiased pixel and intensity slope detector, V. Vyšniauskas (2009).
///
/// This implementation takes a full color image, encoded as 32-bit (8 bit color channels) RGBA, and directly uses the
/// alpha channel as intensity values. It then applies the algorithm described in Vyšniauskas' paper and first
/// implemented in `pixelmatch`, to detect antialiased pixels. These pixels are then returned as an `AntialiasDetectionResult`.
///
/// Parameters:
/// - `image`: The input image to detect antialiased pixels in. Should be in RGBA format.
/// - `ground_truth_image_opt`: An optional ground truth image to compare against, for inclusion of statistics, such as
///   the number of false-positives and false-negatives.
///
/// Returns:
/// - `Result<AntialiasDetectionResult, GridSkipperError>`: The result of the antialias detection.
pub fn detect_antialiased_pixels_vysniauskas_alpha(
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
                    // This assumes that the image is in RGBA format.
                    let delta = compute_brightness_delta_alpha(&rgba_image, x, y, nx, ny);

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
            if has_at_least_three_equal_adjacent_pixels(&rgba_image, min_x, min_y)
                || has_at_least_three_equal_adjacent_pixels(&rgba_image, max_x, max_y)
            {
                result.antialiased_pixels.push(ImageLocation { x, y });
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

/// Detect antialiased pixels using Vyšniauskas' algorithm (2009) by using CIELAB "lightness" values for intensity.
///
/// The original algorithm is described in the paper:
///   - Antialiased pixel and intensity slope detector, V. Vyšniauskas (2009).
///
/// This implementation takes a full color image, encoded as 32-bit (8 bit color channels) RGBA, and converts it to
/// CIELAB using the `CieLabPixel::from()` method. It then applies the algorithm described in Vyšniauskas' paper and
/// first implemented in `pixelmatch`, to detect antialiased pixels. These pixels are then returned as an
/// `AntialiasDetectionResult`.
///
/// Parameters:
/// - `image`: The input image to detect antialiased pixels in. Should be in RGBA format.
/// - `ground_truth_image_opt`: An optional ground truth image to compare against, for inclusion of statistics, such as
///   the number of false-positives and false-negatives.
///
/// Returns:
/// - `Result<AntialiasDetectionResult, GridSkipperError>`: The result of the antialias detection.
pub fn detect_antialiased_pixels_vysniauskas_lab(
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
                    // This assumes that the image is in RGBA format.
                    let delta = compute_brightness_delta_rgba(&rgba_image, x, y, nx, ny);

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
            if has_at_least_three_equal_adjacent_pixels(&rgba_image, min_x, min_y)
                || has_at_least_three_equal_adjacent_pixels(&rgba_image, max_x, max_y)
            {
                result.antialiased_pixels.push(ImageLocation { x, y });
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
    use crate::stats::{calculate_f1_score, calculate_precision_score, calculate_recall_score};
    use crate::utils::nearly_equal;

    use super::*;

    #[test]
    fn test_validate_file_paths_identical_paths_error() {
        let result = validate_file_paths("same.png", "same.png");
        assert!(matches!(result, Err(GridSkipperError::IdenticalPaths)));
    }

    #[test]
    fn test_validate_file_paths_one_side_valid() {
        let result1 = validate_file_paths(
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png",
            "same.png",
        );

        let result2 = validate_file_paths(
            "same.png",
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png",
        );

        assert!(matches!(result1, Err(GridSkipperError::FileNotFound(_))));
        assert!(matches!(result2, Err(GridSkipperError::FileNotFound(_))));
    }

    #[test]
    fn test_validate_file_paths_both_sides_valid() {
        let result = validate_file_paths(
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png",
            "data/images/no-aa/lowercase-b-noto-serif-noaa.png",
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_file_not_found_error() {
        let result = validate_single_file_path("nonexistent.png");
        assert!(matches!(result, Err(GridSkipperError::FileNotFound(_))));
    }

    #[test]
    fn test_validate_single_file_path_exists() {
        let result =
            validate_single_file_path("data/images/analytic-aa/lowercase-b-noto-serif-aa.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_diff_images_happy_path() {
        let diff_result = diff_images(
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png",
            "data/images/no-aa/lowercase-b-noto-serif-noaa.png",
        );

        assert!(diff_result.is_ok());
        let diff = diff_result.unwrap();

        assert_eq!(
            diff,
            DiffResult {
                identical: true,
                diff_percentage: 0.0,
                diff_pixels: 0
            }
        );
    }

    #[test]
    fn test_lowercase_b_notosans_against_ground_truth() {
        let ground_truth_image =
            image::open("data/images/no-aa/lowercase-b-noto-serif-noAa.png").unwrap();
        let image_under_consideration =
            image::open("data/images/analytic-aa/lowercase-b-noto-serif-aa.png").unwrap();

        // Detect ground truth antialiasing
        let result = detect_antialiased_pixels_vysniauskas_rgba8(
            image_under_consideration,
            Some(ground_truth_image),
        );
        assert!(result.is_ok());

        let unwrapped_result = result.unwrap();
        let stats_opt = unwrapped_result.statistics;
        assert!(stats_opt.is_some());

        let stats = stats_opt.unwrap();
        assert_eq!(stats.num_ground_truth_antialiased_pixels, 793);
        assert_eq!(stats.num_false_negatives, 0);
        assert_eq!(stats.num_false_positives, 0);
        assert_eq!(stats.num_true_positives, 793);
    }

    #[test]
    fn test_und_image_with_alpha() {
        let image_under_consideration =
            image::open("data/images/analytic-aa/und-logo-aa.png").unwrap();
        let ground_truth_image = image::open("data/images/no-aa/und-logo-noaa.png").unwrap();

        // Detect ground truth antialiasing
        let result = detect_antialiased_pixels_vysniauskas_rgba8(
            image_under_consideration,
            Some(ground_truth_image),
        );
        assert!(result.is_ok());

        let unwrapped_result = result.unwrap();
        let stats_opt = unwrapped_result.statistics;
        assert!(stats_opt.is_some());

        let stats = stats_opt.unwrap();
        assert_eq!(unwrapped_result.num_antialiased_pixels, 192);
        assert_eq!(stats.num_ground_truth_antialiased_pixels, 1163);
        assert_eq!(stats.num_true_positives, 192);
        assert_eq!(stats.num_false_positives, 0);
        assert_eq!(stats.num_false_negatives, 700);
        let precision_score = calculate_precision_score(&unwrapped_result);
        let recall_score = calculate_recall_score(&unwrapped_result);

        assert_eq!(precision_score, Some(1.0));
        assert_eq!(recall_score, Some(0.21524663677130046));

        assert_eq!(
            calculate_f1_score(precision_score, recall_score),
            Some(0.3542435424354244)
        );
    }

    #[test]
    fn test_und_image_with_blackbg() {
        let image_under_consideration =
            image::open("data/images/analytic-aa/und-logo-blackbg-aa.png").unwrap();
        let ground_truth_image =
            image::open("data/images/no-aa/und-logo-blackbg-noaa.png").unwrap();

        // Detect ground truth antialiasing
        let result = detect_antialiased_pixels_vysniauskas_rgba8(
            image_under_consideration,
            Some(ground_truth_image),
        );
        assert!(result.is_ok());

        let unwrapped_result = result.unwrap();
        let stats_opt = unwrapped_result.statistics;
        assert!(stats_opt.is_some());

        let stats = stats_opt.unwrap();
        assert_eq!(unwrapped_result.num_antialiased_pixels, 1094);
        assert_eq!(stats.num_ground_truth_antialiased_pixels, 1163);
        assert_eq!(stats.num_true_positives, 1094);
        assert_eq!(stats.num_false_positives, 0);
        assert_eq!(stats.num_false_negatives, 63);

        let precision_score = calculate_precision_score(&unwrapped_result);
        let recall_score = calculate_recall_score(&unwrapped_result);

        assert_eq!(precision_score, Some(1.0));
        assert_eq!(recall_score, Some(0.9455488331892826));

        let f1_score_opt = calculate_f1_score(precision_score, recall_score);
        assert!(f1_score_opt.is_some());

        let f1_score = f1_score_opt.unwrap();
        assert!(nearly_equal(f1_score, 0.9720124389, 1e-10));
    }
}
