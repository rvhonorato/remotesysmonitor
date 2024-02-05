use chrono::Local;
/// Generates a formatted timestamp string based on the current local time.
///
/// This function retrieves the current local time and formats it into a more readable string format,
/// typically used for logging or displaying timestamps in a user-friendly manner. The format used is
/// day/month(abbreviated)/year hour:minute timezone.
///
/// # Returns
///
/// Returns a `String` representing the current local time formatted as "dd/Mon/yy HH:MM TZ". For example,
/// "02/Feb/23 15:04 PST".
///
/// # Examples
///
/// ```
/// let timestamp = make_pretty_timestamp();
/// println!("Current time: {}", timestamp);
/// ```
///
/// # Note
///
/// This function relies on the `chrono` crate's `Local::now` to get the current time and its `format` method
/// to convert the time into a string. Ensure the `chrono` crate is included in your project's dependencies
/// to use this function.
pub fn make_pretty_timestamp() -> String {
    let now = Local::now();
    let formatted = now.format("%d/%b/%y %H:%M %Z").to_string();
    formatted
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_pretty_timestamp_not_empty() {
        let timestamp = make_pretty_timestamp();
        assert!(!timestamp.is_empty(), "Timestamp should not be empty");
    }

    #[test]
    fn test_make_pretty_timestamp_format() {
        let timestamp = make_pretty_timestamp();
        // This is a simplistic check and might need to be adjusted based on the format you expect
        assert!(
            timestamp.contains('/'),
            "Timestamp should contain '/' indicating date format"
        );
        assert!(
            timestamp.contains(':'),
            "Timestamp should contain ':' indicating time format"
        );
    }
}
