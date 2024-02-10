use crate::ssh;
use regex::Regex;
use reqwest::blocking::get;
use reqwest::StatusCode;
use ssh2::Session;
use std::vec;

/// Executes a check to count the number of folders in specified paths on a remote server.
///
/// This function connects to a remote server via SSH and runs a command to list the contents
/// of given directories. It counts the number of entries in each directory, interpreting these
/// as folders. Based on the count, it annotates each result with an emoji to indicate if the
/// number of folders exceeds a threshold.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session` for executing commands on the remote server.
/// * `server_name` - The name of the server where the check is performed. Used for reporting results.
/// * `path` - A slice of `String` objects, each representing a path on the remote server to check for the number of folders.
///
/// # Returns
///
/// Returns a `String` containing the results of the check for each path. Each line in the returned
/// string reports the number of folders found in the corresponding path, along with an emoji to
/// visually indicate whether the count exceeds a specified threshold (e.g., more than 10 folders
/// results in a "❌", while 10 or fewer result in a "✅").
///
/// # Panics
///
/// This function panics if it fails to execute the SSH command on the remote server.
///
/// # Examples
///
/// ```
/// let session = // Assume `session` is an established SSH `Session`.
/// let server_name = "example_server";
/// let paths = vec![String::from("/path/to/directory1"), String::from("/path/to/directory2")];
/// let result = number_of_folders(&session, server_name, &paths);
/// println!("{}", result);
/// ```
///
/// # Note
///
/// The function assumes that the `ssh::run_ssh_command` successfully connects and executes
/// commands on the remote server. Error handling for SSH connection issues or command execution
/// failures is currently not implemented, leading to potential panics.
pub fn number_of_folders(sess: &Session, server_name: &str, path: &[String]) -> String {
    let mut result = vec![];
    path.iter().for_each(|p| {
        let command = format!("ls -l {}", p);
        let output = ssh::run_ssh_command(sess, command.as_str()).unwrap();
        let count = output.split('\n').count() - 1;

        // Use a different emoji depending on the number of subfolders, if its larger than 10 use a red cross else use a green tick
        let emoji = if count > 100 { "❌" } else { "✅" };

        result.push(format!(
            "{} {} folders @ `{}:{}`",
            emoji, count, server_name, p
        ));
    });

    // Join the results with a newline
    result.join("\n")
}

/// Checks the system load average on a remote server for a specified interval.
///
/// This function connects to a remote server via SSH and retrieves the system's load average using the `uptime` command.
/// It parses the output to extract the load average for a specified interval (1, 5, or 15 minutes) and annotates the
/// result with an emoji based on the load value: a green check mark (✅) if the load is 50 or below, and a red cross (❌)
/// if above 50.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session` for executing the command on the remote server.
/// * `server_name` - The name of the server, used for reporting in the result.
/// * `interval` - The time interval for the load average, in minutes. Valid values are 1, 5, or 15.
///
/// # Returns
///
/// Returns a `String` that includes the emoji annotation, the load average to two decimal places, the interval,
/// and the server name.
///
/// # Errors
///
/// If the specified interval is not one of the valid values (1, 5, or 15), the function prints an error message
/// and returns an empty string. It panics if it fails to execute the SSH command, parse the output, or if the load
/// average cannot be parsed into a floating-point number.
///
/// # Examples
///
/// ```
/// let session = // Assume `session` is an established SSH `Session`.
/// let server_name = "example_server";
/// let interval = 5; // Valid intervals: 1, 5, 15
/// let result = load(&session, server_name, interval);
/// println!("{}", result);
/// ```
///
/// # Note
///
/// The function assumes the `ssh::run_ssh_command` method successfully connects and executes commands on the remote server.
/// Ensure error handling for SSH session establishment and command execution is properly managed outside this function.
pub fn load(sess: &Session, server_name: &str, interval: u16) -> String {
    // Check if the interval is 1, 5 or 15
    if ![1, 5, 15].contains(&interval) {
        eprintln!("Interval must be 1, 5 or 15");
        return "".to_string();
    }

    // We will select a different index depending on the interval
    let index = match interval {
        1 => 0,
        5 => 1,
        15 => 2,
        _ => 0,
    };
    let output = ssh::run_ssh_command(sess, "uptime").unwrap();
    let load = output.split("load average:").collect::<Vec<&str>>()[1]
        .split(',')
        .collect::<Vec<&str>>()[index]
        .trim()
        .parse::<f64>()
        .unwrap();

    let emoji = if load > 50.0 { "❌" } else { "✅" };
    format!(
        "{} load {:.2} ({}min) @ {}",
        emoji, load, interval, server_name
    )
}

