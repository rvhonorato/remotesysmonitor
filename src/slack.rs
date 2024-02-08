use crate::utils;
use slack_hook::{PayloadBuilder, Slack};

/// Posts a message to a Slack channel using a webhook URL.
///
/// This function leverages the `slack_hook` crate to send messages to Slack. It constructs
/// a message payload from the provided text and sends it to the specified Slack webhook URL.
///
/// # Arguments
///
/// * `slack_hook_url` - A string slice containing the webhook URL provided by Slack. This URL is
///   unique to the Slack workspace and channel where the message will be posted.
/// * `payload` - The message text to be sent to Slack.
///
/// # Panics
///
/// This function will panic if the `Slack::new` or `PayloadBuilder::build` calls fail, which
/// can occur if the provided webhook URL is invalid or if there is an issue constructing the
/// message payload.
///
/// # Examples
///
/// ```
/// let webhook_url = "https://hooks.slack.com/services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX";
/// let message = "Hello, world! This is a test message from Rust.";
/// post_to_slack(webhook_url, message);
/// ```
///
/// # Note
///
/// The function currently prints the result of the Slack message operation to the console, with
/// "ok" indicating success and "ERR" followed by the error message indicating failure. Future
/// versions of this function might return a `Result` to allow calling code to handle successes
/// and errors more flexibly.
pub fn post_to_slack(slack_hook_url: &str, payload: &str) {
    // The payload is the message, add a timestamp
    let timestamp = utils::make_pretty_timestamp();
    let mut payload = format!("{}\n{}", timestamp, payload);

    // Check if there is any `❌` in the payload, if it contains, add a @all mention
    if payload.contains('❌') {
        payload = format!("@all\n{}", payload);
    }

    let slack = Slack::new(slack_hook_url).unwrap();
    let p = PayloadBuilder::new().text(payload).build().unwrap();
    let res = slack.send(&p);
    match res {
        Ok(()) => println!("ok"),
        Err(x) => eprintln!("ERR: {:?}", x),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    #[ignore] // TODO
    fn test_post_to_slack() {}
}
