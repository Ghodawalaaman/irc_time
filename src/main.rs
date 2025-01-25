use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec;
use std::time;


use futures::prelude::*;
use irc::client::prelude::*;
use irc::client::prelude::Prefix::Nickname;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let start = SystemTime::now();
    let bot_join_time = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut user_stay_time: HashMap<String, u64> = HashMap::new();
    let config = Config {
        nickname: Some("time_counter".to_owned()),
        server: Some("irc.libera.chat".to_owned()),
        channels: vec!["#bsah".to_owned()],
        ..Config::default()
    };
    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;
    let sender = client.sender();

    let mut interval = tokio::time::interval(time::Duration::from_secs(1));

    let mut joined_users: Vec<String> = Vec::new();
    loop {
        tokio::select! {
            Some(m) = stream.next() => {
                let message = m?;
                let Message { tags, prefix, command } = message;
                match command {
                    Command::Response(res, v) => {
                        if res == Response::RPL_NAMREPLY {
                            let tmp = v.clone();
                            joined_users = tmp[3].split(" ").map(|s| s.to_string()).collect::<Vec<_>>();
                        }
                    },
                    Command::PRIVMSG(channel, text) => {
                        if let Some(prefix) = prefix {
                            if let Nickname(name, _, _) = prefix {
                                if text.len() > 4 && &text[0..5] == "@time" {
                                    match user_stay_time.get(&name) {
                                        Some(time) => {
                                            let message = format!("You have been spent {} seconds since {} in #bsah", time, bot_join_time);
                                            sender.send_privmsg(channel, message)?;
                                        }
                                        None => sender.send_privmsg(channel, 0)?,
                                    };
                                }
                                
                            }
                        }
                    },
                    _ => {},
                }
            }
            _ = interval.tick() => {
                let m = Message{
                    prefix: None,
                    tags: None,
                    command: Command::NAMES(Some("#bsah".to_owned()), None),
                };
                client.send(m)?;
                for u in joined_users.clone() {
                    user_stay_time.entry(u.to_string()).and_modify(|counter| *counter += 1).or_insert(0);
                }
            }
        }
    }


    Ok(())
}
