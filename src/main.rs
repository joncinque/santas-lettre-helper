use {
    clap::{Arg, ArgAction, Command},
    lettre::{
        transport::stub::AsyncStubTransport, AsyncSendmailTransport, AsyncTransport, Message,
        Tokio1Executor,
    },
    rand::prelude::*,
    std::{collections::HashMap, error::Error},
};

const FROM_EMAIL: &str = "secret@santa.holiday";

type Email<'a> = &'a str;
type Name<'a> = &'a str;

fn valid_santa_for_participant(
    santa: Name,
    participant: Name,
    santas: &HashMap<Name, (Email, Name)>,
    avoids: &HashMap<Name, Name>,
) -> bool {
    santa != participant
        && santas.get(participant).map(|s| s.1) != Some(santa)
        && avoids.get(participant) != Some(&santa)
}

fn try_assign_santas<'a, 'b>(
    participants: &'b [(Name<'a>, Email<'a>)],
    avoids: &'b HashMap<Name<'a>, Name<'a>>,
) -> Result<HashMap<Name<'a>, (Email<'a>, Name<'a>)>, Box<dyn Error>> {
    let mut santas = HashMap::<Name, (Email, Name)>::new();
    let mut unassigned_santas = participants.to_vec().clone();
    let mut rng = rand::thread_rng();
    for participant in participants.iter() {
        unassigned_santas.shuffle(&mut rng);
        if let Some(santa) = unassigned_santas
            .iter()
            .find(|&&x| valid_santa_for_participant(x.0, participant.0, &santas, avoids))
        {
            let santa = *santa;
            unassigned_santas.retain(|&s| s.0 != santa.0);
            println!("{} gives a gift to {}", santa.0, participant.0);
            santas.insert(santa.0, (santa.1, participant.0));
        } else {
            return Err("Could not resolve santas".into());
        }
    }
    Ok(santas)
}

fn assign_santas<'a>(
    mut participants: Vec<(Name<'a>, Email<'a>)>,
    avoids: &HashMap<Name<'a>, Name<'a>>,
) -> HashMap<Name<'a>, (Email<'a>, Name<'a>)> {
    let mut rng = rand::thread_rng();
    participants.shuffle(&mut rng);
    loop {
        match try_assign_santas(&participants, avoids) {
            Ok(santas) => return santas,
            Err(err) => eprintln!("Assign: {}", err),
        }
    }
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
                .long("from")
                .value_name("FROM_EMAIL")
                .help("Sets the from email address")
                .default_value(FROM_EMAIL)
        )
        .arg(
            Arg::new("avoid")
                .long("avoid")
                .value_name("NAME:NAME")
                .help(
                    "Specifies two participants to *not* match as their two names, separated with a ':', e.g. Santa:Elf",
                )
                .action(ArgAction::Append),
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
    let from = matches.get_one::<String>("from").unwrap();
    let reply_to = matches.get_one::<String>("reply_to").unwrap_or(from);
    let participants: Vec<(Name, Email)> = matches
        .get_many::<String>("participant")
        .unwrap()
        .map(|p| p.split_once(':').unwrap())
        .collect();
    let mut avoids = HashMap::new();
    matches
        .get_many::<String>("avoid")
        .unwrap_or_default()
        .for_each(|p| {
            let (a, b) = p.split_once(':').unwrap();
            avoids.insert(a, b);
            avoids.insert(b, a);
        });
    let santas = assign_santas(participants, &avoids);

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
        let send_ok = if matches.get_flag("test") {
            let sender = AsyncStubTransport::new_ok();
            sender.send(email).await.is_ok()
        } else {
            let sender = AsyncSendmailTransport::<Tokio1Executor>::new();
            sender.send(email).await.is_ok()
        };
        assert!(send_ok);
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

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
        let avoids = [
            ("1", "2"),
            ("2", "1"),
            ("3", "4"),
            ("4", "3"),
            ("5", "6"),
            ("6", "5"),
            ("7", "8"),
            ("8", "7"),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();
        for _ in 1..1_000 {
            let santas = assign_santas(participants.clone(), &avoids);
            for (santa, (_, recipient)) in &santas {
                let recipient_of_recipient = &santas.get(recipient).unwrap().1;
                assert_ne!(santa, recipient_of_recipient);
                assert!(avoids.get(santa) != Some(recipient));
            }
        }
    }
}
