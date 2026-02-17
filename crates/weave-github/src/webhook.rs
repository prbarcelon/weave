use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::merge::handle_pull_request;
use crate::AppState;

type HmacSha256 = Hmac<Sha256>;

/// Verify the webhook signature and dispatch the event.
///
/// Returns 200 immediately for valid webhooks. Merge processing
/// happens in a background tokio task.
pub async fn handle_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    // Verify signature
    let signature = match headers.get("x-hub-signature-256") {
        Some(v) => v.to_str().unwrap_or(""),
        None => return StatusCode::UNAUTHORIZED,
    };

    if !verify_signature(&state.config.webhook_secret, &body, signature) {
        return StatusCode::UNAUTHORIZED;
    }

    // Parse event type
    let event = match headers.get("x-github-event") {
        Some(v) => v.to_str().unwrap_or(""),
        None => return StatusCode::BAD_REQUEST,
    };

    if event != "pull_request" {
        return StatusCode::OK;
    }

    // Parse payload
    let payload: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST,
    };

    // Only handle opened, synchronize (new push), reopened
    let action = payload["action"].as_str().unwrap_or("");
    if !matches!(action, "opened" | "synchronize" | "reopened") {
        return StatusCode::OK;
    }

    // Extract fields we need
    let pr = match parse_pr_event(&payload) {
        Some(pr) => pr,
        None => return StatusCode::BAD_REQUEST,
    };

    // Spawn background task, return 200 immediately
    tokio::spawn(async move {
        if let Err(e) = handle_pull_request(&state, &pr).await {
            tracing::error!(
                repo = %pr.repo_full_name,
                pr = pr.pr_number,
                "merge processing failed: {e}"
            );
        }
    });

    StatusCode::OK
}

/// Parsed pull request event data.
pub struct PrEvent {
    pub installation_id: u64,
    pub repo_full_name: String,
    pub owner: String,
    pub repo: String,
    pub pr_number: u64,
    pub head_sha: String,
    pub base_sha: String,
    pub base_ref: String,
    pub head_ref: String,
}

fn parse_pr_event(payload: &serde_json::Value) -> Option<PrEvent> {
    let installation_id = payload["installation"]["id"].as_u64()?;
    let repo_full_name = payload["repository"]["full_name"].as_str()?.to_string();
    let pr = &payload["pull_request"];
    let pr_number = pr["number"].as_u64()?;
    let head_sha = pr["head"]["sha"].as_str()?.to_string();
    let base_sha = pr["base"]["sha"].as_str()?.to_string();
    let base_ref = pr["base"]["ref"].as_str()?.to_string();
    let head_ref = pr["head"]["ref"].as_str()?.to_string();

    let (owner, repo) = repo_full_name.split_once('/')?;
    let owner = owner.to_string();
    let repo = repo.to_string();

    Some(PrEvent {
        installation_id,
        repo_full_name,
        owner,
        repo,
        pr_number,
        head_sha,
        base_sha,
        base_ref,
        head_ref,
    })
}

fn verify_signature(secret: &str, body: &[u8], signature: &str) -> bool {
    let sig_hex = signature.strip_prefix("sha256=").unwrap_or(signature);

    let expected = match hex::decode(sig_hex) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);

    mac.verify_slice(&expected).is_ok()
}
