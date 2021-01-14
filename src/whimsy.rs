use serenity::{
    client::Context,
    model::channel::{Message, ReactionType},
};

pub async fn add_reaction_emoji(ctx: &Context, msg: &Message) {
    const EMOJI: [(&str, &str); 2] = [
        ("🐔", "🥚"), // chicken -> egg
        ("🐴", "💎"), // horse -> gemstone
    ];

    for (src, reaction) in EMOJI.iter() {
        if msg.content.contains(src) {
            if let Err(why) = msg
                .react(&ctx.http, ReactionType::Unicode(String::from(*reaction)))
                .await
            {
                println!("Error adding reaction emoji: {:?}", why);
            }
        }
    }
}
