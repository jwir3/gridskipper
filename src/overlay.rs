use image::{DynamicImage, GenericImage, Rgba, RgbaImage};

use crate::error::GridSkipperError;
use crate::results::AntialiasDetectionResult;

/// Generate an antialiased overlay image based on the provided `AntialiasDetectionResult`.
///
/// Parameters:
/// - `width`: The width of the overlay image. This must be the same as the width of the base image.
/// - `height`: The height of the overlay image. This must be the same as the height of the base image.
/// - `result`: The `AntialiasDetectionResult` used to generate the overlay.
/// - `color`: The color to use for the overlay. If alpha channel is equal to `255`, then this color will completely
///            replace the pixel in the base image when overlaid.
///
/// Returns: A `DynamicImage` representing the antialiased overlay.
pub fn generate_antialiased_overlay(
    width: u32,
    height: u32,
    result: &AntialiasDetectionResult,
    color: Rgba<u8>,
) -> DynamicImage {
    let mut image = DynamicImage::new_rgba8(width, height);

    for y in 0..height {
        for x in 0..width {
            let pixel_color = if result.is_antialiased(x, y) {
                color
            } else {
                Rgba([0, 0, 0, 0])
            };
            image.put_pixel(x, y, pixel_color);
        }
    }

    image
}

/// Composite two images in an overlay fashion, where the overlay image is placed on top of the base image, and the
/// color values of the overlay image are blended with the base image using alpha and (1-alpha), respectively.
///
/// Typically, in image processing, this is referred to as "alpha blending". See also Jim Blinn's Corner: Dirty Pixels
/// pp. 181 for information on how to implement this properly. Note that this implementation assumes _only_ two images,
/// and that the background image is assumed to be fully opaque (that is, it does not have its own alpha value).
///
/// Parameters:
/// - `base_image`: The base image on which the overlay image will be placed. The width and height must match that of
///   the overlay image.
/// - `overlay_image`: The overlay image that will be placed on top of the base image. The width and height must match
///   that of the base image.
///
/// Returns:
/// - `Result<DynamicImage, GridSkipperError>`: A result containing the composite image or an error if the dimensions
///   do not match.
pub fn composite_overlay(
    base_image: RgbaImage,
    overlay_image: RgbaImage,
) -> Result<DynamicImage, GridSkipperError> {
    if base_image.width() != overlay_image.width() || base_image.height() != overlay_image.height()
    {
        return Err(GridSkipperError::OverlayCompositingError(
            "Image dimensions do not match".to_string(),
        ));
    }

    let mut image = DynamicImage::new_rgba8(base_image.width(), base_image.height());
    for y in 0..base_image.height() {
        for x in 0..base_image.width() {
            let base_pixel = base_image.get_pixel(x, y);
            let overlay_pixel = overlay_image.get_pixel(x, y);

            let alpha_overlay = overlay_pixel[3];
            let red = (base_pixel[0] as u16 * (255 - alpha_overlay as u16)
                + overlay_pixel[0] as u16 * alpha_overlay as u16)
                / 255;
            let green = (base_pixel[1] as u16 * (255 - alpha_overlay as u16)
                + overlay_pixel[1] as u16 * alpha_overlay as u16)
                / 255;
            let blue = (base_pixel[2] as u16 * (255 - alpha_overlay as u16)
                + overlay_pixel[2] as u16 * alpha_overlay as u16)
                / 255;

            // Alpha should now be 255 because we've composited the alpha values. Note that this isn't _quite_ correct,
            // because we're not taking into account the alpha values of the base image. (That is, there could be more
            // than one layer that needs compositing, but it's good enough for now).
            image.put_pixel(x, y, Rgba([red as u8, green as u8, blue as u8, 255]));
        }
    }

    Ok(image)
}

#[cfg(test)]
mod tests {
    use image::GenericImageView;

    use crate::base::ImageLocation;

    use super::*;