/// Performs a ping check on a list of URLs for a specified host.
///
/// This function attempts to make HTTP GET requests to each URL concatenated with the host. It evaluates
/// the response status for each URL. If a request successfully returns a status code of 200 (OK), the URL
/// is considered reachable, and a positive result is recorded. Otherwise, the URL is marked as unreachable.
///
/// # Arguments
///
/// * `host` - The hostname or IP address where the URLs will be checked. This host is prefixed to each URL in the `url` slice.
/// * `url` - A slice of `String` objects, each representing a partial URL to be checked. These URLs are appended to `host` to form complete URLs.
///
/// # Returns
///
/// Returns a `String` that concatenates the results of each ping check, separated by newlines. Each result includes an emoji:
/// - "✅" indicates the URL is reachable.
/// - "❌" indicates the URL is unreachable, followed by the error message.
///
/// # Examples
///
/// ```
/// let host = "example.com";
/// let urls = vec![String::from("/path1"), String::from("/path2")];
/// let result = ping(host, &urls);
/// println!("{}", result);
/// ```
///
/// # Note
///
/// This function uses synchronous HTTP GET requests to perform the ping checks, which may block the executing thread.
/// Handling each URL's response status, it only considers a 200 (OK) status as a successful ping, logging errors for any other scenarios.
pub fn ping(host: &str, url: &[String]) -> String {
    let mut result = vec![];
    url.iter().for_each(|u| {
        let loc = format!("{}{}", host, u);
        match get(loc.as_str()) {
            Ok(response) => {
                if response.status() == StatusCode::OK {
                    result.push(format!("✅ {}", loc));
                }
            }
            Err(e) => result.push(format!("❌ {} == `{}`", loc, e)),
        }
    });

    result.join("\n")
}

/// Retrieves and evaluates the temperature from a specified sensor on a remote server.
///
/// This function sends a command via SSH to read the sensor's data, expecting to find a temperature
/// value formatted as 't=XXXX', where 'XXXX' represents the temperature in millidegrees Celsius. It
/// parses this value, converts it to degrees Celsius, and evaluates whether the temperature is within
/// an acceptable range (<= 30°C considered normal).
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session` for executing the command on the remote server.
/// * `sensor` - The file path to the sensor data on the remote server. This path should contain temperature
///   data in a format that includes 't=XXXX' to denote the temperature.
///
/// # Returns
///
/// Returns a `String` that includes:
/// - An "✅" emoji and the temperature in Celsius if the reading is successful and the temperature is 30°C or below.
/// - A "❌" emoji and the temperature in Celsius if the reading is successful but the temperature is above 30°C.
/// - A "❌" emoji with a message indicating the temperature could not be read if there are issues parsing the sensor data.
///
/// # Panics
///
/// This function panics if it fails to execute the SSH command, compile the regular expression, or parse the temperature value.
///
/// # Examples
///
/// ```
/// let session = // Assume `session` is an established SSH `Session`.
/// let sensor_path = "/sys/class/thermal/thermal_zone0/temp";
/// let result = temperature(&session, sensor_path);
/// println!("{}", result);
/// ```
///
/// # Note
///
/// This function assumes that the sensor data is available at the specified path and in the expected format.
/// The regular expression and parsing logic are designed specifically for a temperature format of 't=XXXX'.
/// Ensure that the remote server and sensor path conform to these expectations to avoid errors or incorrect readings.
pub fn temperature(sess: &Session, sensor: &str) -> String {
    // let result = "".to_string();
    let command = format!("cat {}", sensor);
    let output = ssh::run_ssh_command(sess, command.as_str()).unwrap();
    let re = Regex::new(r"t=(\d+)").unwrap(); // Compile regex to match 't=' followed by one or more digits
    if let Some(caps) = re.captures(&output) {
        if let Some(matched) = caps.get(1) {
            // caps.get(1) gets the first capture group, which is the digits after 't='
            let temperature = matched.as_str();

            // Convert to u32 and divide by 1000 to get the temperature in degrees celsius
            let temperature = temperature.parse::<u32>().unwrap() / 1000;
            if temperature < 30 {
                return format!("✅ {}°C", temperature);
            } else {
                return format!("❌ {}°C", temperature);
            }
        }
    }
    "❌ Cannot read temperature!".to_string()
}

