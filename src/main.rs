use {
    clap::{Arg, ArgAction, Command},
    lettre::{
        transport::stub::AsyncStubTransport, AsyncSendmailTransport, AsyncTransport, Message,
        Tokio1Executor,
    },
    rand::prelude::*,
    std::{collections::HashMap, error::Error},
};

const FROM_EMAIL: &str = "Secret Santa";

type Email<'a> = &'a str;
type Name<'a> = &'a str;

fn assign_santas<'a>(mut participants: Vec<(Name<'a>, Email<'a>)>) -> HashMap<Name, (Email, Name)> {
    let mut santas = HashMap::<Name, (Email, Name)>::new();
    let mut unassigned_santas = participants.clone();

    let mut rng = rand::thread_rng();
    participants.shuffle(&mut rng);
    for participant in participants.iter() {
        let mut santa = *unassigned_santas.choose(&mut rng).unwrap();
        while santa.0 == participant.0 || santas.get(participant.0).map(|s| s.1) == Some(&santa.0) {
            santa = *unassigned_santas.choose(&mut rng).unwrap();
        }

        unassigned_santas.retain(|&s| s.0 != santa.0);
        println!("{} gives a gift to {}", santa.0, participant.0);
        santas.insert(santa.0, (santa.1, participant.0));
    }
    santas
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let matches = Command::new("Santa's Lettre Helper")
        .version("0.1")
        .about("Match up people for Secret Santa, and send out an email to everyone!")
        .arg(
            Arg::new("test")
                .short('t')
                .long("test")
                .help("Test mode, doesn't actually send emails")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("reply_to")
                .short('r')
                .long("reply-to")
                .help("Sets the reply email file to use"),
        )
        .arg(
            Arg::new("from")
                .value_name("FROM_EMAIL")
                .help("Sets the from email address")
                .required(true)
        )
        .arg(
            Arg::new("participant")
                .value_name("NAME:EMAIL")
                .help(
                    "Specifies a participant as their name and email, separated with a ':', e.g. Santa:santa@northpole.com",
                )
                .required(true)
                .action(ArgAction::Append),
        )
        .get_matches();
    let from = matches.get_one::<&str>("from").unwrap_or(&FROM_EMAIL);
    let reply_to = matches.get_one::<&str>("reply_to").unwrap_or(from);
    let participants: Vec<(Name, Email)> = matches
        .get_many::<&str>("participant")
        .unwrap()
        .map(|p| p.split_once(':').unwrap())
        .collect();
    let santas = assign_santas(participants);

    for (santa, (email, recipient)) in santas {
        let email = Message::builder()
            .from(from.parse()?)
            .reply_to(reply_to.parse()?)
            .to(email.parse()?)
            .subject("Secret Santa")
            .body(format!(
                "Hey {}, it's Secret Santa.  Get a gift for {}!",
                santa, recipient
            ))?;
        // types are gross, we can probably do this better some other time
        if matches.get_flag("test") {
            let sender = AsyncStubTransport::new_ok();
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

#[cfg(test)]
mod test {
    use crate::assign_santas;

    #[test]
    fn success_assign() {
        let participants = vec![
            ("1", ""),
            ("2", ""),
            ("3", ""),
            ("4", ""),
            ("5", ""),
            ("6", ""),
            ("7", ""),
            ("8", ""),
        ];
        let santas = assign_santas(participants);
        for (santa, (_, recipient)) in &santas {
            let recipient_of_recipient = &santas.get(recipient).unwrap().1;
            assert_ne!(santa, recipient_of_recipient);
        }
    }
}
