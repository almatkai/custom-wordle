use custom_wordle_beta_contract_check::native_core::{
    common::{CoreContext, CorePaths, CoreResult},
    extensions::{adapter, v2_sdk, Sessions},
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

type Calls = Arc<Mutex<Vec<(String, Vec<Value>)>>>;

#[tokio::test]
async fn actual_beta_catalog_and_registered_runtime_report_compiled_custom_wordle() {
    use custom_wordle_beta_contract_check::native_core::extensions::{
        catalog_install, catalog_installed, coverage, supports,
    };
    for id in [
        "custom-wordle",
        "raycast.custom-wordle",
        "com.tezbar.native.custom-wordle",
    ] {
        assert!(supports(id, "create-custom-wordle"));
        assert!(!supports(id, "unported-command"));
    }
    assert!(!supports("com.tezbar.native.other", "create-custom-wordle"));
    let manifest = coverage();
    assert_eq!(manifest["totalCommands"], 1);
    let available = cfg!(any(target_os = "macos", target_os = "windows"));
    assert_eq!(
        manifest["commands"][0]["runtime"],
        if available { "rust" } else { "unavailable" }
    );
    assert!(manifest["verification"]
        .as_array()
        .unwrap()
        .iter()
        .all(|v| v["state"] == "not-run"));
    let f = Fixture::new();
    assert!(!catalog_installed(&f.ctx));
    if available {
        let installed = catalog_install(&f.ctx).unwrap();
        assert_eq!(installed["id"], "com.tezbar.native.custom-wordle");
        assert!(catalog_installed(&f.ctx));
        let path = f
            .ctx
            .paths
            .data_dir
            .join("extensions/packages/com.tezbar.native.custom-wordle/native-v2.json");
        std::fs::write(path, b"{}").unwrap();
        assert!(!catalog_installed(&f.ctx));
    } else {
        assert!(catalog_install(&f.ctx).is_err());
    }
    assert_eq!(f.count(), 0);
}
struct Fixture {
    dir: tempfile::TempDir,
    ctx: CoreContext,
    sessions: Sessions,
    calls: Calls,
    reply: Arc<Mutex<CoreResult>>,
}
impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let ctx = context(dir.path());
        std::fs::create_dir_all(&ctx.paths.data_dir).unwrap();
        std::fs::write(
            ctx.paths.data_dir.join("package.json"),
            include_bytes!("../../manifest.json"),
        )
        .unwrap();
        let calls: Calls = Arc::default();
        let reply = Arc::new(Mutex::new(Ok(json!({"ok":true}))));
        let (record, response) = (calls.clone(), reply.clone());
        ctx.set_router(Arc::new(move |channel, args| {
            assert!(matches!(channel.as_str(), "clipboard:write" | "shell:open"));
            record.lock().unwrap().push((channel, args));
            let result = response.lock().unwrap().clone();
            Box::pin(async move { result })
        }))
        .unwrap();
        Self {
            dir,
            ctx,
            sessions: Sessions::default(),
            calls,
            reply,
        }
    }
    async fn call(&self, op: &str, body: Value) -> Value {
        self.sessions.call(&self.ctx, op, body).await
    }
    async fn launch(&self, args: Value) -> Value {
        self.call("extension:run-command",json!({"extensionId":"com.tezbar.native.custom-wordle","commandName":"create-custom-wordle","argumentValues":args})).await
    }
    async fn action(&self, view: &Value, index: usize) -> Value {
        self.call(
            "extension:invoke-action",
            json!({"sessionId":view["sessionId"],"actionId":view["actions"][index]["id"]}),
        )
        .await
    }
    fn count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}
fn context(dir: &std::path::Path) -> CoreContext {
    CoreContext::new(
        CorePaths {
            data_dir: dir.join("data"),
            config_dir: dir.join("config"),
            resource_dir: dir.join("resources"),
            temp_dir: dir.join("tmp"),
            home_dir: dir.into(),
        },
        None,
    )
}
fn payload() -> Value {
    json!({"argumentValues":{"word":" APPLE "}})
}

