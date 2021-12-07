use {
    clap::{App, Arg},
    lettre::{AsyncSendmailTransport, AsyncTransport, Message, transport::stub::StubTransport, Tokio1Executor},
    rand::prelude::*,
    std::{collections::HashMap, error::Error},
};

const FROM_EMAIL: &str = "Secret Santa";

type Email<'a> = &'a str;
type Name<'a> = &'a str;

fn assign_santas<'a>(mut participants: Vec<(Name<'a>, Email<'a>)>) -> HashMap<(Name, Email), Name> {
    let mut santas = HashMap::new();
    let mut unassigned_santas = participants.clone();

    let mut rng = rand::thread_rng();
    participants.shuffle(&mut rng);
    for participant in participants.iter() {
        let mut santa = *unassigned_santas.choose(&mut rng).unwrap();
        while santa.0 == participant.0 || santas.get(participant) == Some(&santa.0) {
            santa = *unassigned_santas.choose(&mut rng).unwrap();
        }

        unassigned_santas.retain(|&s| s.0 != santa.0);
        println!("{} gives a gift to {}", santa.0, participant.0);
        santas.insert(santa, participant.0);
    }
    santas
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("Santa's Lettre Helper")
        .version("0.1")
        .about("Match up people for Secret Santa, and send out an email to everyone!")
        .arg(
            Arg::with_name("test")
                .short("t")
                .long("test")
                .help("Test mode, doesn't actually send emails")
                .takes_value(false)
        )
        .arg(
            Arg::with_name("reply_to")
                .short("r")
                .long("reply-to")
                .help("Sets the reply email file to use"),
        )
        .arg(
            Arg::with_name("from")
                .value_name("FROM_EMAIL")
                .help("Sets the from email address")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("participant")
                .value_name("NAME:EMAIL")
                .help(
                    "Specifies a participant as their name and email, separated with a ':', e.g. Santa:santa@northpole.com",
                )
                .required(true)
                .multiple(true),
        )
        .get_matches();
    let from = matches.value_of("from").unwrap_or(FROM_EMAIL);
    let reply_to = matches.value_of("reply_to").unwrap_or(from);
    let participants: Vec<(Name, Email)> = matches
        .values_of("participant")
        .unwrap()
        .map(|p| p.split_once(':').unwrap())
        .collect();
    let santas = assign_santas(participants);

    for (santa, recipient) in santas {
        let email = Message::builder()
            .from(from.parse()?)
            .reply_to(reply_to.parse()?)
            .to(santa.1.parse()?)
            .subject("Secret Santa")
            .body(format!(
                "Hey {}, it's Secret Santa.  Get a gift for {}!",
                santa.0, recipient
            ))?;
        // types are gross, we can probably do this better some other time
        if matches.is_present("test") {
            let sender = StubTransport::new_ok();
            let result = sender.send(email).await;
            assert!(result.is_ok());
        } else {
            let sender = AsyncSendmailTransport::<Tokio1Executor>::new();
            let result = sender.send(email).await;
            assert!(result.is_ok());
        };
    }
    Ok(())
}