    fn get_dummy_image() -> image::DynamicImage {
        let mut dummy_image = image::DynamicImage::new_rgb8(5, 5);

        // The further we get to the right, the more green is added
        // and the further we go down, the more blue is added.
        let mut red: u8;
        let mut green: u8;
        let mut blue = 0;

        let per_px_difference = 255 / 5;

        for y in 0..5 {
            red = 255;
            green = 0;

            for x in 0..5 {
                dummy_image.put_pixel(x, y, Rgba([red as u8, green as u8, blue as u8, 255]));
                red = red - per_px_difference;
                green = green + per_px_difference;
            }

            blue = blue + per_px_difference;
        }

        dummy_image
    }

    #[test]
    pub fn test_generate_antialiased_overlay() {
        // Create a dummy antialiased results that has a single pixel in the center of a 5x5 image that is AA
        let results = AntialiasDetectionResult {
            number_pixels: 5 * 5,
            num_antialiased_pixels: 1,
            antialiased_pixels: vec![ImageLocation { x: 2, y: 2 }],
            statistics: None,
        };

        let overlay = generate_antialiased_overlay(5, 5, &results, Rgba([255, 0, 0, 128]));

        // Hah, someday when we implement the diff side, we could actually use the diff to test
        assert_eq!(overlay.width(), 5);
        assert_eq!(overlay.height(), 5);
        assert_eq!(overlay.get_pixel(0, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(0, 1), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(0, 2), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(0, 3), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(0, 4), Rgba([0, 0, 0, 0]));

        assert_eq!(overlay.get_pixel(1, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(1, 1), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(1, 2), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(1, 3), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(1, 4), Rgba([0, 0, 0, 0]));

        assert_eq!(overlay.get_pixel(2, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(2, 1), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(2, 2), Rgba([255, 0, 0, 128]));
        assert_eq!(overlay.get_pixel(2, 3), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(2, 4), Rgba([0, 0, 0, 0]));

        assert_eq!(overlay.get_pixel(3, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(3, 1), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(3, 2), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(3, 3), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(3, 4), Rgba([0, 0, 0, 0]));

        assert_eq!(overlay.get_pixel(4, 0), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(4, 1), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(4, 2), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(4, 3), Rgba([0, 0, 0, 0]));
        assert_eq!(overlay.get_pixel(4, 4), Rgba([0, 0, 0, 0]));
    }

    #[test]
    pub fn test_composite_overlay() {
        let dummy_image = get_dummy_image();

        assert_eq!(dummy_image.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(dummy_image.get_pixel(0, 1), Rgba([255, 0, 51, 255]));
        assert_eq!(dummy_image.get_pixel(0, 2), Rgba([255, 0, 102, 255]));
        assert_eq!(dummy_image.get_pixel(0, 3), Rgba([255, 0, 153, 255]));
        assert_eq!(dummy_image.get_pixel(0, 4), Rgba([255, 0, 204, 255]));

        assert_eq!(dummy_image.get_pixel(1, 0), Rgba([204, 51, 0, 255]));
        assert_eq!(dummy_image.get_pixel(1, 1), Rgba([204, 51, 51, 255]));
        assert_eq!(dummy_image.get_pixel(1, 2), Rgba([204, 51, 102, 255]));
        assert_eq!(dummy_image.get_pixel(1, 3), Rgba([204, 51, 153, 255]));
        assert_eq!(dummy_image.get_pixel(1, 4), Rgba([204, 51, 204, 255]));

        assert_eq!(dummy_image.get_pixel(2, 0), Rgba([153, 102, 0, 255]));
        assert_eq!(dummy_image.get_pixel(2, 1), Rgba([153, 102, 51, 255]));
        assert_eq!(dummy_image.get_pixel(2, 2), Rgba([153, 102, 102, 255]));
        assert_eq!(dummy_image.get_pixel(2, 3), Rgba([153, 102, 153, 255]));
        assert_eq!(dummy_image.get_pixel(2, 4), Rgba([153, 102, 204, 255]));

        assert_eq!(dummy_image.get_pixel(3, 0), Rgba([102, 153, 0, 255]));
        assert_eq!(dummy_image.get_pixel(3, 1), Rgba([102, 153, 51, 255]));
        assert_eq!(dummy_image.get_pixel(3, 2), Rgba([102, 153, 102, 255]));
        assert_eq!(dummy_image.get_pixel(3, 3), Rgba([102, 153, 153, 255]));
        assert_eq!(dummy_image.get_pixel(3, 4), Rgba([102, 153, 204, 255]));

        assert_eq!(dummy_image.get_pixel(4, 0), Rgba([51, 204, 0, 255]));
        assert_eq!(dummy_image.get_pixel(4, 1), Rgba([51, 204, 51, 255]));
        assert_eq!(dummy_image.get_pixel(4, 2), Rgba([51, 204, 102, 255]));
        assert_eq!(dummy_image.get_pixel(4, 3), Rgba([51, 204, 153, 255]));
        assert_eq!(dummy_image.get_pixel(4, 4), Rgba([51, 204, 204, 255]));

        let results = AntialiasDetectionResult {
            number_pixels: 5 * 5,
            num_antialiased_pixels: 1,
            antialiased_pixels: vec![ImageLocation { x: 2, y: 2 }],
            statistics: None,
        };

        let overlay = generate_antialiased_overlay(5, 5, &results, Rgba([255, 0, 0, 128]))
            .as_rgba8()
            .unwrap()
            .to_owned();

        let dummy_image_rgba8 = dummy_image.to_rgba8();
        let dummy_image_with_overlay = composite_overlay(dummy_image_rgba8, overlay);

        assert!(dummy_image_with_overlay.is_ok());

        let overlaid = dummy_image_with_overlay.unwrap();

        assert_eq!(overlaid.get_pixel(0, 0), Rgba([255, 0, 0, 255]));
        assert_eq!(overlaid.get_pixel(0, 1), Rgba([255, 0, 51, 255]));
        assert_eq!(overlaid.get_pixel(0, 2), Rgba([255, 0, 102, 255]));
        assert_eq!(overlaid.get_pixel(0, 3), Rgba([255, 0, 153, 255]));
        assert_eq!(overlaid.get_pixel(0, 4), Rgba([255, 0, 204, 255]));

        assert_eq!(overlaid.get_pixel(1, 0), Rgba([204, 51, 0, 255]));
        assert_eq!(overlaid.get_pixel(1, 1), Rgba([204, 51, 51, 255]));
        assert_eq!(overlaid.get_pixel(1, 2), Rgba([204, 51, 102, 255]));
        assert_eq!(overlaid.get_pixel(1, 3), Rgba([204, 51, 153, 255]));
        assert_eq!(overlaid.get_pixel(1, 4), Rgba([204, 51, 204, 255]));

        assert_eq!(overlaid.get_pixel(2, 0), Rgba([153, 102, 0, 255]));
        assert_eq!(overlaid.get_pixel(2, 1), Rgba([153, 102, 51, 255]));
        assert_eq!(overlaid.get_pixel(2, 2), Rgba([204, 50, 50, 255]));
        assert_eq!(overlaid.get_pixel(2, 3), Rgba([153, 102, 153, 255]));
        assert_eq!(overlaid.get_pixel(2, 4), Rgba([153, 102, 204, 255]));

        assert_eq!(overlaid.get_pixel(3, 0), Rgba([102, 153, 0, 255]));
        assert_eq!(overlaid.get_pixel(3, 1), Rgba([102, 153, 51, 255]));
        assert_eq!(overlaid.get_pixel(3, 2), Rgba([102, 153, 102, 255]));
        assert_eq!(overlaid.get_pixel(3, 3), Rgba([102, 153, 153, 255]));
        assert_eq!(overlaid.get_pixel(3, 4), Rgba([102, 153, 204, 255]));

        assert_eq!(overlaid.get_pixel(4, 0), Rgba([51, 204, 0, 255]));
        assert_eq!(overlaid.get_pixel(4, 1), Rgba([51, 204, 51, 255]));
        assert_eq!(overlaid.get_pixel(4, 2), Rgba([51, 204, 102, 255]));
        assert_eq!(overlaid.get_pixel(4, 3), Rgba([51, 204, 153, 255]));
        assert_eq!(overlaid.get_pixel(4, 4), Rgba([51, 204, 204, 255]));
    }
}
