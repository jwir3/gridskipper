use clap::{Parser, CommandFactory, error::ErrorKind};

#[derive(Parser, Debug)]
#[command(author, version, about="A command line tool for determining differences between two images.", long_about = None)]
struct Args {
    /// The first image file path to compare. Will throw an error if the file path is identical
    /// to the second image file path.
    left_side: String,
    /// The second image file path to compare. Will throw an error if the file path is identical
    /// to the first image file path.
    right_side: String,
}

fn main() {
    let args = Args::parse();
    if args.left_side.eq(&args.right_side) {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::InvalidValue, "Error: Both file paths cannot be the same")
            .exit();
    }

    // Check to make sure the files both exist.
    if !std::path::Path::new(&args.left_side).exists() {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::InvalidValue, "Error: Left side file does not exist")
            .exit();
    }
    if !std::path::Path::new(&args.right_side).exists() {
        let mut cmd = Args::command();
        cmd.error(ErrorKind::InvalidValue, "Error: Right side file does not exist")
            .exit();
    }

    println!("gridskipper is not yet implemented. Please check back later.");
}
