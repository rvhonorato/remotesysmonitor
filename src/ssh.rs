use log::error;

/// This module handles SSH connections and command execution.
///
/// It provides functionality to create SSH sessions and run commands on a remote server
/// using the `ssh2` crate for Rust.
use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

/// Executes a given command on an SSH session and returns the command's output as a `String`.
///
/// This function opens a new channel on the provided SSH session, executes the specified command,
/// and reads the output of the command into a `String`. It ensures that the command executes
/// successfully by checking the command's exit status. If the command execution fails or if reading
/// the output encounters an error, the function logs the error and returns an `Err` containing
/// the error information.
///
/// # Parameters
/// - `sess`: A reference to an established SSH `Session` object.
/// - `command`: The command to be executed on the SSH session as a string slice.
///
/// # Returns
/// - `Ok(String)`: The output of the successfully executed command as a string.
/// - `Err(Box<dyn std::error::Error>)`: An error boxed as `Box<dyn std::error::Error>` if the command
///   execution fails at any step, including establishing a channel, executing the command,
///   reading the output, closing the channel, or if the command exits with a non-zero status.
///
/// # Examples
/// ```no_run
/// use ssh2::Session;
/// use std::net::TcpStream;
///
/// let tcp = TcpStream::connect("127.0.0.1:22").unwrap();
/// let mut sess = Session::new().unwrap();
/// sess.set_tcp_stream(tcp);
/// sess.handshake().unwrap();
/// sess.userauth_password("username", "password").unwrap();
///
/// let output = run_ssh_command(&sess, "echo Hello, world!").unwrap();
/// println!("Command output: {}", output);
/// ```
///
/// # Errors
/// This function will return an error if:
/// - It fails to open a new channel on the SSH session.
/// - The command execution fails.
/// - Reading the command output into a string fails.
/// - The command exits with a non-zero status.
///
/// All errors are logged with an appropriate error message and then returned as `Box<dyn std::error::Error>`.
pub fn run_ssh_command(
    sess: &Session,
    command: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut channel = sess.channel_session()?;
    channel.exec(command).map_err(|e| {
        error!(
            "Could not execute command '{}' due to error: {}",
            command, e
        );
        Box::<dyn std::error::Error>::from(e)
    })?;
    let mut s = String::new();
    channel.read_to_string(&mut s).map_err(|e| {
        error!(
            "could not read output of command '{}' due to error: {}",
            command, e
        );
        Box::<dyn std::error::Error>::from(e)
    })?;

    channel.wait_close().ok();
    let exit_status = channel.exit_status()?;
    if exit_status != 0 {
        return Err(Box::<dyn std::error::Error>::from(format!(
            "Command '{}' exited with status {}",
            command, exit_status
        )));
    }

    Ok(s)
}

/// Establishes an SSH session using a private key for authentication.
///
/// This function attempts to connect to an SSH server at a specified host and port,
/// then authenticates the session using a specified username and the private key
/// located at `private_key_path`. It ensures that the session is authenticated before
/// returning the session object.
///
/// # Parameters
/// - `host`: The hostname or IP address of the SSH server as a string slice.
/// - `port`: The port number on which the SSH server is listening.
/// - `username`: The username for authentication with the SSH server.
/// - `private_key_path`: The filesystem path to the private key file used for authentication.
///
/// # Returns
/// - `Ok(Session)`: An authenticated SSH `Session` object if the connection and authentication succeed.
/// - `Err(Box<dyn std::error::Error>)`: An error boxed as `Box<dyn std::error::Error>` if any step of the
///   session establishment process fails, including TCP connection establishment, session creation,
///   session handshake, or authentication.
///
/// # Examples
/// ```no_run
/// use ssh2::Session;
/// use std::net::TcpStream;
///
/// let session = create_session("127.0.0.1", 22, "username", "/path/to/private/key").unwrap();
/// // Use `session` for executing commands, transferring files, etc.
/// ```
///
/// # Errors
/// This function will return an error in the following cases:
/// - TCP connection to the specified host and port fails.
/// - Creation of the SSH session object fails.
/// - The SSH handshake fails.
/// - Authentication with the provided username and private key fails.
/// - The session is not authenticated after attempting the provided authentication method.
///
/// All errors are logged with an appropriate message for debugging purposes.
///
/// # Remarks
/// The function requires an SSH server to be accessible at the specified host and port.
/// The private key file specified by `private_key_path` must be in a format recognized
/// by the server (e.g., RSA, DSA) and must not be encrypted with a passphrase.
pub fn create_session(
    host: &str,
    port: u16,
    username: &str,
    private_key_path: &str,
) -> Result<Session, Box<dyn std::error::Error>> {
    let host_w_port = format!("{}:{}", host, port);
    let tcp = TcpStream::connect(&host_w_port).map_err(|e| {
        error!("Could not connect to {}", host_w_port);
        Box::<dyn std::error::Error>::from(e)
    })?;

    let mut sess = Session::new().expect("Failed to create SSH session");
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    sess.userauth_pubkey_file(username, None, Path::new(private_key_path), None)
        .map_err(|e| {
            error!(
                "could not authenticate with {} using {}: {}",
                host, private_key_path, e
            );
            Box::<dyn std::error::Error>::from(e)
        })?;

    if !sess.authenticated() {
        let err_msg = format!("Authentication failed: {}", host_w_port);
        error!("{}", err_msg);
        return Err(err_msg.into());
    }

    Ok(sess)
}

#[cfg(test)]
mod test {

    #[test]
    #[ignore] // Heavily relies on external resources
    fn test_create_session() {}

    #[test]
    #[ignore] // Heavily relies on external resources
    fn test_run_ssh_command() {}
}
