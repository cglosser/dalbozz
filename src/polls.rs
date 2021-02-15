use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::sync::Arc;

use itertools::Itertools;

use serenity::{
    http::client::Http,
    model::{
        channel::{Message, ReactionType},
        id::{ChannelId, UserId},
        user::User,
    },
    prelude::*,
    utils::MessageBuilder,
};

pub enum PollError {
    TooManyOptions,
    SerenityError(SerenityError),
    HttpError(reqwest::Error),
}

impl From<SerenityError> for PollError {
    fn from(e: SerenityError) -> Self {
        PollError::SerenityError(e)
    }
}

impl From<reqwest::Error> for PollError {
    fn from(e: reqwest::Error) -> Self {
        PollError::HttpError(e)
    }
}

pub struct Polls {
    http: Arc<Http>,
    upcoming: HashMap<UserId, UpcomingPoll>,
    //TODO active
}

struct UpcomingPoll {
    author: User,
    channel_to_post: ChannelId,
    games: Vec<UpcomingGame>,
}

struct UpcomingGame {
    name: String,
    icon: String,
}

impl Polls {
    pub fn new(http: Arc<Http>) -> Polls {
        Polls {
            http,
            upcoming: HashMap::new(),
        }
    }

    pub async fn start_new(
        &mut self,
        user: User,
        channel: ChannelId,
    ) -> Result<(), PollError> {
        let reply = MessageBuilder::new()
            .mention(&user)
            .push(" has started a new LFG.  ")
            .push("When done, it will be posted to this channel")
            .build();

        channel.say(&self.http, &reply).await?;

        let pm = "It looks like you're using LFG.  Reply with the name of each game in the poll, then reply 'Done'.";
        user.direct_message(&self.http, |m| m.content(&pm)).await?;

        self.upcoming.insert(
            user.id,
            UpcomingPoll {
                author: user,
                channel_to_post: channel,
                games: Vec::new(),
            },
        );

        Ok(())
    }

    pub async fn respond_to_private_message(
        &mut self,
        msg: Message,
    ) -> Result<(), PollError> {
        let poll = self.upcoming.get_mut(&msg.author.id);

        if let None = poll {
            msg.reply(&self.http, "You aren't currently preparing any polls.")
                .await?;
            return Ok(());
        }

        let poll = poll.unwrap();

        if msg.content.to_lowercase() == "done" {
            poll.post(&self.http).await?;
            self.upcoming.remove(&msg.author.id);
        } else {
            poll.add_game(&msg.content)?;
            poll.show_menu(&self.http).await?;
            //msg.delete(&self.http).await?;
            let res = msg.delete(&self.http).await;
            println!("Message deletion result: {:?}", res);
        }

        Ok(())
    }
}

impl UpcomingPoll {
    fn add_game(&mut self, game_name: &str) -> Result<(), PollError> {
        let existing_icons = self
            .games
            .iter()
            .map(|g| &g.icon)
            .to_owned()
            .collect::<HashSet<_>>();

        let icon = (0..26)
            // Map onto Regional indicator symbols
            // https://en.wikipedia.org/wiki/Regional_indicator_symbol
            .map(|c| std::char::from_u32(0x1F1E6 + c).unwrap().to_string())
            .filter(|c| !existing_icons.contains(c))
            .next()
            .ok_or(PollError::TooManyOptions)?;

        self.games.push(UpcomingGame {
            name: game_name.to_string(),
            icon,
        });
        println!("Added another game to the poll: {}", game_name);

        Ok(())
    }

    fn poll_text(&self) -> String {
        let game_names: String = self
            .games
            .iter()
            .map(|g| format!("{} {}", g.icon, g.name))
            .intersperse("\n".to_string())
            .collect();

        MessageBuilder::new()
            .mention(&self.author)
            .push(" is looking for a group.  ")
            .push("Game options are:\n")
            .push(game_names)
            .build()
    }

    async fn post(&self, ctx: &Arc<Http>) -> Result<(), PollError> {
        let res =
            self.channel_to_post
                .send_message(&ctx, |m| {
                    m.content(self.poll_text());
                    m.reactions(self.games.iter().map(|g| {
                        ReactionType::try_from(g.icon.as_str()).unwrap()
                    }));
                    m
                })
                .await;
        println!("Res = {:?}", res);
        println!("Posted poll to the channel");

        Ok(())
    }

    async fn show_menu(&self, ctx: &Arc<Http>) -> Result<(), PollError> {
        let reply = MessageBuilder::new()
            .push_line("The poll will show up as follows.")
            .push_quote_line(self.poll_text())
            .push_line("Reply \"done\" to finish, or reply with another game for the poll.")
            .build();

        self.author
            .direct_message(&ctx, |m| m.content(&reply))
            .await?;

        Ok(())
    }
}
