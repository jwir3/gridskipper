use image::{Luma, Rgb, RgbImage, Rgba, RgbaImage};
use ndarray::arr2;

// This assumes a D65 white point. Note that the rows sum to the D65 white point:
// $X=0.95047, Y=1.00000, Z=1.08883$
const RGB_TO_CIEXYZ_MATRIX: [[f64; 3]; 3] = [
    [0.4124564, 0.3575761, 0.1804375],
    [0.2126729, 0.7151522, 0.0721750],
    [0.0193339, 0.1191920, 0.9503041],
];

const CIEXYZ_REFERENCE_POINT_D65: (f64, f64, f64) = (95.047, 100.00, 108.883);

/// A pixel in the CIE Lab color space.
pub struct CieLabPixel {
    pub l: f64,
    pub a: f64,
    pub b: f64,
}

impl CieLabPixel {
    pub fn new(l: f64, a: f64, b: f64) -> Self {
        CieLabPixel { l, a, b }
    }

    fn electro_optical_transfer(color_value: f64) -> f64 {
        if color_value <= 0.04045 {
            color_value / 12.92
        } else {
            ((color_value + 0.055) / 1.055).powf(2.4)
        }
    }

    fn linear_rgb_to_ciexyz(red: f64, green: f64, blue: f64) -> (f64, f64, f64) {
        let rgb_to_ciexyz_nd = arr2(&RGB_TO_CIEXYZ_MATRIX);
        let linear_rgb = arr2(&[[red], [green], [blue]]);
        let ciexyz = rgb_to_ciexyz_nd.dot(&linear_rgb);
        // Scale by 100 to match the reference white point scale (0-100 range)
        (
            ciexyz[[0, 0]] * 100.0,
            ciexyz[[1, 0]] * 100.0,
            ciexyz[[2, 0]] * 100.0,
        )
    }

    fn nonlinear_transformation_ciexyz_to_cielab(t: f64) -> f64 {
        if t > 0.008856 {
            t.cbrt()
        } else {
            7.787 * t + (16.0 / 116.0)
        }
    }

    fn ciexyz_to_cielab(x: f64, y: f64, z: f64) -> (f64, f64, f64) {
        // Normalize within the XYZ color system. This is essentially saying "this is what pure white looks like under
        // my current lighting (D65)" and get the ratios of our inputs to this reference point.
        let tx = x / CIEXYZ_REFERENCE_POINT_D65.0;
        let ty = y / CIEXYZ_REFERENCE_POINT_D65.1;
        let tz = z / CIEXYZ_REFERENCE_POINT_D65.2;

        // Similar to sRGB -> CIEXYZ, we have a piecewise defined function that is linear near 0.
        let ftx = Self::nonlinear_transformation_ciexyz_to_cielab(tx);
        let fty = Self::nonlinear_transformation_ciexyz_to_cielab(ty);
        let ftz = Self::nonlinear_transformation_ciexyz_to_cielab(tz);
        let l = 116.0 * fty - 16.0;
        let a = 500.0 * (ftx - fty);
        let b = 200.0 * (fty - ftz);

        (l, a, b)
    }

    pub fn lightness_as_u8(&self) -> u8 {
        ((self.l / 100.0) * 255.0).round() as u8
    }
}

