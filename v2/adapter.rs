//! Production sibling of v2_sdk, compiled into the host (not a plugin loader).
//! Uses the unchanged beta CoreContext and SDK. No OS operation is implemented here.
use super::v2_sdk as sdk;
use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    OnceLock,
};
use tezbar_custom_wordle_v2::{acknowledged, from_payload, generate, Generated, COMMAND};

static PROCESS: OnceLock<String> = OnceLock::new();
static NEXT: AtomicU64 = AtomicU64::new(1);

fn capability() -> Result<String, String> {
    let process = PROCESS.get_or_init(|| uuid::Uuid::new_v4().to_string());
    let number = NEXT
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| "Session capability resource limit reached")?;
    Ok(format!("wordle:{process}:{number}"))
}

pub(super) fn commands() -> &'static [&'static str] {
    &[COMMAND]
}

fn canonical(state: &Value) -> Result<Generated, String> {
    let word = state["word"]
        .as_str()
        .ok_or("Invalid Custom Wordle session")?;
    let generated = generate(word).map_err(|_| "Invalid Custom Wordle session")?;
    if state["url"] != generated.url
        || !state["copied"].is_boolean()
        || !state["opened"].is_boolean()
        || !state["cap"].is_string()
        || !matches!(state["pending"].as_str(), Some("" | "copy" | "open"))
    {
        return Err("Invalid Custom Wordle session".into());
    }
    Ok(generated)
}

fn render(state: &Value) -> Value {
    let copied = state["copied"] == true;
    let pending = state["pending"].as_str().unwrap_or("");
    let message = match pending {
        "copy" => "Clipboard copy was not confirmed. It may have completed. Choose Retry Copy Link to try again deliberately.",
        "open" => "Browser open was not confirmed. It may have completed. Choose Retry Open Link to try again deliberately.",
        _ if state["opened"] == true => "Link copied to clipboard. Browser open acknowledged.",
        _ if copied => "Link copied to clipboard",
        _ => "Link generated; clipboard copy has not been confirmed.",
    };
    let cap = state["cap"].as_str().unwrap_or("");
    let mut actions = Vec::new();
    if copied {
        actions.push(json!({"id":format!("{cap}:open"),"title":if pending == "open" {"Retry Open Link"} else {"Open Link"},"kind":"action"}));
    }
    actions.push(json!({"id":format!("{cap}:copy"),"title":if pending == "copy" {"Retry Copy Link"} else {"Copy Link"},"kind":"action"}));
    // URL is plain code, never a clickable markdown/metadata bypass of SDK acknowledgements.
    sdk::view(
        if copied {
            "Custom Wordle Created"
        } else {
            "Custom Wordle"
        },
        json!({"type":"Detail","props":{"markdown":format!("{message}\n\n`{}`",state["url"].as_str().unwrap_or(""))}}),
        actions,
    )
}

async fn effect(
    ctx: &CoreContext,
    state: &mut Value,
    kind: &str,
    generated: &Generated,
) -> CoreResult {
    // Checkpoint before awaiting: cancellation must not erase the uncertain effect or
    // any earlier positive acknowledgement. Refresh/search never repeat it.
    state["pending"] = json!(kind);
    let reply = if kind == "copy" {
        sdk::copy(ctx, &generated.url).await
    } else {
        sdk::open(ctx, &generated.url).await
    };
    if !reply.as_ref().is_ok_and(acknowledged) {
        return Err(if kind == "copy" {
            "Clipboard copy was not confirmed; retry only if you want another copy attempt."
        } else {
            "Browser open was not confirmed; retry only if you want another open attempt."
        }
        .into());
    }
    // Preserve acknowledgement even if issuing a new capability subsequently fails.
    state[if kind == "copy" { "copied" } else { "opened" }] = json!(true);
    state["pending"] = json!("");
    state["cap"] = json!(capability()?);
    Ok(render(state))
}

pub(super) async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    payload: &Value,
    state: &mut Value,
) -> CoreResult {
    if command != COMMAND {
        return Err("Unknown Custom Wordle command".into());
    }
    match operation {
        "extension:dispose-session" => {
            *state = json!({"disposed":true});
            Ok(sdk::unchanged())
        }
        "extension:run-command" => {
            if state.as_object().is_none_or(|s| !s.is_empty()) {
                return Err("Custom Wordle session already started".into());
            }
            let generated = from_payload(payload).map_err(|e| e.message().to_owned())?;
            *state = json!({"word":generated.word,"url":generated.url,"cap":capability()?,"copied":false,"opened":false,"pending":""});
            match effect(ctx, state, "copy", &generated).await {
                Ok(view) => Ok(view),
                // A launch must retain the session so the user can deliberately retry.
                Err(_) => Ok(render(state)),
            }
        }
        "extension:search-text-changed" | "extension:load-more" => {
            canonical(state)?;
            Ok(sdk::unchanged())
        }
        "extension:refresh-session" => {
            canonical(state)?;
            if state["pending"] != "" {
                Ok(render(state))
            } else {
                Ok(sdk::unchanged())
            }
        }
        "extension:invoke-action" => {
            let generated = canonical(state)?;
            // No form in the source. Reject malformed/nonempty callback values before
            // consuming a valid capability, and never read event-supplied URLs/outputs.
            if payload
                .get("formValues")
                .is_some_and(|v| !v.as_object().is_some_and(|o| o.is_empty()))
            {
                return Err("Custom Wordle actions do not accept form values".into());
            }
            let id = payload["actionId"]
                .as_str()
                .ok_or("Missing Custom Wordle action")?;
            let cap = state["cap"]
                .as_str()
                .ok_or("Invalid Custom Wordle session")?;
            let kind = if id == format!("{cap}:copy") {
                "copy"
            } else if id == format!("{cap}:open") && state["copied"] == true {
                "open"
            } else {
                return Err("Unknown or stale Custom Wordle action; reopen actions".into());
            };
            effect(ctx, state, kind, &generated).await
        }
        _ => Err("Unsupported Custom Wordle operation".into()),
    }
}
