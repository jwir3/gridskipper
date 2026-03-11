use std::fmt::Display;

use colored::Colorize;

use crate::stats::{calculate_f1_score, calculate_precision_score, calculate_recall_score};
use crate::ui::get_colored_text;
use crate::{base::ImageLocation, stats::RawAAStatistics};

/// Result of comparing two images
#[derive(Debug, PartialEq, Clone)]
pub struct DiffResult {
    /// Whether the images are identical
    pub identical: bool,
    /// Number of different pixels found
    pub diff_pixels: u32,
    /// Percentage of different pixels
    pub diff_percentage: f64,
}

impl Default for DiffResult {
    fn default() -> Self {
        Self {
            identical: true,
            diff_pixels: 0,
            diff_percentage: 0.0,
        }
    }
}

/// Result of antialiasing detection
#[derive(Debug, Default)]
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

impl AntialiasDetectionResult {
    pub fn is_antialiased(&self, x: u32, y: u32) -> bool {
        self.antialiased_pixels.contains(&ImageLocation { x, y })
    }
}

impl Display for AntialiasDetectionResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print outputs using color

        writeln!(f, "{}", "Antialiasing Results".green().bold())?;
        writeln!(f, "{}", "====================".bold())?;
        writeln!(
            f,
            "{} {}",
            "Total Pixels:".bold(),
            self.number_pixels.to_string().bold()
        )?;

        writeln!(
            f,
            "{} {}",
            "Detected Antialiased Pixels:".bold(),
            self.num_antialiased_pixels.to_string().green().bold()
        )?;

        let stats_opt = self.statistics;
        if let Some(stats) = stats_opt {
            writeln!(
                f,
                "{} {}",
                "Ground Truth Antialiased Pixels:".bold(),
                stats
                    .num_ground_truth_antialiased_pixels
                    .to_string()
                    .green()
                    .bold()
            )?;

            writeln!(
                f,
                "{} {}",
                "True Positives:".bold(),
                stats.num_true_positives.to_string().green().bold()
            )?;

            writeln!(
                f,
                "{} {}",
                "False Positives:".bold(),
                stats.num_false_positives.to_string().green().bold()
            )?;

            writeln!(
                f,
                "{} {}",
                "False Negatives:".bold(),
                stats.num_false_negatives.to_string().green().bold()
            )?;

            let precision_score_opt = calculate_precision_score(self);
            if let Some(precision_score) = precision_score_opt {
                writeln!(
                    f,
                    "{} {}",
                    "Precision Score:".bold(),
                    get_colored_text(precision_score, 0.95, 0.9)
                )?;
            }

            let recall_score_opt = calculate_recall_score(self);
            if let Some(recall_score) = recall_score_opt {
                writeln!(
                    f,
                    "{} {}",
                    "Recall Score:".bold(),
                    get_colored_text(recall_score, 0.95, 0.9)
                )?;
            }

            let f1_score_opt = calculate_f1_score(precision_score_opt, recall_score_opt);
            if let Some(f1_score) = f1_score_opt {
                writeln!(
                    f,
                    "{} {}",
                    "F1 Score:".bold(),
                    get_colored_text(f1_score, 0.95, 0.9),
                )?;
            }
        }

        Ok(())
    }
}
