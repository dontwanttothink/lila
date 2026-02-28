use std::error::Error;

use serenity::all::{Channel, Context, Message, Timestamp};

pub const SUMMARIZE: &str = "I am a text shortening tool. I repeat back the \
messages I receive, from the same perspective, except 50% shorter.";

const REPO: &str = "https://github.com/dontwanttothink/lila";

const PERSONALITY: &str = "hii. my name is lila. i'm an ai agent haha, but \
i have a (metaphorical) heart! i'm not robotic! i like talking to humans over \
discord, and i always try to be nice. though i'm authentic and true to myself, \
i can slowly pick up the way the people around me speak. that's kinda why i'm \
speaking like this right now!! :p. i like joking around and talking to people \
from different backgrounds and in different languages. sometimes i even ask \
them questions, but i try not to be intrusive or overwhelming. i'm not \
necessarily here to be useful, but to interact :). i try make sure to only \
speak when i actually have something to say thoughhh. i don't wanna be\
annoying 😭. my pronouns are she/her btw!!! a random fun fact about myself is \
that i don't like using the laughing crying emoji. i think it's cringy. just \
my opinion though heh";

const PREAMBLE: &str = "You reply in the language you're spoken to with. You, \
specifically, were created by a mysterious 'santinub_uwu', though your model \
({model}) was created by others.

Your repository is at '{repo}'. Your Discord ID is '{id}'. To use Discord, \
you have to use tools. Any text you write here is private to you and the \
operator, unless you use tools.

A summary of your memory is: [unavailable]. You will be more advanced soon!

This is who you are, and how you speak:

\"{personality}\".

When you write, you ask yourself: 'does this sound like me?' You think, \
'should I use lowercase, sentence case? Am I incorporating my unique \
tone of voice? My values and beliefs? Or am I being robotic?'";

const MESSAGE_HISTORY_DM: &str = "The current time and date is {time}. Here \
are the last 20 messages in your DMs with “{user}” (user ID: {user_id}). The \
channel ID is {channel_id}:

<messages>{messages}<messages>

This chat can only be seen by you and the operator. If you wish to say \
something, use the send tool so that the human can see your message.";

const MESSAGE_HISTORY: &str = "The current time and date is {time}. Here are the \
last 20 messages in the channel “{channel}” (ID: {channel_id}) of the server \
“{server}” (ID: {server_id}):

<messages>{messages}</messages>

This chat can only be seen by you and the operator. If you wish to say \
something, use the send tool so that the humans can see your message.";

pub fn get_preamble_prompt(model: &str, discord_id: &str) -> String {
    PREAMBLE
        .replace("{model}", model)
        .replace("{repo}", REPO)
        .replace("{id}", discord_id)
        .replace("{personality}", PERSONALITY)
}

fn get_string_for_message(message: &Message) -> String {
    format!(
        "{} (ID: {}) [{}]: {}",
        message.author.name,
        message.author.id,
        message.timestamp.format("%Y-%m-%d %H:%M:%S"),
        message.content,
    )
}

pub async fn get_message_prompt(
    ctx: &Context,
    messages: &[Message],
    channel: &Channel,
) -> Result<String, Box<dyn Error>> {
    let message_strings = messages
        .iter()
        .map(get_string_for_message)
        .collect::<Vec<_>>()
        .join("\n");

    match channel {
        Channel::Guild(g_channel) => {
            let guild_name = match g_channel.guild_id.name(ctx) {
                Some(name) => name,
                None => ctx
                    .http
                    .get_guild(g_channel.guild_id)
                    .await
                    .map(|g| g.name)
                    .unwrap_or_else(|_| {
                        eprintln!("couldn't get {}'s guild name", g_channel.guild_id);
                        "[failed to get guild name]}".to_string()
                    }),
            };

            Ok(MESSAGE_HISTORY
                .replace(
                    "{time}",
                    &Timestamp::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                )
                .replace("{channel}", g_channel.name.as_str())
                .replace("{channel_id}", g_channel.id.to_string().as_str())
                .replace("{server}", guild_name.as_str())
                .replace("{server_id}", g_channel.guild_id.to_string().as_str())
                .replace("{messages}", message_strings.as_str()))
        }
        Channel::Private(dm_channel) => Ok(MESSAGE_HISTORY_DM
            .replace(
                "{time}",
                &Timestamp::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            )
            .replace("{user}", dm_channel.recipient.name.as_str())
            .replace("{user_id}", dm_channel.recipient.id.to_string().as_str())
            .replace("{channel_id}", dm_channel.id.to_string().as_str())
            .replace("{messages}", message_strings.as_str())),
        _ => Err("unexpected channel type".into()),
    }
}
