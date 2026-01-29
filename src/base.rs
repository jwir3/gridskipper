//! Basic data structurs for Gridskipper.

/// A structure representing the location, in x and y coordinates, of a pixel in an image.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ImageLocation {
    pub x: u32,
    pub y: u32,
}

/// A structure indicating the indices of neighboring pixels in an image.
pub struct PixelNeighbors {
    pub left: u32,
    pub right: u32,
    pub top: u32,
    pub bottom: u32,
}
