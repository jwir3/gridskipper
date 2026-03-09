//! Generic utility functions for Gridskipper.
use image::{DynamicImage, GrayImage, Pixel, Rgba, RgbaImage};

use crate::base::PixelNeighbors;
use crate::color::CieLabPixel;

/// Retrieve the indices of neighboring pixels in an image, given a center pixel. Respects image boundaries so that
/// the indices are within the image dimensions.
/// Parameters:
/// - `x`: The x-coordinate of the center pixel.
/// - `y`: The y-coordinate of the center pixel.
/// - `width`: The width of the image.
/// - `height`: The height of the image.
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
    adjacent_pixel[0] as i16 - center_pixel[0] as i16
}

pub fn compute_brightness_delta_rgba(
    rgba_image: &RgbaImage,
    center_x: u32,
    center_y: u32,
    adjacent_x: u32,
    adjacent_y: u32,
) -> i16 {
    let center_pixel = rgba_image.get_pixel(center_x, center_y);
    let center_pixel_intensity = get_intensity_value_luma(center_pixel);
    let adjacent_pixel = rgba_image.get_pixel(adjacent_x, adjacent_y);
    let adjacent_pixel_intensity = get_intensity_value_luma(adjacent_pixel);
    adjacent_pixel_intensity as i16 - center_pixel_intensity as i16
}

pub fn compute_brightness_delta_alpha(
    rgba_image: &RgbaImage,
    center_x: u32,
    center_y: u32,
    adjacent_x: u32,
    adjacent_y: u32,
) -> i16 {
    let center_pixel = rgba_image.get_pixel(center_x, center_y);
    let center_pixel_intensity = get_intensity_value_alpha(center_pixel);
    let adjacent_pixel = rgba_image.get_pixel(adjacent_x, adjacent_y);
    let adjacent_pixel_intensity = get_intensity_value_alpha(adjacent_pixel);
    adjacent_pixel_intensity as i16 - center_pixel_intensity as i16
}

pub fn compute_brightness_delta_lab(
    rgba_image: &RgbaImage,
    center_x: u32,
    center_y: u32,
    adjacent_x: u32,
    adjacent_y: u32,
) -> f64 {
    let center_pixel = rgba_image.get_pixel(center_x, center_y);
    let center_pixel_intensity = CieLabPixel::from(*center_pixel).l;
    let adjacent_pixel = rgba_image.get_pixel(adjacent_x, adjacent_y);
    let adjacent_pixel_intensity = CieLabPixel::from(*adjacent_pixel).l;

    adjacent_pixel_intensity - center_pixel_intensity
}

/// Check if there are at least three equal adjacent pixels in an RGBA image.
///
/// Parameters:
/// - `image`: The image, in RGBA format.
/// - `x`: The x-coordinate of the center pixel.
/// - `y`: The y-coordinate of the center pixel.
///
/// TODO_jwir3:
/// We really should make this less specific to grayscale images. It should actually detect whether the
///_luma_ is the same for the adjacent pixels.
///
/// Returns:
/// - `true` if there are at least three equal adjacent pixels, `false` otherwise.
pub fn has_at_least_three_equal_adjacent_pixels(image: &RgbaImage, x: u32, y: u32) -> bool {
    let pixel_neighbors =
        get_neighbors_respect_image_bounds(x as i32, y as i32, image.width(), image.height());
    // We add a bonus match if we're on an edge to account for a reduced constraint when we don't have enough data.
    let mut same_pixel_count = if pixel_neighbors.left == x
        || pixel_neighbors.right == x
        || pixel_neighbors.top == y
        || pixel_neighbors.bottom == y
    {
        1
    } else {
        0
    };

    for ny in pixel_neighbors.top..pixel_neighbors.bottom + 1 {
        for nx in pixel_neighbors.left..pixel_neighbors.right + 1 {
            if nx != x || ny != y {
                // XXX_jwir3: We lose a bit of fidelity here if we don't convert to intensity when we
                //            perform the comparison (that is, if we do pixel1 == pixel2).
                let pixel1 = image.get_pixel(nx, ny);
                let pixel1_intensity = get_intensity_value_luma(pixel1);
                let pixel2 = image.get_pixel(x, y);
                let pixel2_intensity = get_intensity_value_luma(pixel2);
                if pixel1_intensity == pixel2_intensity {
                    same_pixel_count += 1;
                }

                if same_pixel_count > 2 {
                    return true;
                }
            }
        }
    }

    same_pixel_count > 2
}

// Test utility function to compare floating point numbers
pub fn nearly_equal(a: f64, b: f64, epsilon: f64) -> bool {
    (a - b).abs() < epsilon
}

/// Retrieve an intensity value from a pixel color using Luma (grayscale value) as intensity.
///
/// Parameters:
/// - `pixel_color`: The pixel color to retrieve the intensity value from.
///
/// Returns:
/// - The intensity value of the pixel color.
pub fn get_intensity_value_luma(pixel_color: &Rgba<u8>) -> u8 {
    pixel_color.to_luma()[0]
}

/// Retrieve an intensity value from a pixel color using Alpha (transparency value) as intensity.
///
/// Parameters:
/// - `pixel_color`: The pixel color to retrieve the intensity value from.
///
/// Returns:
/// - The intensity value of the pixel color.
pub fn get_intensity_value_alpha(pixel_color: &Rgba<u8>) -> u8 {
    pixel_color.0[3]
}

