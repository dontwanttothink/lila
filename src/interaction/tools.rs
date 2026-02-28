use std::error::Error;

use async_openai::types::chat::{
    ChatCompletionTool,
    ChatCompletionTools::{self, Function},
    FunctionObjectArgs,
};
use serde_json::{Value, json};
use serenity::all::{ChannelId, Context};

pub fn get_send_tool() -> ChatCompletionTools {
    Function(ChatCompletionTool {
        function: FunctionObjectArgs::default()
            .name("send")
            .description("Send a message to a channel")
            .parameters(json!({
                "type": "object",
                "properties": {
                    "channel_id": {
                        "type": "string",
                        "description": "The ID of the channel you want to send a message in."
                    },
                    "message": {
                        "type": "string",
                        "description": "The content of the message."
                    }
                },
                "required": ["channel_id", "message"],
                "additionalProperties": false
            }))
            .strict(true)
            .build()
            .unwrap(),
    })
}

pub async fn dispatch_send_message(ctx: &Context, call: Value) -> Value {
    let channel_id = match call["channel_id"].as_str().map(|id| id.parse()) {
        Some(Ok(v)) => v,
        Some(Err(_)) => return json!({"error": "cannot parse channel id"}),
        None => return json!({"error": "missing channel id or wrong type"}),
    };
    let msg = match call["message"].as_str() {
        Some(v) => v,
        None => return json!({"error": "missing message"}),
    };

    match send_message(ctx, channel_id, msg).await {
        Ok(_) => json!({"success": true}),
        Err(why) => json!({"error": why.to_string()}),
    }
}

pub async fn send_message(ctx: &Context, channel_id: u64, msg: &str) -> Result<(), Box<dyn Error>> {
    ChannelId::new(channel_id).say(&ctx.http, msg).await?;
    Ok(())
}

pub async fn call_tool(ctx: &Context, name: &str, args: &str) -> Value {
    let parsed = match args.parse::<Value>() {
        Ok(v) => v,
        Err(why) => return json!({"error": why.to_string()}),
    };

    match name {
        "send" => dispatch_send_message(ctx, parsed).await,
        _ => json!({"error": "unknown tool"}),
    }
}