#[tokio::test]
async fn actual_sessions_copy_first_then_explicit_browser_and_bound_targets() {
    let f = Fixture::new();
    let v = f.launch(json!({"word":" APPLE "})).await;
    assert_eq!(v["mode"], "view");
    assert_eq!(v["root"]["type"], "Detail");
    assert_eq!(v["actions"][0]["title"], "Open Link");
    assert_eq!(v["actions"][1]["title"], "Copy Link");
    assert_eq!(
        *f.calls.lock().unwrap(),
        vec![(
            "clipboard:write".into(),
            vec![json!("https://mywordle.strivemath.com/?word=wdgop")]
        )]
    );
    let opened=f.call("extension:invoke-action",json!({"sessionId":v["sessionId"],"actionId":v["actions"][0]["id"],"url":"https://evil.invalid","output":"forged","argumentValues":{"word":"forged"}})).await;
    assert_eq!(opened["ok"], true);
    assert_eq!(
        f.calls.lock().unwrap()[1],
        (
            "shell:open".into(),
            vec![json!("https://mywordle.strivemath.com/?word=wdgop")]
        )
    );
    assert_eq!(f.action(&v, 0).await["ok"], false);
    assert_eq!(f.count(), 2);
    assert_eq!(f.action(&opened, 1).await["ok"], true);
    assert_eq!(f.count(), 3);
    let report = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../reports");
    std::fs::create_dir_all(&report).unwrap();
    std::fs::write(
        report.join("render.json"),
        serde_json::to_vec_pretty(&v).unwrap(),
    )
    .unwrap();
}

#[tokio::test]
async fn malformed_arguments_and_invalid_words_have_no_effects() {
    let f = Fixture::new();
    for args in [
        json!(null),
        json!([]),
        json!("apple"),
        json!({}),
        json!({"word":null}),
        json!({"word":false}),
        json!({"word":3}),
        json!({"word":[]}),
        json!({"word":""}),
        json!({"word":" \t\n"}),
        json!({"word":"two words"}),
        json!({"word":"emoji😀"}),
        json!({"word":"a+b"}),
    ] {
        let v = f.launch(args).await;
        assert_eq!(v["ok"], false);
        assert!(v.get("effects").is_none());
    }
    assert_eq!(f.count(), 0);
}

#[tokio::test]
async fn forged_cross_session_malformed_and_stale_ids_do_not_consume_actions() {
    let f = Fixture::new();
    let a = f.launch(json!({"word":"a"})).await;
    let b = f.launch(json!({"word":"b"})).await;
    for body in [
        json!({"sessionId":a["sessionId"]}),
        json!({"sessionId":a["sessionId"],"actionId":false}),
        json!({"sessionId":a["sessionId"],"actionId":"open"}),
        json!({"sessionId":a["sessionId"],"actionId":b["actions"][0]["id"]}),
    ] {
        assert_eq!(f.call("extension:invoke-action", body).await["ok"], false);
    }
    for bad in [
        json!(null),
        json!([]),
        json!(false),
        json!({"url":"forged"}),
    ] {
        assert_eq!(f.call("extension:invoke-action",json!({"sessionId":a["sessionId"],"actionId":a["actions"][0]["id"],"formValues":bad})).await["ok"],false);
    }
    assert_eq!(f.count(), 2);
    assert_eq!(f.action(&a, 0).await["ok"], true);
    assert_eq!(f.action(&a, 0).await["ok"], false);
    assert_eq!(f.action(&b, 0).await["ok"], true);
}

