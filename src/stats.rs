//! Statistics module for gridskipper console application.

use crate::results::AntialiasDetectionResult;

/// Calculate the recall score for a given `AntialiasDetectionResult`.
///
/// The "recall" score is the percentage of true positives out of both true positives and false negatives. It answers
/// the question "What percentage of positive cases were correctly identified?"
///
/// Parameters:
/// - `detection_result`: The result of the antialias detection.
///
/// Returns:
/// - The recall score as a floating-point number.
pub fn calculate_recall_score(detection_result: &AntialiasDetectionResult) -> f64 {
    let true_positives = detection_result.num_true_positives as f64;
    let false_negatives = detection_result.num_false_negatives as f64;

    if true_positives == 0.0 && false_negatives == 0.0 {
        0.0
    } else {
        true_positives / (true_positives + false_negatives)
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
/// - The precision score as a floating-point number.
pub fn calculate_precision_score(detection_result: &AntialiasDetectionResult) -> f64 {
    let true_positives = detection_result.num_true_positives as f64;
    let false_positives = detection_result.num_false_positives as f64;

    if true_positives == 0.0 && false_positives == 0.0 {
        0.0
    } else {
        true_positives / (true_positives + false_positives)
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
pub fn calculate_f1_score(precision_score: f64, recall_score: f64) -> f64 {
    if precision_score == 0.0 && recall_score == 0.0 {
        0.0
    } else {
        2.0 * (precision_score * recall_score) / (precision_score + recall_score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_precision_score() {
        let result = AntialiasDetectionResult {
            num_ground_truth_antialiased_pixels: 100,
            num_antialiased_pixels: 95,
            num_false_negatives: 5,
            num_false_positives: 5,
            num_true_positives: 86,
            antialiased_pixels: Vec::new(),
        };

        assert_eq!(calculate_precision_score(&result), 86.0 / (86.0 + 5.0));
    }

    #[test]
    fn test_calculate_recall_score() {
        let result = AntialiasDetectionResult {
            num_ground_truth_antialiased_pixels: 762,
            num_antialiased_pixels: 638,
            num_false_negatives: 521,
            num_false_positives: 397,
            num_true_positives: 241,
            antialiased_pixels: Vec::new(),
        };

        assert_eq!(calculate_recall_score(&result), 0.31627296587926507);
    }
}