impl From<Rgba<u8>> for CieLabPixel {
    fn from(pixel_color: Rgba<u8>) -> Self {
        // Normalize to a floating point value between 0 and 1.
        // Note that alpha is ignored.
        // XXX_jwir3: For correctness with Vyšniauskas' algorithm, we probably should blend somehow, rather than just
        // discarding.
        let norm_red: f64 = pixel_color[0] as f64 / 255.0;
        let norm_green: f64 = pixel_color[1] as f64 / 255.0;
        let norm_blue: f64 = pixel_color[2] as f64 / 255.0;

        // Standard sRGB values are gamma-corrected, so we need to linearize them by applying inverse gamma correction.
        // Note: Gamma correction was specifically for old CRT monitors. They had a natural intensity response described
        // by $I = V^\gamma$, where $2.1 <= \gamma <= 2.9$, with an average of 2.5. 2.2 is specifically used in the sRGB
        // specification, but sRGB doesn't use a simple exponent here. Instead, it uses an exponent of 2.4 to avoid a
        // situation where there would be an infinite slope at $C_{linear}^{\frac{1}{2.4}}$ when $C_{linear}=0$.
        // Thus, there's a small _linear segment_ near the origin $C = 0$: $C_{srgb} \le 0.04045$. This is called the
        // "Electro-Optical Transfer Function" (EOTF).

        // TODO_jwir3: We need to fix this to ensure reversed alpha blending is accurate!
        // When we are reversing alpha blending to estimate if a pixel resulted from AA, the precision of the linear
        // segment is vital.Because many AA signatures (like those from FXAA or MSAA) manifest as subtle gradients in
        // the "near-black" range, using a simplified $\gamma = 2.2$ power law across the whole range will introduce a
        // systematic error of about 2-5% in the darks. This error could easily be mistaken for an alpha-blending
        // artifact, leading to a false positive in the intensity slope detector.

        // TODO_jwir3: If we end up processing high-resolution frames, we could increase performance by converting all
        //             pixels at once using either:
        //             - SIMD: These operations are highly parallelizable using AVX or NEON instructions
        //             - Look-Up Tables (LUTs): For 8-bit or 10-bit source images, a 1D LUT is often faster than
        //               calculating the power function $x^{2.4}$ in real-time.
        let linear_red = Self::electro_optical_transfer(norm_red);
        let linear_green = Self::electro_optical_transfer(norm_green);
        let linear_blue = Self::electro_optical_transfer(norm_blue);

        // Next, we need to convert to the CIEXYZ color space, a linear transformation of the RGB values.
        let ciexyz = Self::linear_rgb_to_ciexyz(linear_red, linear_green, linear_blue);
        let x = ciexyz.0;
        let y = ciexyz.1;
        let z = ciexyz.2;

        // Finally, we convert to the CIELAB color space, which is a perceptually uniform color space.
        let cielab = Self::ciexyz_to_cielab(x, y, z);
        let l = cielab.0;
        let a = cielab.1;
        let b = cielab.2;

        CieLabPixel::new(l, a, b)
    }
}

pub enum PixelType {
    Alpha(Rgba<u8>),
    Grayscale(Luma<u8>),
    Lab(CieLabPixel),
}

pub fn get_intensity_for(pixel: PixelType) -> i16 {
    match pixel {
        PixelType::Grayscale(val) => val.0[0] as i16,
        PixelType::Lab(val) => val.l.round() as i16,
        PixelType::Alpha(val) => val.0[3] as i16,
    }
}

