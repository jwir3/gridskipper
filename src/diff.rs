use image::{DynamicImage, Rgba};

use crate::{color::OkLabPixel, error::GridSkipperError, results::DiffResult, validate};

pub struct DiffOptions {
    /// The threshold of difference that is tolerable.
    ///
    /// "Threshold" is a difficult concept, in practice, to explain to users because it isn't
    /// intuitive once we get into the Y, Cb, Cr space.
    ///
    /// TODO_jwir3: Explain this a little better.
    threshold: f64,

    /// Opacity of original image in diff output.
    ///
    /// XXX_jwir3: I think this is a little backwards. We should consider removing this option, and,
    ///            instead, make the diff pixel itself have an opacity.
    alpha: f64,

    /// The color, as an Rgba<u8>, to make pixels in the diff output image that are detected to be
    /// pixels shaded as the result of antialiasing. Defaults to `None`, in which case antialiased
    /// pixels will not be shown in the resulting diff image.
    aaColor: Option<Rgba<u8>>,

    /// The color, as an Rgba<u8>, to set in the resulting diff image for pixels that are different
    /// between the two images being compared.
    diffColor: Rgba<u8>,
    // diffColorAlternate: Option<Rgba<u8>>,

    diff_image_output_path: Option<String>,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            threshold: 0.1f64,
            alpha: 0.1f64,
            aaColor: None,
            diffColor: Rgba([255, 0, 0, 128]),
            diff_image_output_path: None,
        }
    }
}

fn get_rgba_byte_view(img: &DynamicImage) -> Vec<u8> {
    img.to_rgba8().into_raw()
}

pub fn fast_check_for_identical_images(
    left_side: DynamicImage,
    right_side: DynamicImage,
) -> Result<Option<DiffResult>, GridSkipperError> {
    if left_side.width() != right_side.width() || left_side.height() != right_side.height() {
        return Ok(None);
    }

    let left_bytes = get_rgba_byte_view(&left_side);
    let right_bytes = get_rgba_byte_view(&right_side);

    if left_bytes == right_bytes {
        Ok(Some(DiffResult {
            identical: true,
            diff_pixels: 0,
            diff_percentage: 0.0,
        }))
    } else {
        Ok(None)
    }
}

/// Compare two RGBA pixels using a hybrid distance metric.
///
/// The HyAB distance metric is a perceptual color difference metric that is designed to be more
/// accurate than simple Euclidean distance. It uses the Manhattan distance for lightness and the
/// Euclidean distance for chroma channels.
///
/// @returns 0 if the distance is within the threshold, 1 if the left pixel is lighter, and -1 if
///          the right pixel is lighter.
///
pub fn color_delta_hybrid_ok_lab(left: &Rgba<u8>, right: &Rgba<u8>, threshold: f64) -> i8 {
    // First, convert the RGBA pixels to the OKLab color space.
    let left_oklab = OkLabPixel::from(*left);
    let right_oklab = OkLabPixel::from(*right);

    let distance = (left_oklab.l - right_oklab.l).abs()
        + ((left_oklab.a - right_oklab.a).powi(2) + (left_oklab.b - right_oklab.b).powi(2)).sqrt();
    if distance <= threshold {
        0
    } else if left_oklab.l > right_oklab.l {
        1
    } else {
        -1
    }
}

pub fn output_diff_image(
    width: u32,
    height: u32,
    differences: &[i8],
    options: &DiffOptions,
) {
    let mut diff_image = image::RgbaImage::new(width, height);

    for (i, &delta) in differences.iter().enumerate() {
        let x = (i as u32) % width;
        let y = (i as u32) / width;

        let pixel = match delta {
            0 => {
                // Pixels are similar; draw background as grayscale image blended with white
                let alpha = options.alpha;
                let gray_value = ((1.0 - alpha) * 255.0 + alpha * 255.0) as u8;
                Rgba([gray_value, gray_value, gray_value, 255])
            }
            1 => options.diffColor,
            -1 => options.diffColor,
            _ => unreachable!(),
        };

        diff_image.put_pixel(x, y, pixel);
    }

    if let Some(output_path) = &options.diff_image_output_path {
        diff_image.save(output_path).unwrap();
    }
}

