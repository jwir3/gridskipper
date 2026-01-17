//! Generic utility functions for Gridskipper.

use image::{GrayImage, RgbaImage, buffer::ConvertBuffer};

use crate::base::PixelNeighbors;

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

/// Check if there are at least three equal adjacent pixels in a grayscale image.
///
/// Parameters:
/// - `image`: The grayscale image.
/// - `x`: The x-coordinate of the center pixel.
/// - `y`: The y-coordinate of the center pixel.
///
/// TODO_jwir3:
/// We really should make this less specific to grayscale images. It should actually detect whether the
///_luma_ is the same for the adjacent pixels.
///
/// Returns:
/// - `true` if there are at least three equal adjacent pixels, `false` otherwise.
pub fn has_at_least_three_equal_adjacent_pixels(image: &GrayImage, x: u32, y: u32) -> bool {
    let pixel_neighbors =
        get_neighbors_respect_image_bounds(x as i32, y as i32, image.width(), image.height());
    let mut same_pixel_count = 0;
    for ny in pixel_neighbors.top..pixel_neighbors.bottom {
        for nx in pixel_neighbors.left..pixel_neighbors.right {
            let pixel1 = image.get_pixel(nx, ny);
            let pixel2 = image.get_pixel(x, y);
            if pixel1 == pixel2 {
                same_pixel_count += 1;
            }

            if same_pixel_count >= 3 {
                return true;
            }
        }
    }

    return same_pixel_count >= 3;
}

#[cfg(test)]
pub mod tests {
    use image::{GrayImage, Luma};

    use crate::utils::{
        compute_brightness_delta_grayscale, has_at_least_three_equal_adjacent_pixels,
    };

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
    fn test_has_at_least_three_equal_adjacent_pixels() {
        let image = GrayImage::from_pixel(3, 3, Luma([128]));
        assert!(has_at_least_three_equal_adjacent_pixels(&image, 1, 1));

        let image = GrayImage::from_vec(3, 3, vec![0, 1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        assert!(!has_at_least_three_equal_adjacent_pixels(&image, 1, 1));
    }
}
