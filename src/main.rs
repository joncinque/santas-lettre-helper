use lettre::{Message, AsyncTransport, Tokio1Executor, AsyncSendmailTransport};
use std::error::Error;

const FROM_EMAIL: &str = "Secret Santa";
const REPLY_TO_EMAIL: &str = "Santa's Lettre Helper";
const TO_EMAIL: &str = "Secret Santa Recipient";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let email = Message::builder()
        .from(FROM_EMAIL.parse()?)
        .reply_to(REPLY_TO_EMAIL.parse()?)
        .to(TO_EMAIL.parse()?)
        .subject("Secret Santa")
        .body(String::from("Hey there, it's Secret Santa.  Get a gift for XXXXX!"))?;
    let sender = AsyncSendmailTransport::<Tokio1Executor>::new();
    let result = sender.send(email).await;
    assert!(result.is_ok());
    Ok(())
}
