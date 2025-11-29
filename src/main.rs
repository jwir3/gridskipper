use clap::{Parser, Subcommand, CommandFactory, error::ErrorKind};

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
    /// Check if a single image file exists
    Detect {
        /// The image file path to check for existence
        path: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Commands::Diff { left_side, right_side } => {
            handle_diff(&left_side, &right_side);
        },
        Commands::Detect { path } => {
            handle_detect(&path);
        },
    }
}

fn handle_diff(left_side: &str, right_side: &str) {
    // Check if both file paths are the same
    if left_side.eq(right_side) {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::InvalidValue, "Error: Both file paths cannot be the same")
            .exit();
    }

    // Check to make sure the files both exist
    if !std::path::Path::new(left_side).exists() {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::InvalidValue, "Error: Left side file does not exist")
            .exit();
    }
    if !std::path::Path::new(right_side).exists() {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::InvalidValue, "Error: Right side file does not exist")
            .exit();
    }

    println!("gridskipper diff functionality is not yet implemented. Please check back later.");
}

fn handle_detect(path: &str) {
    // Check if the file exists
    if !std::path::Path::new(path).exists() {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::InvalidValue, "Error: File does not exist")
            .exit();
    }

    println!("gridskipper detect functionality is not yet implemented. Please check back later.");
}