#[tokio::test]
async fn every_bad_copy_ack_is_uncertain_and_explicit_retry_only() {
    for response in [
        Ok(json!(null)),
        Ok(json!(true)),
        Ok(json!({})),
        Ok(json!({"ok":false})),
        Ok(json!({"ok":true,"success":false})),
        Ok(json!({"ok":true,"error":"secret"})),
        Ok(json!({"ok":true,"success":null})),
        Err("secret private path".into()),
    ] {
        let f = Fixture::new();
        *f.reply.lock().unwrap() = response;
        let v = f.launch(json!({"word":"apple"})).await;
        assert_eq!(v["mode"], "view");
        assert_eq!(v["actions"].as_array().unwrap().len(), 1);
        assert_eq!(v["actions"][0]["title"], "Retry Copy Link");
        assert!(!v.to_string().contains("secret"));
        assert!(!v.to_string().contains("Link copied to clipboard"));
        for op in [
            "extension:refresh-session",
            "extension:search-text-changed",
            "extension:load-more",
        ] {
            f.call(op, json!({"sessionId":v["sessionId"]})).await;
        }
        assert_eq!(f.count(), 1);
        let failure = f.action(&v, 0).await;
        assert_eq!(failure["ok"], false);
        assert!(!failure.to_string().contains("secret"));
        assert_eq!(f.count(), 2);
        *f.reply.lock().unwrap() = Ok(json!({"ok":true,"error":null,"success":true}));
        let recovered = f.action(&v, 0).await;
        assert_eq!(recovered["actions"][0]["title"], "Open Link");
        assert_eq!(f.count(), 3);
    }
}

#[tokio::test]
async fn every_bad_open_ack_preserves_confirmed_copy_and_sanitizes() {
    for response in [
        Ok(json!(null)),
        Ok(json!(false)),
        Ok(json!({"ok":true,"error":false})),
        Ok(json!({"ok":true,"success":false})),
        Err("secret browser path".into()),
    ] {
        let f = Fixture::new();
        let v = f.launch(json!({"word":"apple"})).await;
        *f.reply.lock().unwrap() = response;
        let failed = f.action(&v, 0).await;
        assert_eq!(failed["ok"], false);
        assert!(!failed.to_string().contains("secret"));
        let refreshed = f
            .call(
                "extension:refresh-session",
                json!({"sessionId":v["sessionId"]}),
            )
            .await;
        assert_eq!(refreshed["actions"][0]["title"], "Retry Open Link");
        assert_eq!(f.count(), 2);
        *f.reply.lock().unwrap() = Ok(json!({"ok":true}));
        assert_eq!(f.action(&refreshed, 0).await["ok"], true);
        assert_eq!(f.count(), 3);
        assert_eq!(
            f.calls
                .lock()
                .unwrap()
                .iter()
                .filter(|(c, _)| c == "clipboard:write")
                .count(),
            1
        );
    }
}

#[tokio::test]
async fn idle_refresh_search_load_and_discarded_clones_do_not_issue_capabilities() {
    let f = Fixture::new();
    let mut s = json!({});
    adapter(&f.ctx, "extension:run-command", &payload(), &mut s)
        .await
        .unwrap();
    let before = s.clone();
    for op in [
        "extension:refresh-session",
        "extension:search-text-changed",
        "extension:load-more",
    ] {
        let mut clone = s.clone();
        let result = adapter(&f.ctx, op, &json!({"searchText":"forged"}), &mut clone)
            .await
            .unwrap();
        assert_eq!(result, json!({"ok":true,"mode":"unchanged"}));
        assert_eq!(clone, before);
    }
    assert_eq!(f.count(), 1);
    let mut stale = s.clone();
    adapter(&f.ctx, "extension:dispose-session", &json!({}), &mut stale)
        .await
        .unwrap();
    assert!(adapter(
        &f.ctx,
        "extension:invoke-action",
        &json!({"actionId":format!("{}:copy",before["cap"].as_str().unwrap())}),
        &mut stale
    )
    .await
    .is_err());
    let mut new = json!({});
    adapter(&f.ctx, "extension:run-command", &payload(), &mut new)
        .await
        .unwrap();
    assert_ne!(new["cap"], s["cap"]);
}

