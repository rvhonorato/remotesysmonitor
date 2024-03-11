use crate::ssh;
use regex::Regex;
use ssh2::Session;

/// Executes a check to count the number of folders in specified paths on a remote server.
///
/// This function connects to a remote server via SSH and runs a command to count the number
/// of directories in the given paths. It formats the results into a string, where each line
/// corresponds to one of the input paths. Each line reports the number of folders found and
/// is prefixed with an emoji to visually indicate the result: a green check mark (✅) indicates
/// no folders were found, a red cross (❌) indicates folders were found, and "❌ Error:" is
/// prefixed if an error occurred during command execution.
///
/// The function leverages the `find` command on the remote server to count directories directly,
/// minimizing the overhead and potential for misinterpretation compared to listing and manually
/// counting entries.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session` for executing commands on the remote server.
/// * `server_name` - The name of the server where the check is performed. This is used for reporting
///   results and does not affect the execution of the SSH command.
/// * `paths` - A slice of `String` objects, each representing a path on the remote server to check
///   for the number of folders.
///
/// # Returns
///
/// Returns a `String` containing the results of the check for each path. Each line in the returned
/// string reports the number of folders found in the corresponding path, prefixed with an emoji to
/// visually indicate the presence of folders or an error. The lines are formatted as follows:
/// - "✅ No folders @ `server_name:path`" if no folders are found,
/// - "❌ 1 folder @ `server_name:path`" if exactly one folder is found,
/// - "❌ N folders @ `server_name:path`" for N > 1 folders found,
/// - "❌ Error: error_description" if an error occurs during the execution of the SSH command.
///
/// # Examples
///
/// ```rust
/// let session = // Assume `session` is an established SSH `Session`.
/// let server_name = "example_server";
/// let paths = vec![String::from("/path/to/directory1"), String::from("/path/to/directory2")];
/// let result = number_of_folders(&session, server_name, &paths);
/// println!("{}", result);
/// ```
///
/// This will execute the folder count check for each path specified in `paths` on `example_server`
/// and print the results, one per line, with appropriate emojis indicating the outcome.
///
/// # Note
///
/// The function assumes that `ssh::run_ssh_command` can successfully connect and execute commands
/// on the remote server. It handles command execution failures by including an error message in the
/// output string. This function does not catch panics from parsing the command output, which should
/// be considered when interpreting the results.
pub fn number_of_folders(
    sess: &Session,
    server_name: &str,
    paths: &[String],
    max_folders: &i32,
) -> String {
    paths
        .iter()
        .map(|path| {
            let command = format!("find {} -maxdepth 1 -type d | tail -n +2 | wc -l", path);
            ssh::run_ssh_command(sess, &command).map_or_else(
                |err| format!("❌ Error: {}", err),
                |output| {
                    let count: usize = output.trim().parse().unwrap_or(0);
                    match count {
                        0 => format!("✅ No folders @ `{}:{}`", server_name, path),
                        1 => format!("✅ {} folder @ `{}:{}`", count, server_name, path),
                        _ if count >= *max_folders as usize => {
                            format!("❌ {} folders @ `{}:{}`", count, server_name, path)
                        }
                        _ => format!("✅ {} folders @ `{}:{}`", count, server_name, path),
                    }
                },
            )
        })
        .collect::<Vec<String>>()
        .join("\n")
}

