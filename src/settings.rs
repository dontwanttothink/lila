pub const DISCORD_TOKEN_VAR: &str = "DISCORD_TOKEN";
// const LLM_TOKEN_VAR: &str = "GROQ_TOKEN";
// const LLM_API_BASE: &str = "https://api.groq.com/openai/v1";
// pub const MODEL: &str = "openai/gpt-oss-120b";
pub const LLM_TOKEN_VAR: &str = "OPEN_ROUTER_TOKEN";
pub const LLM_API_BASE: &str = "https://openrouter.ai/api/v1";
// pub const MODEL: &str = "anthropic/claude-haiku-4.5"; // it has all the claude mannerisms — a bit annoying as it goes on and on about alignment and being tested
pub const MODEL: &str = "google/gemini-3-flash-preview"; // really good; honestly perfect
// pub const MODEL: &str = "bytedance-seed/seed-2.0-mini"; // good but forgets language sometimes and is really slow and uses way too many tokens
// pub const MODEL: &str = "arcee-ai/trinity-large-preview:free"; // good but forgets style sometimes
// pub const MODEL: &str = "minimax/minimax-m2.5"; // really good actually but frequently fails at tool calling

pub const SUMMARIZATION_MODEL: &str = "llama-3.1-8b-instant";

pub const MESSAGE_CONTEXT_SIZE: u8 = 20;