pub fn compare_images_pixel_by_pixel(
    left_side: DynamicImage,
    right_side: DynamicImage,
    options: &DiffOptions,
) -> Result<DiffResult, GridSkipperError> {
    let left_image_rgba8 = match left_side.as_rgba8() {
        Some(x) => x,
        None => {
            return Err(GridSkipperError::InvalidImageFormat(
                "Failed to convert left image to RGBA8".into(),
            ));
        }
    };

    let left_image_count_pixels = left_image_rgba8.width() * left_image_rgba8.height();

    let right_image_rgba8 = match right_side.as_rgba8() {
        Some(x) => x,
        None => {
            return Err(GridSkipperError::InvalidImageFormat(
                "Failed to convert right image to RGBA8".into(),
            ));
        }
    };

    let differences: Vec<i8> = left_image_rgba8
        .pixels()
        .zip(right_image_rgba8.pixels())
        .map(|(left_pixel, right_pixel)| {
            let color_delta = color_delta_hybrid_ok_lab(left_pixel, right_pixel, options.threshold);
            color_delta
        })
        .collect();

    let diff_count = differences.iter().filter(|&&x| x != 0).count();

    if diff_count > 0 && options.diff_image_output_path.is_some() {
        output_diff_image(left_side.width(), left_side.height(), &differences, options);
    }

    Ok(DiffResult {
        identical: diff_count == 0,
        diff_pixels: diff_count as u32,
        diff_percentage: (diff_count as f64) / (left_image_count_pixels as f64) * 100.0,
    })
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
/// use gridskipper::diff::diff_images;
/// use gridskipper::diff::DiffOptions;
///
/// match diff_images("image1.png", "image2.png", &DiffOptions::default()) {
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
pub fn diff_images(
    left_side: &str,
    right_side: &str,
    options: &DiffOptions,
) -> Result<DiffResult, GridSkipperError> {
    // Validate input
    validate::validate_file_paths(left_side, right_side)?;

    // Validate images and metadata
    let (left_image, right_image) = validate::validate_images_and_metadata(left_side, right_side)?;

    // First, let's perform a fast check for identical images.
    if let Some(diff_result) =
        fast_check_for_identical_images(left_image.clone(), right_image.clone())?
    {
        return Ok(diff_result);
    }

    // If that didn't return anything, then we're going to do our full difference calculation.
    let results = compare_images_pixel_by_pixel(left_image, right_image, options);

    results
}

#[cfg(test)]
mod tests {
    use super::fast_check_for_identical_images;
    use crate::{
        diff::{DiffOptions, diff_images},
        results::DiffResult,
    };
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgb, RgbaImage};

    #[test]
    fn test_fast_check_for_identical_images() {
        let identical_left =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255])));
        let identical_right =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255])));

        assert_eq!(
            fast_check_for_identical_images(identical_left, identical_right).unwrap(),
            Some(DiffResult {
                identical: true,
                diff_pixels: 0,
                diff_percentage: 0.0,
            })
        );

        let different_left =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 30, 255])));
        let different_right =
            DynamicImage::ImageRgba8(RgbaImage::from_pixel(2, 2, image::Rgba([10, 20, 31, 255])));

        assert_eq!(
            fast_check_for_identical_images(different_left, different_right).unwrap(),
            None
        );
    }

    #[test]
    fn test_fast_check_for_identical_images_rgb() {
        let rgb_identical_left =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(2, 2, Rgb([10, 20, 30])));
        let rgb_identical_right =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(2, 2, Rgb([10, 20, 30])));

        assert_eq!(
            fast_check_for_identical_images(rgb_identical_left, rgb_identical_right).unwrap(),
            Some(DiffResult {
                identical: true,
                diff_pixels: 0,
                diff_percentage: 0.0,
            })
        );

        let rgb_different_left =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(2, 2, Rgb([10, 20, 30])));
        let rgb_different_right =
            DynamicImage::ImageRgb8(ImageBuffer::from_pixel(2, 2, Rgb([10, 20, 31])));

        assert_eq!(
            fast_check_for_identical_images(rgb_different_left, rgb_different_right).unwrap(),
            None
        );
    }

    #[test]
    fn test_diff_images_no_difference_default_options() {
        let diff_result = diff_images(
            "data/images/diff/baseline-text-shift.png",
            "data/images/diff/baseline-text-shift.png",
            &DiffOptions::default(),
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
    fn test_different_images_small() {
        let diff_options = DiffOptions {
            threshold: 0.1,
            alpha: 0.1,
            aaColor: None,
            diffColor: image::Rgba([255, 0, 0, 128]),
            diff_image_output_path: Some("diff.png".to_string()),
        };

        let result = diff_images(
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png",
            "data/images/no-aa/lowercase-b-noto-serif-noaa.png",
            &diff_options,
        );

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            DiffResult {
                identical: false,
                diff_pixels: 646,
                diff_percentage: 2.749755246243562 // Pixelmatch, without AA, returns:
                                                   // diff_pixels: 602,
                                                   // diff_percentage: 2.56,
            }
        );

        // Read the diff image file and check that it is the same as the reference file.
        let diff_image = image::open("diff.png").unwrap();
        let expected_diff_image = image::open("data/images/diff/noto_serif_lowercase_b_analyticaa_vs_noaa.png").unwrap();

        assert_eq!(diff_image.dimensions(), expected_diff_image.dimensions());
        assert_eq!(diff_image.to_rgba8(), expected_diff_image.to_rgba8());

        // Remove the diff.png image file after the test
        std::fs::remove_file("diff.png").unwrap();
    }
}