/// Retrieves and parses the load average from a remote server over SSH and formats the result.
///
/// This function executes the `uptime` command on a remote server via SSH to retrieve the system's
/// load averages. It then parses the output to extract the load average corresponding to a specified
/// interval (1, 5, or 15 minutes). The function formats the load average with an emoji indicating
/// whether the load is above a certain threshold (in this case, 50.0) and returns this as a string.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session` for executing commands on the remote server.
/// * `server_name` - The name of the server where the command is executed. This is used for formatting
///   the output string but does not influence the command execution.
/// * `interval` - A `u16` specifying the interval for the load average to retrieve. Valid values are 1, 5,
///   or 15, corresponding to the standard intervals provided by the `uptime` command for load averages.
///
/// # Returns
///
/// Returns a `String` formatted with an emoji and the load average for the specified interval. If the load
/// is greater than 50.0, a "❌" is prefixed, otherwise a "✅". If an error occurs during command execution
/// or parsing, an error message is returned.
///
/// # Errors
///
/// If the `uptime` command fails to execute or if the output cannot be parsed to extract the load average,
/// the function prints an error message to stderr and returns a string indicating the error.
///
/// # Examples
///
/// ```rust
/// let session = // Assume `session` is an established SSH `Session`.
/// let server_name = "example_server";
/// let interval = 5; // Specify the interval for load average.
/// let result = load(&session, server_name, interval);
/// println!("{}", result);
/// ```
///
/// This will print the load average for the past 5 minutes from `example_server`, formatted with
/// an emoji indicating if the load is above 50.0.
///
/// # Notes
///
/// - The function assumes that the `ssh::run_ssh_command` function is available and correctly set up
///   to execute SSH commands.
/// - The choice of 50.0 as the threshold for determining high load is arbitrary and may not be suitable
///   for all systems. Consider adjusting this threshold based on your system's capacity and typical loads.
/// - The function currently only supports the fixed intervals of 1, 5, or 15 minutes, as these are the
///   standard intervals reported by the `uptime` command.
pub fn load(sess: &Session, server_name: &str, interval: u16) -> String {
    let output = match ssh::run_ssh_command(sess, "uptime") {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error: {}", e);
            return "".to_string();
        }
    };

    let load_parts = output.split("load average:").nth(1);
    let load = if let Some(parts) = load_parts {
        parts
            .split(',')
            .nth(match interval {
                1 => 0,
                5 => 1,
                15 => 2,
                _ => 0,
            })
            .and_then(|s| s.trim().parse::<f64>().ok())
    } else {
        None
    };

    match load {
        Some(load) => {
            let emoji = if load > 50.0 { "❌" } else { "✅" };
            format!(
                "{} load {:.2} ({}min) @ {}",
                emoji, load, interval, server_name
            )
        }
        None => "❌ Error: Could not parse load average".to_string(),
    }
}

/// Performs HTTP GET requests to a list of URLs constructed from a specified host and path segments.
///
/// This function iterates over a slice of URL path segments, appends each segment to the given host
/// to form complete URLs, and then performs an HTTP GET request to each URL. The function collects
/// the results of these requests into a single `String`, where each line represents the outcome of a
/// request to a specific URL. Successful requests are noted with a "✅", while failures due to
/// either network errors or non-success HTTP status codes are marked with a "❌".
///
/// # Arguments
///
/// * `host` - A string slice representing the base host to which the URL path segments will be appended.
/// * `urls` - A slice of `String` objects, each representing a path segment to be appended to the host
///   to form complete URLs for the GET requests.
///
/// # Returns
///
/// Returns a `String` where each line corresponds to the result of a request to one of the constructed URLs.
/// Successful requests are marked with "✅" followed by the URL. Unsuccessful requests are marked with "❌",
/// followed by the URL and either the HTTP status code (for responses that were received but indicated failure)
/// or the error message if the request failed to complete.
///
/// # Examples
///
/// ```rust
/// let host = "http://example.com";
/// let paths = vec![String::from("/api/health"), String::from("/api/status")];
/// let results = ping(host, &paths);
/// println!("{}", results);
/// ```
///
/// This might print something like:
///
/// ```text
/// ✅ http://example.com/api/health
/// ❌ http://example.com/api/status == `404 Not Found`
/// ```
///
/// # Note
///
/// The function uses `reqwest::blocking::get` to perform synchronous HTTP GET requests. This means
/// that each request will block the executing thread until a response is received or an error occurs.
/// As such, the total execution time of this function will be at least the sum of the response times
/// for all the requests, plus any additional overhead. For applications requiring non-blocking behavior
/// or high levels of concurrency, consider using asynchronous requests or a different approach.
///
/// Error handling in this function distinguishes between two types of failures: HTTP errors, where a
/// response was received but indicated an error through its status code, and network or other errors,
/// where the request could not be completed at all. In the former case, the specific status code is
/// included in the output; in the latter, the error message provided by the failure is included.
pub fn ping(host: &str, urls: &[String]) -> String {
    let mut results = String::new();
    urls.iter().for_each(|u| {
        let request_url = format!("{}{}", host, u);
        match reqwest::blocking::get(&request_url) {
            Ok(response) => {
                let status = response.status();
                if status.is_success() {
                    results.push_str(&format!("✅ {}\n", request_url));
                } else {
                    results.push_str(&format!("❌ {} == `{}`\n", request_url, status));
                }
            }
            Err(e) => results.push_str(&format!(
                "❌ {} is not accessible\n```{}```\n",
                request_url, e
            )),
        }
    });

    results.trim_end().to_string()
}

