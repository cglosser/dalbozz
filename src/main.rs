use std::collections::HashMap;
use std::env;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod whimsy;
use whimsy::*;

mod slash_commands;

use serenity::{
    async_trait,
    model::{
        channel::Message,
        gateway::Ready,
        id::{ChannelId, UserId},
        interactions::Interaction,
    },
    prelude::*,
    utils::MessageBuilder,
};

struct Handler;

struct UpcomingPolls;
impl TypeMapKey for UpcomingPolls {
    type Value = HashMap<UserId, UpcomingPoll>;
}

struct UpcomingPoll {
    channel_to_post: ChannelId,
    game_names: Vec<String>,
}

struct BotUserId;
impl TypeMapKey for BotUserId {
    type Value = UserId;
}

#[async_trait]
impl EventHandler for Handler {
    // On connecting to discord
    async fn ready(&self, ctx: Context, ready: Ready) {
        let mut data = ctx.data.write().await;
        data.insert::<BotUserId>(ready.user.id);

        println!("{} is connected!", ready.user.name);
    }

    // On message received, either in a channel or directly from a
    // user.
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages sent from Dalbozz itself
        let bot_user_id =
            ctx.data.read().await.get::<BotUserId>().unwrap().clone();
        if msg.author.id == bot_user_id {
            return;
        }

        println!("Message received from {}: {}", msg.author.name, msg.content);

        ping(&ctx, &msg).await;
        add_reaction_emoji(&ctx, &msg).await;
        whoami(&ctx, &msg).await;

        if msg.is_private() {
            let mut data = ctx.data.write().await;
            let polls = data.get_mut::<UpcomingPolls>().unwrap();
            let mut poll = polls.get_mut(&msg.author.id);
            match poll {
                None => {
                    let res = msg
                        .reply(&ctx, "You don't have any polls in preparation")
                        .await;
                    if let Err(err) = res {
                        println!("Error chastising user: {:?}", err);
                    }
                }
                Some(ref mut poll) => {
                    if msg.content.to_lowercase() == "done" {
                        println!("Posting poll");
                        let game_names = poll.game_names.join("\n");
                        let poll_post = MessageBuilder::new()
                            .mention(&msg.author)
                            .push(" is looking for a group.  ")
                            .push("Game options are:\n")
                            .push(game_names)
                            .build();
                        let res =
                            poll.channel_to_post.say(&ctx, poll_post).await;
                        if let Err(err) = res {
                            println!("Couldn't make the poll in the original channel: {:?}", err);
                        }
                    } else {
                        println!(
                            "Adding another game to the poll: {}",
                            msg.content
                        );
                        poll.game_names.push(msg.content);
                    }
                }
            }
        }
    }

    // On receiving a slash command
    async fn interaction_create(&self, ctx: Context, command: Interaction) {
        println!("Slash command: {:?}", command);

        let res = slash_commands::send_response(&command, false, None).await;

        if let Err(err) = res {
            println!(
                "Error replying to {} ({}): {:?}",
                command.member.user.name,
                command.member.nick.unwrap_or("no nickname".to_string()),
                err
            );
            return;
        }

        let reply = MessageBuilder::new()
            .mention(&command.member.user)
            .push(" has started a new LFG.  ")
            .push("When done, it will be posted to this channel")
            .build();

        let res = command.channel_id.say(&ctx.http, &reply).await;
        if let Err(err) = res {
            println!("Error posting message: {:?}", err);
        }

        let pm = "It looks like you're using LFG.  Reply with the name of each game in the poll, then reply 'Done'.";
        let res = command
            .member
            .user
            .direct_message(&ctx.http, |m| m.content(&pm))
            .await;
        if let Err(err) = res {
            println!("Error initiating config: {:?}", err);
        }

        let mut data = ctx.data.write().await;
        let polls = data.get_mut::<UpcomingPolls>().unwrap();
        polls.insert(
            command.member.user.id,
            UpcomingPoll {
                channel_to_post: command.channel_id,
                game_names: Vec::new(),
            },
        );
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token =
        env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let app_id =
        env::var("DISCORD_APP_ID").expect("Expected app id in the environment");
    let server_id = env::var("DISCORD_SERVER_ID").ok();

    // Send the HTTP POST command to update the slash commands.
    let command_config_fut =
        slash_commands::configure_commands(&token, &app_id, &server_id);

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let client_fut = Client::builder(&token).event_handler(Handler);

    let (client, _command_config) =
        futures::join!(client_fut, command_config_fut);

    let mut client = client.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<UpcomingPolls>(HashMap::new());
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
