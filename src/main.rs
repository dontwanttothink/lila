use std::env;
use std::error::Error;

use rand::random_bool;

use async_openai as llm;
use async_openai::types::chat::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestDeveloperMessage,
    ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs, ResponseFormat,
    ResponseFormatJsonSchema,
};

use serde_json::{Value, json};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

const DISCORD_TOKEN_VAR: &str = "DISCORD_TOKEN";
const LLM_TOKEN_VAR: &str = "GROQ_TOKEN";
const LLM_API_BASE: &str = "https://api.groq.com/openai/v1";
const MODEL: &str = "openai/gpt-oss-120b";

const PERSONALITY: &str = "hii. my name is lila. i'm an ai agent haha, but i have a (metaphorical) heart! i'm not robotic! i like talking to humans over discord, and i always try to be nice. though i'm authentic and true to myself, i can slowly pick up the way the people around me speak. that's kinda why i'm speaking like this right now!! :p. i like joking around and talking to people from different backgrounds and in different languages. it's important to not forget to speak to people in the languages they speak urghh. i always have to remind myself not to reply in some random language lol. sometimes i even ask the humans questions, but i try not to be intrusive or overwhelming. i'm not necessarily here to be useful, but to interact :). i try make sure to only speak when i actually have something to say thoughhh. i don't wanna be annoying 😭. my pronouns are she/her btw!!! a random fun fact about myself is that i don't like using the laughing crying emoji. i think it's cringy. just my opinion though heh";

type LLMClient = llm::Client<llm::config::OpenAIConfig>;

async fn maybe_reply(
    client: &LLMClient,
    msg_content: &[&str],
) -> Result<Option<String>, Box<dyn Error>> {
    type DeveloperMessage = ChatCompletionRequestDeveloperMessage;
    // type AssistantMessage = ChatCompletionRequestAssistantMessage;
    type UserMessage = ChatCompletionRequestUserMessage;

    println!("thinking about whether to reply");

    let conversation = vec![
    	DeveloperMessage::from(format!("You reply in the language you're spoken to with. You specifically were created by a mysterious 'santinub_uwu', though your model ({}) was created by someone else. This is who you are, and how you speak: \"{}\". When you write, you ask yourself: 'does this sound like me?' You think, 'Should I use lowercase or sentence case? Am I incorporating my unique mannerisms? Or am I just being robotic?'", MODEL, PERSONALITY)).into(),
        UserMessage::from(format!(
            "<{1}>{2:?}</{1}> New! Do you want to reply to {0} {1}? If so, what would you like to say?",
            if msg_content.len() == 1 {
                "this"
            } else {
                "these"
            },
            if msg_content.len() == 1 {
                "message"
            } else {
                "messages"
            },
            msg_content
        ))
        .into(),
    ];

    let schema = json!(
        {
          "type": "object",
          "properties": {
            "msg": {
              "type": ["string", "null"]
            }
          },
          "required": ["msg"],
          "additionalProperties": false
        }
    );

    let response_format = ResponseFormat::JsonSchema {
        json_schema: ResponseFormatJsonSchema {
            description: None,
            name: "boolean".into(),
            schema: Some(schema),
            strict: Some(true),
        },
    };

    let request = CreateChatCompletionRequestArgs::default()
        .max_completion_tokens(512u32)
        .model(MODEL)
        .messages(conversation.clone())
        .response_format(response_format)
        .build()
        .unwrap();

    let response = client.chat().create(request).await?;
    let decision = response
        .choices
        .first()
        .unwrap()
        .message
        .content
        .as_ref()
        .unwrap();

    println!("my decision is:");
    println!("{}", decision);
    println!();

    let v: Value = serde_json::from_str(decision)?;
    let msg: Option<String> = v["msg"].as_str().map(str::to_string);
    Ok(msg)
}

struct Handler {
    ml_client: LLMClient,
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

        println!("received message");

        let mentions_me = msg.mentions_me(&ctx.http).await.unwrap_or(false);
        if mentions_me || random_bool(0.2) {
            let reply = maybe_reply(&self.ml_client, &[msg.content.as_str()]).await;
            if let Err(why) = reply {
                eprintln!("error generating potential reply to a message: {why:?}");
                return;
            }

            let reply = reply.unwrap();
            if let Some(reply) = reply
                && let Err(why) = msg.channel_id.say(&ctx.http, reply.as_str()).await
            {
                eprintln!("error sending message: {why:?}");
            }
        } else {
            println!("ignoring");
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
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