/// Reads and parses the temperature from a specified sensor file on a remote system via SSH.
///
/// This function executes a command to read the contents of a sensor file, where the temperature
/// data is expected to be in a specific format, typically containing a string like "t=12345" where
/// the digits represent the temperature in a unit such as millidegrees Celsius. The function then
/// parses this format to extract the temperature value, converts it to degrees Celsius, and returns
/// a formatted string indicating the temperature status. If the temperature is below 30°C, it is
/// considered normal and marked with a "✅". Otherwise, it is marked as potentially problematic with
/// a "❌". In case of errors at any step (e.g., command execution failure, regex compilation error,
/// or parsing failure), an appropriate error message is returned.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session`, used to execute the command on the remote system.
/// * `sensor` - The path to the sensor file on the remote system that contains the temperature data.
///
/// # Returns
///
/// Returns a `String` that indicates the temperature reading and its status:
/// - "✅ XX°C" if the temperature is successfully read and below 30°C.
/// - "❌ XX°C" if the temperature is successfully read but 30°C or above.
/// - "❌ Failed to parse temperature!" if the temperature value cannot be parsed from the file contents.
/// - "❌ Cannot read temperature!" if the sensor data does not match the expected format.
/// - Returns an empty string and prints an error message to stderr if there's an error executing the SSH command
///   or compiling the regular expression.
///
/// # Examples
///
/// Assuming the sensor file "/sys/class/thermal/thermal_zone0/temp" contains "t=29500":
///
/// ```rust
/// let session = // Assume `session` is an established SSH `Session`.
/// let sensor_path = "/sys/class/thermal/thermal_zone0/temp";
/// let temperature_status = temperature(&session, sensor_path);
/// println!("{}", temperature_status);
/// ```
///
/// This might print:
///
/// ```text
/// ✅ 29°C
/// ```
///
/// # Note
///
/// This function assumes that the sensor data format and the command to read it ("cat /path/to/sensor") are consistent
/// across the remote systems it is used with. Variations in sensor data format or the need to use a different
/// command to access it may require modifications to the function.
///
/// Error handling in this function provides basic feedback through returned error messages for specific failure
/// points. For production use, it may be beneficial to implement more detailed error reporting or logging,
/// especially for debugging issues with sensor data retrieval or parsing.
pub fn temperature(sess: &Session, sensor: &str) -> String {
    let command = format!("cat {}", sensor);
    let output = match ssh::run_ssh_command(sess, &command) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error: {}", e);
            return "".to_string();
        }
    };

    // Compile the regular expression to match the temperature value
    let re = match Regex::new(r"t=(\d+)") {
        Ok(re) => re,
        Err(e) => {
            eprintln!("Error: {}", e);
            return "".to_string();
        }
    };

    if let Some(caps) = re.captures(&output) {
        if let Some(matched) = caps.get(1) {
            let temperature = match matched.as_str().parse::<u32>() {
                Ok(temp) => temp / 1000, // Convert to degrees Celsius
                Err(_) => return "❌ Failed to parse temperature!".to_string(),
            };

            if temperature < 30 {
                return format!("✅ {}°C", temperature);
            }
            return format!("❌ {}°C", temperature);
        }
    }
    "❌ Cannot read temperature!".to_string()
}

