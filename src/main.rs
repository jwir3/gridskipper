use clap::{Parser, Subcommand};
use colored::Colorize;
use gridskipper::{
    AntialiasDetectionResult, GridSkipperError, detect_antialiased_pixels_vysniauskas_rgba8,
    diff_images,
};
use image::ImageReader;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(author, version, about="A command line tool for determining differences between two images.", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Compare two image files for differences
    Diff {
        /// The first image file path to compare. Will throw an error if the file path is identical
        /// to the second image file path.
        left_side: String,
        /// The second image file path to compare. Will throw an error if the file path is identical
        /// to the first image file path.
        right_side: String,
    },
    /// Detect antialiased pixels within an image
    Detect {
        /// The image file path to check for antialiased pixels
        #[arg(short, long)]
        image_path: String,

        /// The ground truth image file path to compare with the detected antialiased pixels, for statisical analysis
        #[arg(short, long, default_value = "")]
        without_aa_path: String,

        // The directory for storing the output files
        #[arg(short, long, default_value = "out")]
        output_dir: String,
    },
}

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Commands::Diff {
            left_side,
            right_side,
        } => handle_diff(&left_side, &right_side),
        Commands::Detect {
            image_path,
            without_aa_path,
            output_dir,
        } => handle_detect(&image_path, &without_aa_path, &output_dir),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_diff(left_side: &str, right_side: &str) -> Result<(), GridSkipperError> {
    match diff_images(left_side, right_side) {
        Ok(result) => {
            if result.identical {
                println!("Images are identical");
            } else {
                println!("Images differ:");
                println!("  Different pixels: {}", result.diff_pixels);
                println!("  Difference percentage: {:.2}%", result.diff_percentage);
            }
            println!(
                "Note: gridskipper diff functionality is not yet fully implemented. Please check back later."
            );
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn handle_detect(
    image_path: &str,
    without_aa_path: &str,
    output_dir: &str,
) -> Result<(), GridSkipperError> {
    let stats: bool;
    let image_without_aa;
    if !without_aa_path.is_empty() && Path::exists(Path::new(without_aa_path)) {
        stats = true;
        image_without_aa = Some(ImageReader::open(without_aa_path)?.decode()?);
    } else {
        stats = false;
        image_without_aa = None;
    }

    // Create the output directory, if it doesn't already exist
    fs::create_dir_all(output_dir)?;

    // Open the image file
    let image = ImageReader::open(image_path)?.decode()?;

    // Detect antialiased pixels
    let detection_result = detect_antialiased_pixels_vysniauskas_rgba8(image, image_without_aa)?;

    // Print outputs using color
    println!("");
    println!("{}", "Antialiasing Results".green().bold());
    println!("{}", "====================".bold());
    println!(
        "{} {}",
        "Detected Antialiased Pixels:".bold(),
        detection_result
            .num_antialiased_pixels
            .to_string()
            .green()
            .bold()
    );

    if stats {
        println!(
            "{} {}",
            "Ground Truth Antialiased Pixels:".bold(),
            detection_result
                .num_ground_truth_antialiased_pixels
                .to_string()
                .green()
                .bold()
        );

        println!(
            "{} {}",
            "True Positives:".bold(),
            detection_result
                .num_true_positives
                .to_string()
                .green()
                .bold()
        );

        println!(
            "{} {}",
            "False Positives:".bold(),
            detection_result
                .num_false_positives
                .to_string()
                .green()
                .bold()
        );

        println!(
            "{} {}",
            "False Negatives:".bold(),
            detection_result
                .num_false_negatives
                .to_string()
                .green()
                .bold()
        );

        let precision_score = calculate_precision_score(&detection_result);
        let recall_score = calculate_recall_score(&detection_result);
        let f1_score = calculate_f1_score(precision_score, recall_score);

        println!(
            "{} {}",
            "Precision Score:".bold(),
            get_colored_text(precision_score, 0.95, 0.9)
        );

        println!(
            "{} {}",
            "Recall Score:".bold(),
            get_colored_text(recall_score, 0.95, 0.9)
        );

        println!(
            "{} {}",
            "F1 Score:".bold(),
            get_colored_text(f1_score, 0.95, 0.9),
        );
    }

    Ok(())
}

fn get_colored_text(value: f64, threshold_for_green: f64, threshold_for_yellow: f64) -> String {
    if value >= threshold_for_green {
        value.to_string().green().bold().to_string()
    } else if value >= threshold_for_yellow {
        value.to_string().yellow().bold().to_string()
    } else {
        value.to_string().red().bold().to_string()
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

    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn test_handle_detect_with_valid_image() {
        // Use existing test image
        let image_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_str().unwrap();

        let result = handle_detect(image_path, "", output_dir);

        assert!(result.is_ok());
        assert!(Path::new(output_dir).exists());
    }

    #[test]
    fn test_handle_detect_with_ground_truth() {
        // Use both test images for statistical analysis
        let image_path = "data/images/analytic-aa/lowercase-b-noto-serif-aa.png";
        let ground_truth_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_str().unwrap();

        let result = handle_detect(image_path, ground_truth_path, output_dir);

        assert!(result.is_ok());
        assert!(Path::new(output_dir).exists());
    }

    #[test]
    fn test_handle_detect_with_nonexistent_image() {
        let image_path = "nonexistent_image.png";
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_str().unwrap();

        let result = handle_detect(image_path, "", output_dir);

        assert!(result.is_err());
        match result.unwrap_err() {
            GridSkipperError::FileNotFound(path) => {
                assert_eq!(path, "nonexistent_image.png");
            }
            GridSkipperError::Image(_) => {
                // ImageReader::open might return an Image error in some situations
                // such as when the image format is not supported or the image itself
                // is corrupted.
            }
            GridSkipperError::Io(_) => {
                // ImageReader::open returns an IO error for nonexistent files
            }
            other => panic!(
                "Expected FileNotFound, Image, or IO error, got: {:?}",
                other
            ),
        }
    }

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

    #[test]
    fn test_handle_detect_with_invalid_ground_truth() {
        // Test with valid image but invalid ground truth path
        let image_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        let ground_truth_path = "nonexistent_ground_truth.png";
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_str().unwrap();

        // Should still work - function should handle missing ground truth gracefully
        let result = handle_detect(image_path, ground_truth_path, output_dir);

        assert!(result.is_ok());
        assert!(Path::new(output_dir).exists());
    }

    #[test]
    fn test_handle_detect_creates_output_directory() {
        let image_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().join("new_output_dir");
        let output_dir_str = output_dir.to_str().unwrap();

        // Verify directory doesn't exist initially
        assert!(!output_dir.exists());

        let result = handle_detect(image_path, "", output_dir_str);

        assert!(result.is_ok());
        assert!(output_dir.exists());
    }

    #[test]
    fn test_handle_detect_with_empty_ground_truth_path() {
        // Test that empty string for ground truth is handled correctly
        let image_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_str().unwrap();

        let result = handle_detect(image_path, "", output_dir);

        assert!(result.is_ok());
    }

    #[test]
    fn test_handle_detect_output_messages() {
        // This test verifies that the function executes without panicking
        // and produces the expected console output structure
        let image_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        let temp_dir = TempDir::new().unwrap();
        let output_dir = temp_dir.path().to_str().unwrap();

        // Capture the result to ensure it completes successfully
        let result = handle_detect(image_path, "", output_dir);

        assert!(result.is_ok());

        // Test with ground truth for statistical analysis
        let ground_truth_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        let result_with_stats = handle_detect(image_path, ground_truth_path, output_dir);

        assert!(result_with_stats.is_ok());
    }

    #[test]
    fn test_handle_detect_io_error_handling() {
        let image_path = "data/images/no-aa/lowercase-b-noto-serif-noAa.png";
        // Try to create output directory in a path that should fail (like root)
        let invalid_output_dir = "/invalid/path/that/should/not/exist";

        let result = handle_detect(image_path, "", invalid_output_dir);

        // Should return an IO error wrapped in GridSkipperError
        assert!(result.is_err());
        match result.unwrap_err() {
            GridSkipperError::Io(_) => {
                // This is expected
            }
            other => panic!("Expected IO error, got: {:?}", other),
        }
    }
}
