mod prompts;

use axum::{
    response::{AppendHeaders, IntoResponse},
    routing::post,
    Json, Router,
};
use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use prompts::{
    DIAGNOSTIC_GROUPS_PLACEHOLDER, EVENT_PLACEHOLDER, INPUT_PLACEHOLDER, OUTLINE_PLACEHOLDER,
    PROMPT, SPECULATED_OUTPUT_PLACEHOLDER,
};
use serde::{Deserialize, Serialize};
// use tokio::io::{stdout, AsyncWriteExt};
// use tokio_stream::StreamExt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictEditsBody {
    pub outline: Option<String>,
    pub input_events: String,
    pub input_excerpt: String,
    pub speculated_output: Option<String>,
    #[serde(default)]
    pub diagnostic_groups: Option<Vec<(String, serde_json::Value)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictEditsResponse {
    pub request_id: Uuid,
    pub output_excerpt: String,
}

async fn predict_edits_v2(Json(payload): Json<PredictEditsBody>) -> impl IntoResponse {
    let prompt = PROMPT
        .replace(EVENT_PLACEHOLDER, &payload.input_events)
        .replace(INPUT_PLACEHOLDER, &payload.input_excerpt)
        .replace(
            OUTLINE_PLACEHOLDER,
            payload.outline.as_deref().unwrap_or_default(),
        )
        .replace(
            SPECULATED_OUTPUT_PLACEHOLDER,
            payload.speculated_output.as_deref().unwrap_or_default(),
        )
        .replace(
            DIAGNOSTIC_GROUPS_PLACEHOLDER,
            "", // payload.diagnostic_groups.unwrap_or_default().join("\n"),
        );

    let ollama = Ollama::default();
    let res = ollama
        .generate(GenerationRequest::new("zeta".to_string(), prompt))
        .await
        .unwrap();

    // let mut stream = ollama
    //     .generate_stream(GenerationRequest::new("zeta".to_string(), prompt))
    //     .await
    //     .unwrap();

    // let mut stdout = stdout();
    // while let Some(res) = stream.next().await {
    //     let responses = res.unwrap();
    //     for resp in responses {
    //         stdout.write_all(resp.response.as_bytes()).await.unwrap();
    //         stdout.flush().await.unwrap();
    //     }
    // }

    (
        AppendHeaders([("x-zed-minimum-required-version", "0.173.5")]),
        Json(PredictEditsResponse {
            request_id: Uuid::new_v4(),
            output_excerpt: res.response,
        }),
    )
}

#[tokio::main]
async fn main() {
    // initialize tracing
    // tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `POST /users` goes to `create_user`
        .route("/predict_edits/v2", post(predict_edits_v2));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
