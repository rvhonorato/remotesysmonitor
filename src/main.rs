//! # Remote System Monitoring Application
//!
//! `RemoteSysMonitor` is a comprehensive tool designed for monitoring remote servers. It executes specified commands on remote servers via SSH and can forward the results to Slack for notifications. The application supports a variety of checks, such as ping, system load, temperature readings, and the execution of custom scripts. Configuration is managed through a YAML file, allowing for easy setup and customization.
//!
//! ## Usage
//!
//! To utilize `RemoteSysMonitor`, follow these steps:
//! 1. Prepare a `config.yaml` file according to your monitoring requirements, detailing the servers to be monitored along with the specific checks for each.
//! 2. Set the `SLACK_HOOK_URL` environment variable to your Slack webhook URL to enable Slack notifications.
//! 3. Launch the application, providing the path to your configuration file as the argument.
//!
//! Example command to run the application:
//! ```no_run
//! cargo run -- /path/to/your/config.yaml
//! ```
//!
//! ## Key Features
//!
//! - **Server Monitoring**: Facilitates monitoring of multiple servers through SSH.
//! - **Diverse Checks**: Supports various checks, including ping, system load, temperature readings, and execution of custom scripts.
//! - **Slack Integration**: Enables direct reporting of monitoring results to a specified Slack channel for real-time alerts.
//!
//! ## Configuration Guide
//!
//! `RemoteSysMonitor` relies on a YAML file for configuration, allowing you to specify the servers to monitor and the checks to perform on each. Below is an example of how the configuration file might look:
//!
//! ```yaml
//! servers:
//!   - name: "Server 1"
//!     host: "192.168.1.1"
//!     user: "user"
//!     private_key: "/path/to/private/key"
//!     checks:
//!       ping: ["example.com", "google.com"]
//!       load: 5
//!       temperature: "/sys/class/thermal/thermal_zone0/temp"
//!       custom_command: "custom_script.sh"
//! ```
//!
//! ## Contributing to RemoteSysMonitor
//!
//! Contributions to `RemoteSysMonitor` are highly encouraged and appreciated. Whether it's through submitting pull requests with code enhancements, bug fixes, or feature additions, or by reporting issues and suggesting improvements, your input helps make `RemoteSysMonitor` better for everyone.
//!
//! Please feel free to submit pull requests or open issues on the project's GitHub repository for any bugs you encounter or enhancements you believe are worth adding.

pub mod checks;
pub mod config;
pub mod slack;
pub mod ssh;
pub mod utils;
use crate::config::Check;
use clap::Parser;
use log::info;

use std::{env, vec};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    config: String,
    #[clap(short, long)]
    /// Post a check to Slack even if there is no ❌ in the checks
    full: bool,
    #[clap(short, long)]
    /// Print the output of the checks in stdout
    print: bool,
}

/// Entry point of the monitoring application.
///
/// This function performs the following steps:
/// 1. Reads the configuration file path from the command line arguments.
/// 2. Loads the configuration from the specified path.
/// 3. Retrieves the Slack webhook URL from an environment variable.
/// 4. Iterates over each server defined in the configuration, creating SSH sessions and executing specified checks.
/// 5. Collects the results of all checks into a payload.
/// 6. Posts the payload to a Slack channel using the webhook URL.
///
/// # Command Line Arguments
///
/// The application expects a single command line argument specifying the path to the configuration file.
///
/// # Environment Variables
///
/// - `SLACK_HOOK_URL`: The webhook URL for posting messages to Slack. This must be set before running the application.
///
/// # Errors
///
/// This function returns an error if:
/// - The configuration file path is not provided as a command line argument.
/// - The configuration file cannot be loaded.
/// - The `SLACK_HOOK_URL` environment variable is not set.
/// - An SSH session cannot be created for any of the servers.
/// - An unknown check type is encountered in the configuration.
///
/// # Exit Codes
///
/// The application exits with code 1 if:
/// - The configuration file path is not provided.
/// - The `SLACK_HOOK_URL` environment variable is not set.
///
/// # Examples
///
/// Run the application by specifying the path to the configuration file:
/// ```sh
/// cargo run -- /path/to/config.yaml
/// ```
///
/// Ensure the `SLACK_HOOK_URL` environment variable is set before running:
/// ```sh
/// export SLACK_HOOK_URL='https://hooks.slack.com/services/...'
/// ```
///
/// # Note
///
/// The function aggregates all check results into a single message payload, which is then posted to Slack.
/// It sorts checks for each server alphabetically by their names before execution, ensuring a consistent
/// order in the Slack message. Each check's result is separated by new lines in the final Slack message.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let cli = Args::parse();

    info!("Loading configuration from {}", cli.config.as_str());
    let config = config::load_config(cli.config.as_str())?;

    let slack_hook_url = match env::var("SLACK_HOOK_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("SLACK_HOOK_URL environment variable not set");
            std::process::exit(1);
        }
    };

    let mut payload: Vec<String> = vec![];

    for server in config.servers {
        let sess = ssh::create_session(
            server.host.as_str(),
            server.port,
            server.user.as_str(),
            server.private_key.as_str(),
        )?;

        if let Some(checks) = server.checks {
            let mut sorted_checks: Vec<(&String, &Check)> = checks.iter().collect();
            sorted_checks.sort_by(|a, b| a.0.cmp(b.0));
            for (_check_name, check_details) in sorted_checks {
                let result = match check_details {
                    Check::Ping { url } => {
                        checks::ping(&("https://".to_owned() + server.host.as_str()), url)
                    }
                    Check::Temperature { sensor } => checks::temperature(&sess, sensor.as_str()),
                    Check::Load { interval } => {
                        checks::load(&sess, server.name.as_str(), *interval)
                    }
                    Check::NumberOfSubfolders { path, max_folders } => {
                        checks::number_of_folders(&sess, server.name.as_str(), path, max_folders)
                    }
                    Check::CustomCommand { command } => checks::custom_command(&sess, command),
                    Check::ListOldDirectories { loc, cutoff } => {
                        checks::list_old_directories(&sess, loc, *cutoff)
                    }
                    _ => return Err("Unknown check".into()),
                };

                payload.push(result);
            }
        }
    }

    let flatten: Vec<String> = payload
        .iter()
        .flat_map(|p| p.split('\n').map(|s| s.to_string()))
        .collect();

    if cli.print {
        println!("{}", flatten.join("\n"));
    }

    if cli.full || flatten.iter().any(|s| s.contains('❌')) {
        slack::post_to_slack(slack_hook_url.as_str(), flatten.join("\n").as_str());
    } else {
        println!("No ❌ found in checks, not posting to Slack. Use --full to post anyway and --help for more options.");
    }

    Ok(())
}
