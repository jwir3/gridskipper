use clap::{Parser, Subcommand};
use gridskipper::{diff_images, detect_antialiased_pixels, GridSkipperError};

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
        path: String,
    },
}

fn main() {
    let args = Args::parse();

    let result = match args.command {
        Commands::Diff { left_side, right_side } => {
            handle_diff(&left_side, &right_side)
        },
        Commands::Detect { path } => {
            handle_detect(&path)
        },
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
            println!("Note: gridskipper diff functionality is not yet fully implemented. Please check back later.");
            Ok(())
        },
        Err(e) => Err(e),
    }
}

fn handle_detect(path: &str) -> Result<(), GridSkipperError> {
    match detect_antialiased_pixels(path) {
        Ok(result) => {
            println!("Antialiasing detection results for '{}':", path);
            println!("  Antialiased pixels: {}", result.antialiased_pixels);
            println!("  Antialiased percentage: {:.2}%", result.antialiased_percentage);
            println!("Note: gridskipper detect functionality is not yet fully implemented. Please check back later.");
            Ok(())
        },
        Err(e) => Err(e),
    }
}
