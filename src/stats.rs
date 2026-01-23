//! Statistics module for gridskipper console application.

use crate::results::AntialiasDetectionResult;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct RawAAStatistics {
    /// Number of pixels in the image without antialiasing that are different in the image with antialiasing
    pub num_ground_truth_antialiased_pixels: u64,

    /// Number of true positives. These are the pixels that are detected by the algorithm as antialiased and also are
    /// antialiased in the ground truth image.
    pub num_true_positives: u64,

    /// Number of false positives. This is the number of pixels that were identified as antialiased by the algorithm,
    /// but are not actually antialiased in the ground truth image.
    pub num_false_positives: u64,

    /// Number of false negatives. This is the number of pixels that are antialiased in the ground truth image but were
    /// not identified as antialiased by the algorithm.
    pub num_false_negatives: u64,
}

impl Default for RawAAStatistics {
    fn default() -> Self {
        RawAAStatistics {
            num_ground_truth_antialiased_pixels: 0,
            num_true_positives: 0,
            num_false_positives: 0,
            num_false_negatives: 0,
        }
    }
}

/// Calculate the recall score for a given `AntialiasDetectionResult`.
///
/// The "recall" score is the percentage of true positives out of both true positives and false negatives. It answers
/// the question "What percentage of positive cases were correctly identified?"
///
/// Parameters:
/// - `detection_result`: The result of the antialias detection.
///
/// Returns:
/// - The recall score as a floating-point number, or `None` if the statistics are not available.
pub fn calculate_recall_score(detection_result: &AntialiasDetectionResult) -> Option<f64> {
    if detection_result.statistics.is_none() {
        return None;
    }

    let statistics = detection_result.statistics.as_ref().unwrap();

    let true_positives = statistics.num_true_positives as f64;
    let false_negatives = statistics.num_false_negatives as f64;

    if true_positives == 0.0 && false_negatives == 0.0 {
        Some(0.0)
    } else {
        Some(true_positives / (true_positives + false_negatives))
    }
}

/// Calculate the precision score for a given `AntialiasDetectionResult`.
///
/// The "precision" score is the percentage of true positives out of both true positives and false negatives. It answers
/// the question "Of all the antialiased pixes that exist in the image, how many were correctly identified?"
///
/// Parameters:
/// - `detection_result`: The result of the antialias detection.
///
/// Returns:
/// - The precision score as a floating-point number, or None if statistics are not available.
pub fn calculate_precision_score(detection_result: &AntialiasDetectionResult) -> Option<f64> {
    if detection_result.statistics.is_none() {
        return None;
    }

    let statistics = detection_result.statistics.as_ref().unwrap();

    let true_positives = statistics.num_true_positives as f64;
    let false_positives = statistics.num_false_positives as f64;

    if true_positives == 0.0 && false_positives == 0.0 {
        Some(0.0)
    } else {
        Some(true_positives / (true_positives + false_positives))
    }
}

/// Calculate the F1 score, given both the precision and recall scores. The F1 score is a scoring mechanism that
/// balances both precision and recall.
///
/// Parameters:
/// - `precision_score`: The precision score.
/// - `recall_score`: The recall score.
///
/// Returns:
/// - The F1 score as a floating-point number.
pub fn calculate_f1_score(
    precision_score_opt: Option<f64>,
    recall_score_opt: Option<f64>,
) -> Option<f64> {
    if let Some(precision_score) = precision_score_opt
        && let Some(recall_score) = recall_score_opt
    {
        if precision_score == 0.0 && recall_score == 0.0 {
            return Some(0.0);
        } else {
            return Some(2.0 * (precision_score * recall_score) / (precision_score + recall_score));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_precision_score() {
        let result = AntialiasDetectionResult {
            num_antialiased_pixels: 95,
            number_pixels: 100,
            statistics: Some(RawAAStatistics {
                num_ground_truth_antialiased_pixels: 100,
                num_false_negatives: 5,
                num_false_positives: 5,
                num_true_positives: 86,
            }),
            antialiased_pixels: Vec::new(),
        };

        assert_eq!(
            calculate_precision_score(&result),
            Some(86.0 / (86.0 + 5.0))
        );
    }

    #[test]
    fn test_calculate_recall_score() {
        let result = AntialiasDetectionResult {
            num_antialiased_pixels: 638,
            number_pixels: 762,
            statistics: Some(RawAAStatistics {
                num_ground_truth_antialiased_pixels: 762,
                num_false_negatives: 521,
                num_false_positives: 397,
                num_true_positives: 241,
            }),
            antialiased_pixels: Vec::new(),
        };

        assert_eq!(calculate_recall_score(&result), Some(0.31627296587926507));
    }
}