/// Executes a custom SSH command on a remote server and formats the output.
///
/// This function sends a specified command to be executed on a remote server via an established SSH session.
/// The output of the command is captured and formatted in Markdown code block style, prefixed with a warning emoji
/// and the command itself for clarity. This format is suitable for reporting or logging purposes where the distinction
/// between the command and its output needs to be clear.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session`. This session is used to execute the command on the remote server.
/// * `command` - The SSH command to be executed as a string slice. This command is sent as-is to the server.
///
/// # Returns
///
/// Returns a `String` that contains the formatted command and its output. The command is prefixed with a warning emoji
/// and wrapped in backticks. The output is enclosed in a Markdown code block, ensuring it is displayed correctly in
/// environments that support Markdown formatting (e.g., Slack, GitHub issues).
///
/// # Panics
///
/// This function panics if the SSH command execution fails, indicating an issue with the SSH session, the command's
/// validity, or the server's ability to execute the command.
///
/// # Examples
///
/// ```
/// let session = // Assume `session` is an established SSH `Session`.
/// let custom_cmd = "ls -l /home/user";
/// let result = custom_command(&session, custom_cmd);
/// println!("{}", result);
/// ```
///
/// # Note
///
/// This function is particularly useful for executing diagnostic or monitoring commands where the distinction between
/// the command issued and the output received is crucial. It's designed to integrate well with systems that support
/// Markdown formatting, making the output more readable and informative.
pub fn custom_command(sess: &Session, command: &str) -> String {
    let header = format!("⚠️ `{}`", command);
    let output = ssh::run_ssh_command(sess, command).unwrap();

    let output = format!("```\n{}```", output);

    let mut result = vec![header];
    result.push(output);
    result.join("\n")
}

/// Lists directories older than a specified number of days in a given location on a remote server.
///
/// This function constructs and executes a command via SSH to find directories that exceed the age threshold
/// specified by the `cutoff` parameter. It formats the output to clearly distinguish directories that are older
/// than the specified cutoff, making it useful for identifying outdated or stale directories for potential cleanup.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session` for executing the command on the remote server.
/// * `loc` - The location (directory path) on the remote server to search for old directories.
/// * `cutoff` - The age threshold in days. Directories older than this number of days will be listed.
///
/// # Returns
///
/// Returns a `String` that either:
/// - Lists the directories older than `cutoff` days, each directory on a new line, enclosed in a Markdown code block
///   and prefixed with a warning emoji, or
/// - Indicates that no directories older than `cutoff` days were found in the specified location, prefixed with a
///   checkmark emoji.
///
/// # Panics
///
/// This function panics if the SSH command execution fails. This could occur due to issues with the SSH session,
/// incorrect command syntax, or if the specified `loc` directory path is not accessible on the remote server.
///
/// # Examples
///
/// ```
/// let session = // Assume `session` is an established SSH `Session`.
/// let location = "/var/log";
/// let days_old = 30; // List directories older than 30 days
/// let result = list_old_directories(&session, location, days_old);
/// println!("{}", result);
/// ```
///
/// # Note
///
/// This function is particularly useful for system maintenance tasks, such as identifying old log directories or
/// other outdated content that may need to be reviewed or cleaned up. The output is formatted for readability,
/// especially in environments that support Markdown rendering.
pub fn list_old_directories(sess: &Session, loc: &str, cutoff: u16) -> String {
    let command = format!("find {} -maxdepth 1 -type d -ctime +{}", loc, cutoff);
    let output = ssh::run_ssh_command(sess, command.as_str()).unwrap();
    let files: Vec<&str> = output.split('\n').collect();
    let mut result = vec![];
    result.push(format!("❌ Directories older than {} days:", cutoff));
    result.push("```".to_string());
    for file in files {
        if !file.is_empty() {
            result.push(file.to_string());
        }
    }
    result.push("```".to_string());

    if result.len() > 3 {
        return result.join("\n");
    }
    format!("✅ directories older than {} days in `{}`", cutoff, loc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // TODO
    fn test_number_of_folder() {}

    #[test]
    #[ignore] // TODO
    fn test_load() {}

    #[test]
    fn test_ping() {
        let mut server = mockito::Server::new();

        let host = server.url();
        let _m = server.mock("GET", "/test").with_status(200).create();

        let urls = vec![String::from("/test")];

        let result = ping(host.as_str(), &urls);
        // Check if there ✅ is in the result
        assert!(result.contains('✅'));

        let result = ping("does-not-exist", &urls);
        // Check if there ❌ is in the result
        println!("{}", result);
        assert!(result.contains('❌'));
    }

    #[test]
    fn test_ping_error() {
        let host = "localhost";
        let urls = vec![String::from("/test")];

        let result = ping(host, &urls);
        // Check if there ❌ is in the result
        assert!(result.contains('❌'));
    }

    #[test]
    #[ignore] // TODO
    fn test_temperature() {}

    #[test]
    #[ignore] // TODO
    fn test_custom_command() {}

    #[test]
    #[ignore] // TODO
    fn test_list_old_directories() {}
}
