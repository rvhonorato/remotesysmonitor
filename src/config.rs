use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the monitoring application.
///
/// Contains all the necessary settings to connect to and remotesysmonitor remote servers.
/// This configuration is typically loaded from a YAML file.
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// A list of servers to be monitored.
    pub servers: Vec<Server>,
}

/// Represents a single server to be monitored.
///
/// Includes connection details and checks to be performed on the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct Server {
    /// Human-readable name for the server.
    pub name: String,
    /// Hostname or IP address of the server.
    pub host: String,
    /// Port to connect to on the server.
    pub port: u16,
    /// Username for authentication.
    pub user: String,
    /// Path to the private key for SSH authentication.
    pub private_key: String,
    /// Optional list of checks to be performed on the server.
    /// Each check is identified by a unique name and its corresponding configuration.
    pub checks: Option<HashMap<String, Check>>,
}

/// Defines various checks to be performed on the servers.
///
/// This enum allows for different types of checks, each with their own set of parameters.
#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Check {
    /// Check for the response from a given set of URLs.
    Ping {
        /// URLs to be pinged.
        url: Vec<String>,
    },
    /// Check the temperature from a specified sensor.
    Temperature {
        /// Identifier for the temperature sensor.
        sensor: String,
    },
    /// RemoteSysMonitor the load average over a specified interval.
    Load {
        /// Time interval in seconds over which to calculate the load average.
        interval: u16,
    },
    /// Count the number of subfolders in a specified path.
    NumberOfSubfolders {
        /// Paths to check for subfolders.
        path: Vec<String>,
        /// Maximum number of subfolders allowed.
        /// If the number of subfolders exceeds this value, an alert is triggered.
        max_folders: i32,
    },
    /// Check the age of the files in a list against a maximum age.
    ListAge {
        /// Path to the directory containing the files to check.
        path: Vec<String>,
        /// Maximum allowed age for the files in minutes.
        maximum_age: u16,
    },
    /// Run a custom command on the server and check its output.
    CustomCommand {
        /// The command to be executed on the server.
        command: String,
    },
    // Check the age of the files in a list against a maximum age.
    ListOldDirectories {
        /// Path to the directory containing the directories to check.
        loc: String,
        /// Maximum allowed age for the files in minutes.
        cutoff: u16,
    },
}
/// Loads the application configuration from a YAML file.
///
/// This function reads the configuration from the specified YAML file, parses it into
/// a `Config` struct, and returns it. The function handles reading the file and parsing
/// the YAML content, encapsulating the configuration loading logic.
///
/// # Arguments
///
/// * `file_path` - A string slice that holds the path to the configuration file.
///
/// # Returns
///
/// This function returns a `Result<Config, Box<dyn std::error::Error>>`. On success, it
/// returns the `Config` object encapsulating the loaded configuration. On failure, it
/// returns an error boxed as `Box<dyn std::error::Error>`, which can result from issues
/// reading the file or parsing the YAML content.
///
/// # Examples
///
/// ```
/// let config_path = "config/settings.yaml";
/// match load_config(config_path) {
///     Ok(config) => println!("Configuration loaded successfully."),
///     Err(e) => eprintln!("Failed to load configuration: {}", e),
/// }
/// ```
///
/// # Errors
///
/// This function can return an error in the following cases:
///
/// - The specified file does not exist or cannot be accessed.
/// - The file's contents cannot be read.
/// - The YAML parsing fails due to invalid syntax or other parsing issues.
pub fn load_config(file_path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let config_str = std::fs::read_to_string(file_path).map_err(|e| {
        error!("Could not read configuration file {}: {}", file_path, e);
        Box::<dyn std::error::Error>::from(e)
    })?;
    let config: Config = serde_yaml::from_str(&config_str).map_err(|e| {
        error!("Could not unmarshal: {}", e);
        Box::<dyn std::error::Error>::from(e)
    })?;
    Ok(config)
}

#[cfg(test)]
mod tests {

    #[test]
    #[ignore] // TODO
    fn test_load_config() {}
}