/// Executes a custom command on a remote server via SSH and formats the output.
///
/// This function sends a specified command to be executed on a remote server through an established
/// SSH session. It formats the command and its output for readability, marking the command with a
/// warning emoji and encapsulating the command output in markdown code block syntax. If the command
/// execution fails, it logs the error and returns an empty string.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session`. This session is used to execute the
///   command on the remote server.
/// * `command` - A string slice that holds the command to be executed on the remote server.
///
/// # Returns
///
/// Returns a `String` that starts with a warning emoji and the command itself in backticks, followed
/// by the command output encapsulated in a markdown code block. If an error occurs during command
/// execution, an empty string is returned and the error is logged to standard error.
///
/// # Examples
///
/// ```rust
/// let session = // Assume `session` is an established SSH `Session`.
/// let command = "ls -la";
/// let result = custom_command(&session, command);
/// println!("{}", result);
/// ```
///
/// This might output something like:
///
/// ```text
/// ⚠️ `ls -la`
/// ```
/// ```text
/// total 12
/// drwxr-xr-x  2 user user 4096 Jul 21 12:00 .
/// drwxr-xr-x  4 user user 4096 Jul 20 14:43 ..
/// -rw-r--r--  1 user user   66 Jul 21 12:00 file.txt
/// ```
///
/// # Note
///
/// This function is designed to execute arbitrary commands on a remote server, which can be potentially
/// very dangerous if not used carefully. Ensure that the commands being executed are safe and that the
/// `command` argument comes from a trusted source to prevent security risks such as command injection.
///
/// The function uses `eprintln!` to log errors to standard error, which is suitable for command-line
/// applications but might need to be adapted for use in other contexts.
pub fn custom_command(sess: &Session, command: &str) -> String {
    let header = format!("⚠️ `{}`", command);
    let output = match ssh::run_ssh_command(sess, command) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error: {}", e);
            return "".to_string();
        }
    };

    let formatted_output = format!("```\n{}```", output);

    format!("{}\n{}", header, formatted_output)
}

/// Lists directories older than a specified number of days in a given location on a remote server.
///
/// This function executes a `find` command on a remote server via SSH to identify directories within
/// a specified location (`loc`) that are older than a given number of days (`cutoff`). It formats the
/// list of these directories into a human-readable string. If an error occurs during command execution,
/// an error message is logged, and an empty string is returned. If no directories meet the criteria,
/// a message indicating this is returned instead.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session` for executing commands on the remote server.
/// * `loc` - A string slice that specifies the location on the remote server to search for old directories.
/// * `cutoff` - The number of days used as the threshold for determining if a directory is considered "old".
///
/// # Returns
///
/// Returns a `String` that either:
/// - Lists the directories older than `cutoff` days in the specified location, formatted as a markdown
///   code block for readability, or
/// - Indicates that no directories older than `cutoff` days were found in the specified location, or
/// - Returns an empty string if an error occurred during command execution.
///
/// # Examples
///
/// ```rust
/// let session = // Assume `session` is an established SSH `Session`.
/// let location = "/var/log";
/// let days_old = 30;
/// let result = list_old_directories(&session, location, days_old);
/// println!("{}", result);
/// ```
///
/// This might output something like:
///
/// ```text
/// ❌ Directories older than 30 days:
/// ```
/// ```text
/// /var/log/old_logs
/// /var/log/archive
/// ```
/// Or, if no directories meet the criteria:
///
/// ```text
/// ✅ No directories older than 30 days in `/var/log`
/// ```
///
/// # Note
///
/// This function relies on the `find` command's `-mtime` option to determine the age of directories,
/// which is based on the time of the last modification to the directory's contents. This approach focuses
/// on when files within the directory were last added, removed, or renamed, rather than when their metadata
/// was last changed. Ensure that the remote server's environment and filesystem support the commands and
/// options used.
///
/// Error handling in this function logs command execution errors to standard error and returns an
/// empty string. This approach is suitable for command-line applications but may need adjustment for
/// use in other contexts where error logging or handling might be implemented differently.
pub fn list_old_directories(sess: &Session, loc: &str, cutoff: u16) -> String {
    let command = format!("find {} -maxdepth 1 -type d -mtime +{}", loc, cutoff);
    let output = match ssh::run_ssh_command(sess, &command) {
        Ok(output) => output,
        Err(e) => {
            eprintln!("Error: {}", e);
            return "".to_string();
        }
    };

    let files: Vec<&str> = output.split('\n').filter(|line| !line.is_empty()).collect();

    if files.is_empty() {
        return format!("✅ No directories older than {} days in `{}`", cutoff, loc);
    }

    let mut result = format!("❌ Directories older than {} days:", cutoff);
    result.push_str("\n```");
    for file in files {
        result.push('\n');
        result.push_str(file);
    }
    result.push_str("```");

    result
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
