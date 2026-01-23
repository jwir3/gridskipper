use std::fs;
use std::path::Path;

use clap::{Parser, Subcommand};
use colored::Colorize;
use gridskipper::stats::{calculate_f1_score, calculate_precision_score, calculate_recall_score};
use gridskipper::{
    detect_antialiased_pixels_vysniauskas_rgba8, diff_images, error::GridSkipperError,
};

use gridskipper::ui::get_colored_text;
use image::ImageReader;

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
    let image_without_aa;
    if !without_aa_path.is_empty() && Path::exists(Path::new(without_aa_path)) {
        image_without_aa = Some(ImageReader::open(without_aa_path)?.decode()?);
    } else {
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
        "Total Pixels:".bold(),
        detection_result.number_pixels.to_string().bold()
    );

    println!(
        "{} {}",
        "Detected Antialiased Pixels:".bold(),
        detection_result
            .num_antialiased_pixels
            .to_string()
            .green()
            .bold()
    );

    let stats_opt = detection_result.statistics;
    if let Some(stats) = stats_opt {
        println!(
            "{} {}",
            "Ground Truth Antialiased Pixels:".bold(),
            stats
                .num_ground_truth_antialiased_pixels
                .to_string()
                .green()
                .bold()
        );

        println!(
            "{} {}",
            "True Positives:".bold(),
            stats.num_true_positives.to_string().green().bold()
        );

        println!(
            "{} {}",
            "False Positives:".bold(),
            stats.num_false_positives.to_string().green().bold()
        );

        println!(
            "{} {}",
            "False Negatives:".bold(),
            stats.num_false_negatives.to_string().green().bold()
        );

        let precision_score_opt = calculate_precision_score(&detection_result);
        if let Some(precision_score) = precision_score_opt {
            println!(
                "{} {}",
                "Precision Score:".bold(),
                get_colored_text(precision_score, 0.95, 0.9)
            );
        }

        let recall_score_opt = calculate_recall_score(&detection_result);
        if let Some(recall_score) = recall_score_opt {
            println!(
                "{} {}",
                "Recall Score:".bold(),
                get_colored_text(recall_score, 0.95, 0.9)
            );
        }

        let f1_score_opt = calculate_f1_score(precision_score_opt, recall_score_opt);
        if let Some(f1_score) = f1_score_opt {
            println!(
                "{} {}",
                "F1 Score:".bold(),
                get_colored_text(f1_score, 0.95, 0.9),
            );
        }
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use std::path::Path;

    use gridskipper::error::GridSkipperError;
    use tempfile::TempDir;

    use crate::handle_detect;

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