/// Check if an image has at least one pixel that is semi-transparent.
///
/// Parameters:
/// - `image`: A reference to the `DynamicImage` which should be checked for transparency.
///
/// Returns:
/// - `true`, if at least one pixel has an alpha channel that corresponds to something less than `255`;
///   `false`, otherwise.
pub fn check_image_has_transparency(image: &DynamicImage) -> bool {
    let rgba_image = match image.as_rgba8() {
        Some(img) => img,
        None => return false,
    };

    let width = rgba_image.width();
    let height = rgba_image.height();

    for j in 0..height {
        for i in 0..width {
            let pixel = rgba_image.get_pixel(i, j);

            if pixel.0[3] < 255 {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
pub mod tests {
    use approx::assert_abs_diff_eq;
    use image::{GrayImage, ImageReader, Luma, Rgba, RgbaImage};

    use crate::{
        get_neighbors_respect_image_bounds,
        utils::{
            check_image_has_transparency, compute_brightness_delta_alpha,
            compute_brightness_delta_grayscale, has_at_least_three_equal_adjacent_pixels,
        },
    };

    use super::compute_brightness_delta_lab;

    #[test]
    fn test_get_neighbors_respect_image_bounds_3x3_center() {
        let neighbors = get_neighbors_respect_image_bounds(1, 1, 3, 3);

        assert_eq!(neighbors.top, 0);
        assert_eq!(neighbors.bottom, 2);
        assert_eq!(neighbors.left, 0);
        assert_eq!(neighbors.right, 2);
    }

    #[test]
    fn test_get_neighbors_respect_image_bounds_3x3_top_left() {
        let neighbors = get_neighbors_respect_image_bounds(0, 0, 3, 3);

        assert_eq!(neighbors.top, 0);
        assert_eq!(neighbors.bottom, 1);
        assert_eq!(neighbors.left, 0);
        assert_eq!(neighbors.right, 1);
    }

    #[test]
    fn test_get_neighbors_respect_image_bounds_3x3_bottom_right() {
        let neighbors = get_neighbors_respect_image_bounds(2, 2, 3, 3);

        assert_eq!(neighbors.top, 1);
        assert_eq!(neighbors.bottom, 2);
        assert_eq!(neighbors.left, 1);
        assert_eq!(neighbors.right, 2);
    }

    #[test]
    fn test_compute_brightness_delta_cielab() {
        let mut rgba_image = RgbaImage::new(3, 3);
        for i in 0..2 {
            for j in 0..2 {
                rgba_image.put_pixel(i, j, Rgba([255, 0, 0, 255]));
            }
        }

        rgba_image.put_pixel(1, 1, Rgba([0, 255, 0, 255]));

        assert_abs_diff_eq!(
            -34.49,
            compute_brightness_delta_lab(&rgba_image, 1, 1, 0, 0),
            epsilon = 0.01
        );
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

    #[test]
    fn test_compute_brightness_delta_alpha() {
        let alpha_image = ImageReader::open("data/images/red-example.png")
            .expect("Failed to open image")
            .decode()
            .expect("Failed to decode image")
            .to_rgba8();

        let brightness_delta1 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 0, 0);
        assert_eq!(brightness_delta1, 127);

        let brightness_delta2 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 1, 0);
        assert_eq!(brightness_delta2, 63);

        let brightness_delta3 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 2, 0);
        assert_eq!(brightness_delta3, 0);

        let brightness_delta4 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 0, 1);
        assert_eq!(brightness_delta4, 63);

        let brightness_delta5 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 1, 1);
        assert_eq!(brightness_delta5, 0);

        let brightness_delta6 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 2, 1);
        assert_eq!(brightness_delta6, -64);

        let brightness_delta7 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 0, 2);
        assert_eq!(brightness_delta7, 0);

        let brightness_delta8 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 1, 2);
        assert_eq!(brightness_delta8, -64);

        let brightness_delta9 = compute_brightness_delta_alpha(&alpha_image, 1, 1, 2, 2);
        assert_eq!(brightness_delta9, -128);
    }

    #[test]
    fn test_check_image_has_transparency() {
        let image = ImageReader::open("data/images/red-example.png")
            .expect("Failed to open image")
            .decode()
            .expect("Failed to decode image");

        assert!(check_image_has_transparency(&image));

        let image = ImageReader::open("data/images/analytic-aa/und-logo-blackbg-aa.png")
            .expect("Failed to open image")
            .decode()
            .expect("Failed to decode image");

        assert!(!check_image_has_transparency(&image));
    }

    #[test]
    fn test_has_at_least_three_equal_adjacent_pixels() {
        let image = RgbaImage::from_pixel(3, 3, Rgba([128, 128, 128, 255]));
        assert!(has_at_least_three_equal_adjacent_pixels(&image, 1, 1));

        let mut image = RgbaImage::new(3, 3);
        for y in 0..3 {
            for x in 0..3 {
                let val = (y * 3 + x) as u8;
                image.put_pixel(x, y, Rgba([val, val, val, 255]));
            }
        }
        assert!(!has_at_least_three_equal_adjacent_pixels(&image, 1, 1));
    }
}
