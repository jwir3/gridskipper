//! User interface code for gridskipper console application.
use colored::Colorize;

/// Retrieve colored text based on a value and thresholds.
///
/// Parameters:
/// - `value`: The value to be colored.
/// - `threshold_for_green`: The threshold for green color.
/// - `threshold_for_yellow`: The threshold for yellow color.
///
/// Returns:
/// - A string with the colored text.
pub fn get_colored_text(value: f64, threshold_for_green: f64, threshold_for_yellow: f64) -> String {
    if value >= threshold_for_green {
        value.to_string().green().bold().to_string()
    } else if value >= threshold_for_yellow {
        value.to_string().yellow().bold().to_string()
    } else {
        value.to_string().red().bold().to_string()
    }
}
