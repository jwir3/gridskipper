use image::Rgba;
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
            return color_value / 12.92;
        } else {
            return ((color_value + 0.055) / 1.055).powf(2.4);
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

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use image::Rgba;

    use super::CieLabPixel;

    #[test]
    pub fn test_cielabpixel_from_rgba() {
        let rgba_pixel: Rgba<u8> = Rgba([255, 0, 0, 255]);
        let cielab_pixel = CieLabPixel::from(rgba_pixel);

        assert_abs_diff_eq!(cielab_pixel.l, 53.24, epsilon = 0.01);
        assert_abs_diff_eq!(cielab_pixel.a, 80.09, epsilon = 0.01);
        assert_abs_diff_eq!(cielab_pixel.b, 67.20, epsilon = 0.01);
    }
}
