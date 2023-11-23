use std::path::Path;

use anyhow::Ok;
use conversation::Conversation;
use spin_sdk::http::{IntoResponse, Json, Params, Request, Response, Router, ResponseBuilder};
use spin_sdk::http_component;
use spin_sdk::llm;

mod prompt;
use prompt::Prompt;

mod conversation;

const CONVERSATION_ID_HEADER_NAME: &str = "X-ConversationId";

#[http_component]
fn handle_ai_live(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::default();
    router.get("/", handle_readme);
    router.post("/", handle_prompt);
    router.get("/:conversationId", handle_get_conversation);
    Ok(router.handle(req))
}

/*
-- Incoming POST request at / ✅
-- Deserialize JSON body into Prompt struct ✅
-- Check if there is a X-ConversationId HTTP header
--- If there is, load the conversation with that Id from KV store ✅
--- If not, create a new conversation ✅
-- Construct a proper prompt ✅
-- Call the LLM ✅
-- add question and answer to the conversation (interaction) ✅
-- save the conversation to KV store ✅
-- return the answer and send conversation id as HTTP header ✅
*/

fn handle_prompt(req: http::Request<Json<Prompt>>, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let question = req.body().question.clone();
    let mut conversation = match req.headers().get(CONVERSATION_ID_HEADER_NAME) {
        Some(id) => Conversation::load(id.to_str()?),
        None => Ok(Conversation::new(uuid::Uuid::new_v4().to_string())),
    }?;

    let prompt = conversation.get_prompt(&question);
    let opts = llm::InferencingParams {
        max_tokens: 150, // 1 word is roughly 1.4 tokens
        temperature: 0.1,
        ..Default::default()
    };

    let result = llm::infer_with_options(llm::InferencingModel::Llama2Chat, &prompt, opts)?;
    conversation.add_interaction(&question, &result.text);
    conversation.save()?;

    Ok(Response::builder()
        .status(http::StatusCode::OK)
        .header(CONVERSATION_ID_HEADER_NAME, conversation.id)
        .body(result.text)
        .build())
}

fn handle_get_conversation(_req: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    match params.get("conversationId") {
        Some(id) => {
            if Conversation::exists(id) {
                let conversation = Conversation::load(id)?;
                Ok(Response::builder()
                    .status(http::StatusCode::OK)
                    .header("Content-Type", "application/json")
                    .body(conversation)
                    .build())
            } else {
                Ok(Response::new(404, "Conversation not found"))
            }
        },
        None => Ok(Response::new(404, "Coversation Id missing")),
    }
}

fn handle_readme(_req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let html = markdown::file_to_html(Path::new("README.md"))?;
    Ok(ResponseBuilder::new(http::StatusCode::OK)
        .header("Content-Type", "text/html")
        .body(html)
        .build())
}
