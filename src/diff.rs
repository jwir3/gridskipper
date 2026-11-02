use image::{DynamicImage, Rgba};

use crate::{error::GridSkipperError, results::DiffResult};

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
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            threshold: 0.1f64,
            alpha: 0.1f64,
            aaColor: None,
            diffColor: Rgba([255, 0, 0, 128]),
        }
    }
}

fn get_rgba_byte_view(img: &DynamicImage) -> Vec<u8> {
    img.to_rgba8().into_raw()
}

pub fn fast_check_for_identical_images(left_side: DynamicImage,
                                       right_side: DynamicImage)
    -> Result<Option<DiffResult>, GridSkipperError> {
    if left_side.width() != right_side.width() || left_side.height() != right_side.height() {
        return Ok(None);
    }

    let left_bytes = get_rgba_byte_view(&left_side);
    let right_bytes = get_rgba_byte_view(&right_side);

    if left_bytes == right_bytes {
        Ok(Some(DiffResult { identical: true, diff_pixels: 0, diff_percentage: 0.0 }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::fast_check_for_identical_images;
    use crate::results::DiffResult;
    use image::{DynamicImage, ImageBuffer, Rgb, RgbaImage};

    #[test]
    fn test_fast_check_for_identical_images() {
        let identical_left = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([10, 20, 30, 255]),
        ));
        let identical_right = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([10, 20, 30, 255]),
        ));

        assert_eq!(
            fast_check_for_identical_images(identical_left, identical_right).unwrap(),
            Some(DiffResult {
                identical: true,
                diff_pixels: 0,
                diff_percentage: 0.0,
            })
        );

        let different_left = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([10, 20, 30, 255]),
        ));
        let different_right = DynamicImage::ImageRgba8(RgbaImage::from_pixel(
            2,
            2,
            image::Rgba([10, 20, 31, 255]),
        ));

        assert_eq!(
            fast_check_for_identical_images(different_left, different_right).unwrap(),
            None
        );
    }

    #[test]
    fn test_fast_check_for_identical_images_rgb() {
        let rgb_identical_left = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(
            2,
            2,
            Rgb([10, 20, 30]),
        ));
        let rgb_identical_right = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(
            2,
            2,
            Rgb([10, 20, 30]),
        ));

        assert_eq!(
            fast_check_for_identical_images(rgb_identical_left, rgb_identical_right).unwrap(),
            Some(DiffResult {
                identical: true,
                diff_pixels: 0,
                diff_percentage: 0.0,
            })
        );

        let rgb_different_left = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(
            2,
            2,
            Rgb([10, 20, 30]),
        ));
        let rgb_different_right = DynamicImage::ImageRgb8(ImageBuffer::from_pixel(
            2,
            2,
            Rgb([10, 20, 31]),
        ));

        assert_eq!(
            fast_check_for_identical_images(rgb_different_left, rgb_different_right).unwrap(),
            None
        );
    }
}