// TODO_jwir3: We really need to take gamma into account. If we read from a PNG file, there will likely be an implied
//             gamma, but we _might_ be able to get it from the image metadata, as well. ("gAMA" metadata flag)
pub fn convert_rgba_to_rgb_by_alpha_blend_no_gamma(
    image: &RgbaImage,
    bg_color: Rgb<u8>,
) -> RgbImage {
    let mut rgb_image = RgbImage::new(image.width(), image.height());
    for j in 0..image.height() {
        for i in 0..image.width() {
            let pixel = image.get_pixel(i, j);
            let red = pixel.0[0];
            let green = pixel.0[1];
            let blue = pixel.0[2];
            let alpha = pixel.0[3] as f64 / 255.0;

            let bg_contrib_red = bg_color.0[0] as f64 * (1.0 - alpha);
            let bg_contrib_green = bg_color.0[1] as f64 * (1.0 - alpha);
            let bg_contrib_blue = bg_color.0[2] as f64 * (1.0 - alpha);

            let result_red: u8 = ((red as f64 * alpha) + bg_contrib_red).round() as u8;
            let result_green: u8 = ((green as f64 * alpha) + bg_contrib_green).round() as u8;
            let result_blue: u8 = ((blue as f64 * alpha) + bg_contrib_blue).round() as u8;

            rgb_image.put_pixel(i, j, Rgb([result_red, result_green, result_blue]));
        }
    }

    rgb_image
}

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use image::{Luma, Rgb, RgbImage, Rgba, RgbaImage};

    use crate::color::{
        CieLabPixel, PixelType, convert_rgba_to_rgb_by_alpha_blend_no_gamma, get_intensity_for,
    };
    #[test]
    pub fn test_cielabpixel_from_rgba() {
        let rgba_pixel: Rgba<u8> = Rgba([255, 0, 0, 255]);
        let cielab_pixel = CieLabPixel::from(rgba_pixel);

        assert_abs_diff_eq!(cielab_pixel.l, 53.24, epsilon = 0.01);
        assert_abs_diff_eq!(cielab_pixel.a, 80.09, epsilon = 0.01);
        assert_abs_diff_eq!(cielab_pixel.b, 67.20, epsilon = 0.01);
    }

    #[test]
    pub fn test_convert_rgba_to_rgb_by_alpha_blend_no_gamma() {
        let cols = 3;
        let rows = 3;

        let mut rgba_image = RgbaImage::new(rows, cols);
        rgba_image.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        rgba_image.put_pixel(0, 1, Rgba([255, 0, 0, 192]));
        rgba_image.put_pixel(0, 2, Rgba([255, 0, 0, 128]));

        rgba_image.put_pixel(1, 0, Rgba([255, 0, 0, 192]));
        rgba_image.put_pixel(1, 1, Rgba([255, 0, 0, 128]));
        rgba_image.put_pixel(1, 2, Rgba([255, 0, 0, 64]));

        rgba_image.put_pixel(2, 0, Rgba([255, 0, 0, 128]));
        rgba_image.put_pixel(2, 1, Rgba([255, 0, 0, 64]));
        rgba_image.put_pixel(2, 2, Rgba([255, 0, 0, 0]));

        let mut expected_image = RgbImage::new(rows, cols);
        expected_image.put_pixel(0, 0, Rgb([255, 0, 0]));
        expected_image.put_pixel(0, 1, Rgb([255, 63, 63]));
        expected_image.put_pixel(0, 2, Rgb([255, 127, 127]));

        expected_image.put_pixel(1, 0, Rgb([255, 63, 63]));
        expected_image.put_pixel(1, 1, Rgb([255, 127, 127]));
        expected_image.put_pixel(1, 2, Rgb([255, 191, 191]));

        expected_image.put_pixel(2, 0, Rgb([255, 127, 127]));
        expected_image.put_pixel(2, 1, Rgb([255, 191, 191]));
        expected_image.put_pixel(2, 2, Rgb([255, 255, 255]));

        let result_image =
            convert_rgba_to_rgb_by_alpha_blend_no_gamma(&rgba_image, Rgb([255, 255, 255]));
        for j in 0..cols {
            for i in 0..rows {
                assert_eq!(result_image.get_pixel(i, j), expected_image.get_pixel(i, j));
            }
        }
    }

    #[test]
    fn test_lightness_as_u8() {
        let rgba_pixel: Rgba<u8> = Rgba([255, 0, 0, 255]);
        let cielab_pixel = CieLabPixel::from(rgba_pixel);
        let cielab_pixel_black = CieLabPixel::new(0.0, 0.0, 0.0);
        let cielab_pixel_white = CieLabPixel::new(100.0, 32.4, 55.6);

        assert_eq!(cielab_pixel_black.lightness_as_u8(), 0);
        assert_eq!(cielab_pixel_white.lightness_as_u8(), 255);
        assert_eq!(cielab_pixel.lightness_as_u8(), 136);
    }

    #[test]
    pub fn test_get_intensity_for_grayscale_pixel() {
        let val = Luma::<u8>([60]);

        assert_eq!(60, get_intensity_for(PixelType::Grayscale(val)));
    }

    #[test]
    pub fn test_get_intensity_for_lab_pixel() {
        let val = CieLabPixel::new(60.0, 0.0, 0.0);

        assert_eq!(60, get_intensity_for(PixelType::Lab(val)));
    }

    #[test]
    pub fn test_get_intensity_for_alpha_pixel() {
        let val = Rgba::<u8>([3, 165, 38, 60]);

        assert_eq!(60, get_intensity_for(PixelType::Alpha(val)));
    }
}
