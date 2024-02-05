/// This module handles SSH connections and command execution.
///
/// It provides functionality to create SSH sessions and run commands on a remote server
/// using the `ssh2` crate for Rust.
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

/// Executes a command on a remote server via an established SSH session.
///
/// # Arguments
///
/// * `sess` - A reference to an established SSH `Session`.
/// * `command` - The command to be executed on the remote server as a string slice.
///
/// # Returns
///
/// Returns a `Result<String, Box<dyn std::error::Error>>`. On success, it returns the command's
/// output as a `String`. On failure, it returns an error encapsulated in a `Box<dyn std::error::Error>`.
///
/// # Errors
///
/// This function can return an error if the SSH channel session cannot be created, the command
/// execution fails, or the command output cannot be read into a string.
pub fn run_ssh_command(
    sess: &Session,
    command: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut channel = sess.channel_session()?;
    channel.exec(command)?;
    let mut s = String::new();
    channel.read_to_string(&mut s)?;
    Ok(s)
}

/// Creates and establishes a new SSH session to a specified host and port using a given username and private key.
///
/// # Arguments
///
/// * `host` - The hostname or IP address of the server as a string slice.
/// * `port` - The port number on which to connect to the server.
/// * `username` - The username for authentication.
/// * `private_key_path` - The file path to the private key used for authentication.
///
/// # Returns
///
/// Returns a `Result<Session, Box<dyn std::error::Error>>`. On success, it returns an established SSH `Session`.
/// On failure, it returns an error encapsulated in a `Box<dyn std::error::Error>`.
///
/// # Errors
///
/// This function can return an error if the TCP connection to the server fails, the SSH session
/// cannot be created, the SSH handshake fails, or the public key authentication fails.
pub fn create_session(
    host: &str,
    port: u16,
    username: &str,
    private_key_path: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    let host_w_port = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(host_w_port)?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    sess.userauth_pubkey_file(username, None, Path::new(private_key_path), None)?;
    assert!(sess.authenticated());

    Ok(sess)
}