#[tokio::test]
async fn cancellation_checkpoints_before_await_without_automatic_retry() {
    let f = Fixture::new();
    let mut state = json!({});
    let v = adapter(&f.ctx, "extension:run-command", &payload(), &mut state)
        .await
        .unwrap();
    let ctx = context(f.dir.path());
    let calls: Calls = Arc::default();
    let record = calls.clone();
    ctx.set_router(Arc::new(move |channel, args| {
        record.lock().unwrap().push((channel, args));
        Box::pin(std::future::pending())
    }))
    .unwrap();
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(10),
        adapter(
            &ctx,
            "extension:invoke-action",
            &json!({"actionId":v["actions"][0]["id"]}),
            &mut state,
        ),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(state["copied"], true);
    assert_eq!(state["opened"], false);
    assert_eq!(state["pending"], "open");
    for _ in 0..3 {
        let mut clone = state.clone();
        let result = adapter(&ctx, "extension:refresh-session", &json!({}), &mut clone)
            .await
            .unwrap();
        assert_eq!(result["actions"][0]["title"], "Retry Open Link");
        assert_eq!(clone, state);
    }
    assert_eq!(calls.lock().unwrap().len(), 1);
    assert!(
        adapter(&f.ctx, "extension:run-command", &payload(), &mut state)
            .await
            .is_err()
    );
    let mut copy_state = json!({});
    assert!(tokio::time::timeout(
        std::time::Duration::from_millis(10),
        adapter(&ctx, "extension:run-command", &payload(), &mut copy_state)
    )
    .await
    .is_err());
    assert_eq!(copy_state["word"], "apple");
    assert_eq!(
        copy_state["url"],
        "https://mywordle.strivemath.com/?word=wdgop"
    );
    assert_eq!(copy_state["copied"], false);
    assert_eq!(copy_state["pending"], "copy");
    let recovered = adapter(
        &ctx,
        "extension:refresh-session",
        &json!({}),
        &mut copy_state,
    )
    .await
    .unwrap();
    assert_eq!(recovered["actions"].as_array().unwrap().len(), 1);
    assert_eq!(recovered["actions"][0]["title"], "Retry Copy Link");
    assert_eq!(calls.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn actual_native_sessions_dispose_clear_uninstall_and_stale_run() {
    let f = Fixture::new();
    let a = f.launch(json!({"word":"a"})).await;
    assert_eq!(
        f.call(
            "extension:dispose-session",
            json!({"sessionId":a["sessionId"]})
        )
        .await,
        true
    );
    assert_eq!(
        f.call(
            "extension:dispose-session",
            json!({"sessionId":a["sessionId"]})
        )
        .await,
        false
    );
    assert_eq!(f.action(&a, 0).await["ok"], false);
    let b = f.launch(json!({"word":"b"})).await;
    f.sessions.clear_slug().await;
    assert_eq!(f.action(&b, 0).await["ok"], false);
    let c = f.launch(json!({"word":"c"})).await;
    f.sessions.clear().await;
    assert_eq!(f.action(&c, 0).await["ok"], false);
    let d = f.launch(json!({"word":"d"})).await;
    std::fs::remove_file(f.ctx.paths.data_dir.join("package.json")).unwrap();
    assert_eq!(
        f.call(
            "extension:refresh-session",
            json!({"sessionId":d["sessionId"]})
        )
        .await["mode"],
        "unchanged"
    );
    assert_eq!(f.action(&d, 0).await["ok"], false);
    for channel in [
        "extension:run-command",
        "extension:invoke-action",
        "extension:refresh-session",
    ] {
        assert_eq!(f.call(channel,json!({"sessionId":"native-extension:missing","extensionId":"com.tezbar.native.custom-wordle","commandName":"create-custom-wordle"})).await["ok"],false);
    }
    assert_eq!(f.count(), 4);
}

#[tokio::test]
async fn actual_sessions_shutdown_cancels_pending_browser_without_hanging() {
    let f = Fixture::new();
    let v = f.launch(json!({"word":"a"})).await;
    let blocked = context(f.dir.path());
    let entered = Arc::new(tokio::sync::Notify::new());
    let signal = entered.clone();
    blocked
        .set_router(Arc::new(move |channel, _| {
            assert_eq!(channel, "shell:open");
            let signal = signal.clone();
            Box::pin(async move {
                signal.notify_one();
                std::future::pending().await
            })
        }))
        .unwrap();
    let sessions = f.sessions.clone();
    let body = json!({"sessionId":v["sessionId"],"actionId":v["actions"][0]["id"]});
    let task = tokio::spawn(async move {
        sessions
            .call(&blocked, "extension:invoke-action", body)
            .await
    });
    entered.notified().await;
    tokio::time::timeout(std::time::Duration::from_secs(2), f.sessions.clear())
        .await
        .unwrap();
    assert_eq!(task.await.unwrap()["ok"], false);
    assert_eq!(f.action(&v, 0).await["ok"], false);
}

#[tokio::test]
async fn source_has_no_store_and_known_state_corruption_never_reaches_router() {
    let f = Fixture::new();
    let mut s = json!({});
    adapter(&f.ctx, "extension:run-command", &payload(), &mut s)
        .await
        .unwrap();
    assert!(!f.ctx.paths.data_dir.join("native-extensions-v2").exists());
    for (key, value) in [
        ("url", json!("https://evil.invalid")),
        ("word", json!("bad word")),
        ("copied", json!("true")),
        ("opened", json!(null)),
        ("cap", json!(false)),
        ("pending", json!("paste")),
    ] {
        let mut bad = s.clone();
        bad[key] = value;
        assert!(adapter(
            &f.ctx,
            "extension:invoke-action",
            &json!({"actionId":format!("{}:open",s["cap"].as_str().unwrap())}),
            &mut bad
        )
        .await
        .is_err());
    }
    assert_eq!(f.count(), 1);
}

#[tokio::test]
async fn actual_sdk_storage_process_atomic_unknown_keys_abort_and_corruption() {
    let f = Fixture::new();
    v2_sdk::update_store(&f.ctx, "fixture", |v| {
        *v = json!({"count":0,"unknown":{"keep":true}});
        Ok(())
    })
    .await
    .unwrap();
    let mut tasks = Vec::new();
    for _ in 0..30 {
        let ctx = f.ctx.clone();
        tasks.push(tokio::spawn(async move {
            v2_sdk::update_store(&ctx, "fixture", |v| {
                v["count"] = json!(v["count"].as_u64().ok_or("invalid count")? + 1);
                Ok(())
            })
            .await
            .unwrap()
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    assert_eq!(
        v2_sdk::read_store(&f.ctx, "fixture").await.unwrap(),
        json!({"count":30,"unknown":{"keep":true}})
    );
    assert!(v2_sdk::update_store(&f.ctx, "fixture", |v| {
        v["count"] = json!(999);
        Err("abort".into())
    })
    .await
    .is_err());
    let path = f
        .ctx
        .paths
        .data_dir
        .join("native-extensions-v2/fixture/storage.json");
    for bytes in [b"{broken".as_slice(), b"[]".as_slice()] {
        std::fs::write(&path, bytes).unwrap();
        assert!(v2_sdk::read_store(&f.ctx, "fixture").await.is_err());
        assert!(v2_sdk::update_store(&f.ctx, "fixture", |_| Ok(()))
            .await
            .is_err());
        assert_eq!(std::fs::read(&path).unwrap(), bytes);
    }
    for slug in ["", "..", "../escape", "a/b", "a\\b"] {
        assert!(v2_sdk::read_store(&f.ctx, slug).await.is_err());
    }
}
