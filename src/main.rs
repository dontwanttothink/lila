mod interaction;
mod preferences;

use std::env;

use async_openai::{self as llm};
use dotenvy::dotenv;
use rand::random_bool;

use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity::{all::UserId, async_trait};

use interaction::{DeveloperMessage, maybe_reply, prompts::get_preamble_prompt};

use crate::preferences::{DISCORD_TOKEN_VAR, LLM_API_BASE, LLM_TOKEN_VAR, MODEL};

type LLMClient = llm::Client<llm::config::OpenAIConfig>;

struct Handler {
    ml_client: LLMClient,
}

struct CurrentUserID;
impl TypeMapKey for CurrentUserID {
    type Value = UserId;
}

impl Handler {
    fn new(ml_client: LLMClient) -> Self {
        Self { ml_client }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let bot_id = ctx
            .data
            .read()
            .await
            .get::<CurrentUserID>()
            .unwrap()
            .to_string();
        println!("received message");

        let mut conversation =
            vec![DeveloperMessage::from(get_preamble_prompt(MODEL, bot_id.as_str())).into()];
        // let mut conversation = vec![
        //     ChatCompletionRequestSystemMessage::from(get_preamble_prompt(MODEL, bot_id.as_str()))
        //         .into(),
        // ];

        let mentions_me = msg.mentions_me(&ctx.http).await.unwrap_or(false);
        if mentions_me || random_bool(0.2) {
            let typing = msg.channel_id.start_typing(&ctx.http);
            if let Err(why) = maybe_reply(&self.ml_client, ctx, &mut conversation, msg).await {
                println!("error replying: {why:?}");
            }
            typing.stop();
        } else {
            println!("ignoring");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let mut data = ctx.data.write().await;
        data.insert::<CurrentUserID>(ready.user.id);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var(DISCORD_TOKEN_VAR).expect("expected a Discord token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let llm_config = llm::config::OpenAIConfig::new()
        .with_api_base(LLM_API_BASE)
        .with_api_key(env::var(LLM_TOKEN_VAR).expect("expected an LLM token in the environment"));
    let llm_client = llm::Client::with_config(llm_config);

    let handler = Handler::new(llm_client);
    let mut client = Client::builder(&token, intents)
        .event_handler(handler)
        .await
        .expect("err creating client");

    if let Err(why) = client.start().await {
        eprintln!("client error: {why:?}");
    }
}
