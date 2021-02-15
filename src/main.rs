use std::env;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod whimsy;
use whimsy::*;

mod polls;
use polls::{PollError, Polls};

mod slash_commands;

use serenity::{
    async_trait,
    model::{
        channel::Message, gateway::Ready, id::UserId, interactions::Interaction,
    },
    prelude::*,
};

struct Handler;

impl TypeMapKey for Polls {
    type Value = Polls;
}

struct BotUserId;
impl TypeMapKey for BotUserId {
    type Value = UserId;
}

impl Handler {
    fn log_err(&self, _err: PollError) {
        // TODO
    }
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
            let polls = data.get_mut::<Polls>().unwrap();
            polls
                .respond_to_private_message(msg)
                .await
                .err()
                .map(|e| self.log_err(e));
        }
    }

    // On receiving a slash command
    async fn interaction_create(&self, ctx: Context, command: Interaction) {
        println!("Slash command: {:?}", command);

        slash_commands::send_response(&command, false, None)
            .await
            .err()
            .map(|e| self.log_err(e.into()));

        if command.data.as_ref().map(|d| &d.name) == Some(&"lfg".to_string()) {
            let mut data = ctx.data.write().await;
            let polls = data.get_mut::<Polls>().unwrap();
            polls
                .start_new(command.member.user, command.channel_id)
                .await
                .err()
                .map(|e| self.log_err(e));
        } else {
            println!("Unknown command: {:?}", command);
        }
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
        data.insert::<Polls>(Polls::new(client.cache_and_http.http.clone()));
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
