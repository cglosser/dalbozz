use serenity::{
    client::Context,
    model::channel::{Message, ReactionType},
};

pub async fn ping(ctx: &Context, msg: &Message) {
    if msg.content != "!ping" {
        return;
    }

    let res = msg.channel_id.say(&ctx.http, "Pong!").await;

    if let Err(why) = res {
        println!("Error sending message: {:?}", why);
    }
}

pub async fn whoami(ctx: &Context, msg: &Message) {
    if msg.content != "!whoami" {
        return;
    }

    let res = msg
        .author
        .direct_message(&ctx.http, |m| {
            m.content(format!("You are {}", msg.author.name))
        })
        .await;

    if let Err(why) = res {
        println!("Error sending direct message: {:?}", why);
    }
}

pub async fn add_reaction_emoji(ctx: &Context, msg: &Message) {
    const EMOJI: [(&str, &str); 2] = [
        ("🐔", "🥚"), // chicken -> egg
        ("🐴", "💎"), // horse -> gemstone
    ];

    for (src, reaction) in EMOJI.iter() {
        if msg.content.contains(src) {
            if let Err(why) = msg
                .react(
                    &ctx.http,
                    ReactionType::Unicode(String::from(*reaction)),
                )
                .await
            {
                println!("Error adding reaction emoji: {:?}", why);
            }
        }
    }
}
