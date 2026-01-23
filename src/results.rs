use crate::{base::ImageLocation, stats::RawAAStatistics};

/// Result of comparing two images
#[derive(Debug)]
pub struct DiffResult {
    /// Whether the images are identical
    pub identical: bool,
    /// Number of different pixels found
    pub diff_pixels: u32,
    /// Percentage of different pixels
    pub diff_percentage: f64,
}

/// Result of antialiasing detection
#[derive(Debug)]
pub struct AntialiasDetectionResult {
    /// Total number of pixels in the image that were processed by the algorithm.
    pub number_pixels: u64,

    /// Number of antialiased pixels detected by the algorithm
    /// we're currently running.
    pub num_antialiased_pixels: u64,

    /// A list of antialiased pixel locations
    pub antialiased_pixels: Vec<ImageLocation>,

    /// Statistics collected about the antialiasing detection algorithm, if they can be computed. They are only able to
    /// be computed if a raw image (without antialiasing) was provided.
    pub statistics: Option<RawAAStatistics>,
}

impl Default for AntialiasDetectionResult {
    fn default() -> Self {
        Self {
            num_antialiased_pixels: 0,
            statistics: None,
            antialiased_pixels: Vec::new(),
            number_pixels: 0,
        }
    }
}
