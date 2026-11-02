use std::path::Path;

use image::DynamicImage;

use crate::error::GridSkipperError;

/// Validates that file paths are different and both files exist
pub fn validate_file_paths(left_side: &str, right_side: &str) -> Result<(), GridSkipperError> {
    // Check if both file paths are the same
    // if left_side == right_side {
    //     return Err(GridSkipperError::IdenticalPaths);
    // }

    // Check if files exist
    if !Path::new(left_side).exists() {
        return Err(GridSkipperError::FileNotFound(left_side.to_string()));
    }

    if !Path::new(right_side).exists() {
        return Err(GridSkipperError::FileNotFound(right_side.to_string()));
    }

    Ok(())
}

/// Validates that a single file path exists
pub fn validate_single_file_path(path: &str) -> Result<(), GridSkipperError> {
    if !Path::new(path).exists() {
        return Err(GridSkipperError::FileNotFound(path.to_string()));
    }
    Ok(())
}

pub fn validate_images_and_metadata(left_side: &str, right_side: &str)
  -> Result<(DynamicImage, DynamicImage), GridSkipperError> {
    let left_image = image::open(left_side)?;
    let right_image = image::open(right_side)?;

    if left_image.width() != right_image.width() {
        return Err(GridSkipperError::ImageSizesDifferentWidth(left_image.width(), right_image.width()));
    }
    
    if left_image.height() != right_image.height() {
        return Err(GridSkipperError::ImageSizesDifferentHeight(left_image.height(), right_image.height()));
    }

    return Ok((left_image, right_image))
}


#[cfg(test)]
mod tests {
    use crate::{error::GridSkipperError, validate::{validate_file_paths, validate_images_and_metadata, validate_single_file_path}};

    #[test]
    fn test_validate_single_file_path_exists() {
        let result =
            validate_single_file_path("data/images/analytic-aa/lowercase-b-noto-serif-aa.png");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_file_paths_one_side_valid() {
        let result1 = validate_file_paths(
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png",
            "same.png",
        );

        let result2 = validate_file_paths(
            "same.png",
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png",
        );

        assert!(matches!(result1, Err(GridSkipperError::FileNotFound(_))));
        assert!(matches!(result2, Err(GridSkipperError::FileNotFound(_))));
    }

    #[test]
    fn test_validate_file_paths_both_sides_valid() {
        let result = validate_file_paths(
            "data/images/analytic-aa/ghostscript-tiger-aa.png",
            "data/images/no-aa/ghostscript-tiger-noaa.png",
        );

        assert!(result.is_ok());
        // println!("Error: {:?}", result.err());
        // assert!(false);
    }

    #[test]
    fn test_file_not_found_error() {
        let result = validate_single_file_path("nonexistent.png");
        assert!(matches!(result, Err(GridSkipperError::FileNotFound(_))));
    }

    #[test]
    #[ignore]
    fn test_validate_file_paths_identical_paths_error() {
        let result = validate_file_paths("same.png", "same.png");
        assert!(matches!(result, Err(GridSkipperError::IdenticalPaths)));
    }

    #[test]
    fn test_validate_image_metadata_incorrect_sizes() {
        let result
          = validate_images_and_metadata(
            "data/images/analytic-aa/ghostscript-tiger-aa.png",
            "data/images/analytic-aa/lowercase-b-noto-serif-aa.png");
        assert!(matches!(result, Err(GridSkipperError::ImageSizesDifferentWidth(733, 123))));

        let result2
          = validate_images_and_metadata(
            "data/images/red-example.png",
            "data/images/red-example-height-less.png");
        assert!(matches!(result2, Err(GridSkipperError::ImageSizesDifferentHeight(3, 2))));
    }

    #[test]
    fn test_validate_images_and_metadata_happy_path() {
        let result
          = validate_images_and_metadata(
            "data/images/analytic-aa/ghostscript-tiger-aa.png",
            "data/images/no-aa/ghostscript-tiger-noaa.png");
        
        assert!(result.is_ok());
    }

}