use clap::{Parser, command};

#[derive(Parser, Debug)]
#[command(author, version, about="A command line tool for determining differences between two images.", long_about = None)]
struct Args {
}

fn main() {
    let _args = Args::parse();
    println!("gridskipper is not yet implemented. Please check back later.");
}
