use crate::base::ImageLocation;

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
    /// Number of pixels in the image without antialiasing that are different in the image with antialiasing
    pub num_ground_truth_antialiased_pixels: u64,

    /// Number of antialiased pixels detected
    pub num_antialiased_pixels: u64,

    /// Number of true positives
    pub num_true_positives: u64,

    /// Number of false positives. This is the number of pixels that were identified as antialiased but are not antialiased
    /// in the ground truth image
    pub num_false_positives: u64,

    /// Number of false negatives. This is the number of pixels that are antialiased in the ground truth image but were not
    /// identified as antialiased
    pub num_false_negatives: u64,

    /// A list of antialiased pixel locations
    pub antialiased_pixels: Vec<ImageLocation>,
}

impl Default for AntialiasDetectionResult {
    fn default() -> Self {
        Self {
            num_ground_truth_antialiased_pixels: 0,
            num_antialiased_pixels: 0,
            num_true_positives: 0,
            num_false_positives: 0,
            num_false_negatives: 0,
            antialiased_pixels: Vec::new(),
        }
    }
}
