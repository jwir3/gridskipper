use clap::{Parser};

#[derive(Parser, Debug)]
#[command(author, version, about="A data generation tool for gridskipper", long_about = None)]
struct Args {
    /// Verbose output (-v for WARN, -vv for INFO, -vvv for DEBUG)
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[arg(short = 'o', long = "output", default_value = "output.png")]
    pub output: Option<String>,
}

pub fn main() {
    let args = Args::parse();

    // This creates a huge PNG file with random colors in it for each pixel.
    // This is used to create a large file for benchmarking the performance of gridskipper.
    let max_dim =  1024u32 * 128u32;
    let random_image = image::ImageBuffer::from_fn(max_dim, max_dim, |_, _| {
        image::Rgb([
            rand::random::<u8>(),
            rand::random::<u8>(),
            rand::random::<u8>(),
        ])
    });
    random_image.save(args.output.unwrap_or_else(|| "output.png".into())).unwrap();
}