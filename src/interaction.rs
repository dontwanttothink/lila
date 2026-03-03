use crate::settings::{MESSAGE_CONTEXT_SIZE, MODEL, SUMMARIZATION_MODEL};
use std::error::Error;

use async_openai as llm;
use async_openai::types::chat::{
    ChatCompletionMessageToolCalls, ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestDeveloperMessage,
    ChatCompletionRequestMessage, ChatCompletionRequestToolMessage,
    ChatCompletionRequestUserMessage, CreateChatCompletionRequestArgs,
};

pub mod prompts;
mod tools;

use prompts::{SUMMARIZE, get_message_prompt};
use serenity::all::{ChannelId, Context, Message};
use tokio::task::JoinSet;
use tools::{call_tool, get_send_tool};

pub type DeveloperMessage = ChatCompletionRequestDeveloperMessage;
pub type AssistantMessage = ChatCompletionRequestAssistantMessage;
pub type UserMessage = ChatCompletionRequestUserMessage;

type LLMClient = llm::Client<llm::config::OpenAIConfig>;

pub async fn get_last_messages(
    ctx: &Context,
    channel_id: impl Into<ChannelId> + Clone,
) -> Result<Vec<Message>, Box<dyn Error>> {
    let mut messages = ctx
        .cache
        .channel_messages(channel_id.clone())
        .as_ref()
        .map(|msgs| msgs.values().map(ToOwned::to_owned).collect::<Vec<_>>())
        .unwrap_or_default();

    messages.sort_unstable_by_key(|m| m.timestamp);
    messages.truncate(MESSAGE_CONTEXT_SIZE as usize);

    if messages.len() < MESSAGE_CONTEXT_SIZE as usize {
        messages = ctx
            .http
            .get_messages(channel_id.into(), None, Some(MESSAGE_CONTEXT_SIZE))
            .await?
    }

    messages.sort_unstable_by_key(|m| m.timestamp);

    Ok(messages)
}

pub async fn maybe_reply(
    client: &LLMClient,
    ctx: Context,
    conversation: &mut Vec<ChatCompletionRequestMessage>,
    msg: Message,
) -> Result<(), Box<dyn Error>> {
    println!("thinking about whether to reply");

    let channel = msg.channel(&ctx).await?;
    let channel_id = msg.channel_id;

    let last_messages = get_last_messages(&ctx, channel_id).await?;
    let message_prompt = get_message_prompt(&ctx, last_messages.as_slice(), &channel).await?;
    conversation.push(UserMessage::from(message_prompt).into());

    let request = CreateChatCompletionRequestArgs::default()
        .max_completion_tokens(2048u32)
        .model(MODEL)
        .messages(conversation.clone())
        .tools([get_send_tool()])
        .build()
        .unwrap();

    let response = client.chat().create(request).await?;
    println!("{:?}", conversation);
    println!("{:?}", response);
    let message = &response
        .choices
        .first()
        .ok_or("no choices returned")?
        .message;

    if let Some(tool_calls) = &message.tool_calls {
        let mut set = JoinSet::new();

        conversation.push(
            ChatCompletionRequestAssistantMessageArgs::default()
                .tool_calls(tool_calls.as_slice())
                .build()?
                .into(),
        );

        for tool_call in tool_calls {
            println!("tool_call: {:?}", tool_call);
            if let ChatCompletionMessageToolCalls::Function(function_call) = tool_call {
                let ctx = ctx.clone();
                let name = function_call.function.name.clone();
                let arguments = function_call.function.arguments.clone();
                let id = function_call.id.clone();
                set.spawn(async move { (id, call_tool(&ctx, &name, &arguments).await) });
            }
        }

        while let Some(tool_result) = set.join_next().await {
            let (id, result) = dbg!(tool_result)?;
            conversation.push(ChatCompletionRequestMessage::Tool(
                ChatCompletionRequestToolMessage {
                    content: result.to_string().into(),
                    tool_call_id: id,
                },
            ));
        }
    }
    if let Some(decision) = message.content.as_ref() {
        conversation.push(AssistantMessage::from(decision.as_str()).into());
    }

    Ok(())
}

pub async fn summarize(client: &LLMClient, text: &str) -> Result<String, Box<dyn Error>> {
    let conversation = vec![
        DeveloperMessage::from(SUMMARIZE).into(),
        UserMessage::from(text).into(),
    ];

    let request = CreateChatCompletionRequestArgs::default()
        .model(SUMMARIZATION_MODEL)
        .max_completion_tokens(512u32)
        .messages(conversation)
        .build()
        .unwrap();

    let response = client.chat().create(request).await?;
    let summary = response
        .choices
        .first()
        .unwrap()
        .message
        .content
        .to_owned()
        .unwrap();

    Ok(summary)
}
