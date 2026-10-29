use image::Rgba;

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