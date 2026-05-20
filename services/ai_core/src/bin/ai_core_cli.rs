use aetheris_ai_core::agent::{PolicyEnforcer, StepExecutor, TaskPlanner};
use aetheris_ai_core::cap::CapTokenManager;
use aetheris_ai_core::intents::{create_system_intent_manager, SystemActionContext};
use aetheris_ai_core::ipc::ChatRequest;
use aetheris_ai_core::orchestrator::{MultiModalOrchestrator, OrchestratorInput};
use aetheris_ai_core::router::PromptRouter;
use aetheris_ai_core::runtime::{AcceleratorKind, HegPolicy, RuntimeManager, RuntimeRequest};
use aetheris_ai_core::tools::registry::ToolRegistry;
use aetheris_ai_core::tools::stt::{SpeechToTextTool, SttConfig};
use std::collections::HashMap;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("plan-smoke") {
        return run_plan_smoke(&args[2..]).await;
    }
    if args.get(1).map(String::as_str) == Some("summarize-log") {
        return run_summarize_log(&args[2..]).await;
    }
    if args.get(1).map(String::as_str) == Some("goal-plan") {
        return run_goal_plan(&args[2..]).await;
    }
    if args.get(1).map(String::as_str) == Some("record-outcome") {
        return run_record_outcome(&args[2..]).await;
    }
    if args.get(1).map(String::as_str) == Some("replay-goal") {
        return run_replay_goal(&args[2..]).await;
    }
    if args.get(1).map(String::as_str) == Some("browser-summarize") {
        return run_browser_summarize(&args[2..]).await;
    }
    if args.get(1).map(String::as_str) == Some("browser-classify") {
        return run_browser_classify(&args[2..]).await;
    }
    if args.get(1).map(String::as_str) == Some("runtime-plan") {
        return run_runtime_plan(&args[2..]).await;
    }

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let message = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&buffer) {
        json["message"].as_str().unwrap_or(&buffer).to_string()
    } else {
        buffer.trim().to_string()
    };

    println!("Processing message: {}", message);

    let runtime_manager = Arc::new(RuntimeManager::new_mock());
    let tool_registry = Arc::new(ToolRegistry::new_mock());
    let cap_manager = Arc::new(CapTokenManager::new_mock());
    let prompt_router = Arc::new(PromptRouter::new(
        std::path::PathBuf::from("templates"),
        true,
    ));
    let system_intent_manager = Arc::new(create_system_intent_manager(cap_manager.clone())?);
    let task_planner = Arc::new(TaskPlanner::new(
        runtime_manager.clone(),
        tool_registry.clone(),
    ));
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_manager.clone()));
    let step_executor = Arc::new(StepExecutor::new(
        tool_registry.clone(),
        policy_enforcer.clone(),
    ));
    let mut stt = SpeechToTextTool::new(SttConfig::default())?;
    stt.initialize(None).await?;
    let stt_tool = Arc::new(stt);

    let (tx, rx) = mpsc::channel(32);
    let orchestrator = MultiModalOrchestrator::new(
        rx,
        prompt_router,
        system_intent_manager,
        runtime_manager,
        stt_tool,
        task_planner,
        step_executor,
    );

    tokio::spawn(async move {
        orchestrator.run().await;
    });

    let (resp_tx, resp_rx) = oneshot::channel();
    let request = ChatRequest {
        message,
        ..Default::default()
    };

    tx.send(OrchestratorInput::Text {
        request,
        session_id: "cli-session".to_string(),
        response_tx: resp_tx,
    })
    .await?;

    println!("Waiting for response...");
    let response = resp_rx.await??;
    println!("Response: {:?}", response);

    Ok(())
}

async fn run_summarize_log(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut log_path = None;
    let mut component_path = None;
    let mut metrics_js = None;
    let mut max_bytes = 2 * 1024 * 1024u64;
    let mut approved = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--path" => log_path = iter.next().map(PathBuf::from),
            "--component" => component_path = iter.next().map(PathBuf::from),
            "--metrics-js" => metrics_js = iter.next().map(PathBuf::from),
            "--max-bytes" => {
                let value = iter.next().ok_or("--max-bytes requires a numeric value")?;
                max_bytes = value.parse()?;
            }
            "--approve" => approved = true,
            other => {
                if log_path.is_none() {
                    log_path = Some(PathBuf::from(other));
                }
            }
        }
    }

    let log_path = log_path.ok_or("summarize-log requires --path <log-file>")?;
    let runtime_manager = Arc::new(RuntimeManager::new_mock());
    let tool_registry = Arc::new(ToolRegistry::new_mock());
    let cap_manager = Arc::new(CapTokenManager::new_mock());
    let planner = TaskPlanner::new(runtime_manager, tool_registry.clone());
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_manager));
    let executor = StepExecutor::new(tool_registry, policy_enforcer);

    let mut metadata = HashMap::new();
    if approved {
        metadata.insert("approved_plan".to_string(), serde_json::json!(true));
        metadata.insert(
            "capabilities".to_string(),
            serde_json::json!(["fs.read", "ai.summarize"]),
        );
    }
    let context = SystemActionContext {
        user_id: "cli-user".to_string(),
        session_id: "cli-log-summary".to_string(),
        cap_token: None,
        metadata,
    };

    let plan = planner
        .generate_log_summary_plan(&log_path, max_bytes, component_path.as_deref(), &context)
        .await?;
    eprintln!("{}", serde_json::to_string_pretty(&plan)?);

    let results = executor.execute_plan(plan, context).await?;
    let payload = results
        .iter()
        .find_map(|step| step.result.as_ref())
        .and_then(|result| result.result.as_ref())
        .ok_or("log summarizer completed without a result payload")?;
    let summary: serde_json::Value = serde_json::from_slice(payload)?;
    let metrics_path = metrics_js.unwrap_or_else(default_dashboard_metrics_path);
    let total = write_dashboard_metric(
        &metrics_path,
        "ai_log_summaries_total",
        "last_summary",
        &summary,
    )
    .await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ai_log_summaries_total": total,
            "summary": summary,
            "dashboard_metric": metrics_path,
        }))?
    );

    Ok(())
}

async fn run_goal_plan(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut goal = None;
    let mut ltm_path = PathBuf::from("data/aicore_ltm.ndjson");
    let mut metrics_js = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--goal" => goal = iter.next().cloned(),
            "--ltm-path" => ltm_path = PathBuf::from(iter.next().ok_or("--ltm-path requires a path")?),
            "--metrics-js" => metrics_js = iter.next().map(PathBuf::from),
            other => {
                if goal.is_none() {
                    goal = Some(other.to_string());
                }
            }
        }
    }

    let goal = goal.ok_or("goal-plan requires --goal <text>")?;
    let result = aetheris_ai_core::cognitive::generate_cognitive_goal_plan(&goal, &ltm_path).await?;
    aetheris_ai_core::metrics::record_ai_plan_generated();
    let metrics_path = metrics_js.unwrap_or_else(default_dashboard_metrics_path);
    let total = write_dashboard_metric(
        &metrics_path,
        "ai_plans_generated_total",
        "last_goal_plan",
        &serde_json::to_value(&result.projection)?,
    )
    .await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ai_plans_generated_total": total,
            "plan": result.plan,
            "projection": result.projection,
            "dashboard_metric": metrics_path,
        }))?
    );
    Ok(())
}

async fn run_record_outcome(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut goal = None;
    let mut outcome = None;
    let mut notes = None;
    let mut ltm_path = PathBuf::from("data/aicore_ltm.ndjson");
    let mut metrics_js = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--goal" => goal = iter.next().cloned(),
            "--outcome" => outcome = iter.next().cloned(),
            "--notes" => notes = iter.next().cloned(),
            "--ltm-path" => ltm_path = PathBuf::from(iter.next().ok_or("--ltm-path requires a path")?),
            "--metrics-js" => metrics_js = iter.next().map(PathBuf::from),
            other => {
                if goal.is_none() {
                    goal = Some(other.to_string());
                }
            }
        }
    }

    let goal = goal.ok_or("record-outcome requires --goal <text>")?;
    let outcome = outcome.ok_or("record-outcome requires --outcome <code>")?;
    let event =
        aetheris_ai_core::cognitive::record_learning_event(&goal, &outcome, notes, &ltm_path)
            .await?;
    let metrics_path = metrics_js.unwrap_or_else(default_dashboard_metrics_path);
    let total = write_dashboard_metric(
        &metrics_path,
        "ai_ltm_events_total",
        "last_ltm_event",
        &serde_json::to_value(&event)?,
    )
    .await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ai_ltm_events_total": total,
            "event": event,
            "dashboard_metric": metrics_path,
        }))?
    );
    Ok(())
}

async fn run_replay_goal(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut goal = None;
    let mut ltm_path = PathBuf::from("data/aicore_ltm.ndjson");

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--goal" => goal = iter.next().cloned(),
            "--ltm-path" => ltm_path = PathBuf::from(iter.next().ok_or("--ltm-path requires a path")?),
            other => {
                if goal.is_none() {
                    goal = Some(other.to_string());
                }
            }
        }
    }

    let goal = goal.ok_or("replay-goal requires --goal <text>")?;
    let proof = aetheris_ai_core::cognitive::replay_goal(&goal, &ltm_path).await?;
    println!("{}", serde_json::to_string_pretty(&proof)?);
    Ok(())
}

async fn run_browser_summarize(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut input = None;
    let mut max_chars = 800u64;
    let mut approved = false;
    let mut metrics_js = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--input" => input = iter.next().cloned(),
            "--max-chars" => max_chars = iter.next().ok_or("--max-chars requires a value")?.parse()?,
            "--metrics-js" => metrics_js = iter.next().map(PathBuf::from),
            "--approve" => approved = true,
            other => {
                if input.is_none() {
                    input = Some(other.to_string());
                }
            }
        }
    }

    let input = input.ok_or("browser-summarize requires --input <file-or-text>")?;
    let mut params = serde_json::json!({
        "max_chars": max_chars,
        "requires_remote": false,
        "budget": {"cpu_percent": 5, "memory_mb": 64, "power_mw": 100}
    });
    let mut caps = vec!["ai.browser.summarize".to_string()];
    if Path::new(&input).exists() {
        params["path"] = serde_json::json!(input);
        caps.push("fs.read".to_string());
    } else {
        params["text"] = serde_json::json!(input);
    }

    let metrics_path = metrics_js.unwrap_or_else(default_dashboard_metrics_path);
    let output = match execute_single_tool_plan(
        "browser_summarize",
        "summarize local browser/page content",
        params,
        caps,
        approved,
        vec!["ai.browser.summarize", "fs.read"],
    )
    .await
    {
        Ok(output) => output,
        Err(error) => {
            let _ = write_dashboard_metric(
                &metrics_path,
                "ai_capability_denials_total",
                "last_capability_denial",
                &serde_json::json!({
                    "command": "browser-summarize",
                    "reason": error.to_string(),
                }),
            )
            .await;
            return Err(error);
        }
    };
    let total = write_dashboard_metric(
        &metrics_path,
        "ai_browser_summaries_total",
        "last_browser_summary",
        &output,
    )
    .await?;

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ai_browser_summaries_total": total,
            "summary": output,
            "dashboard_metric": metrics_path,
        }))?
    );
    Ok(())
}

async fn run_browser_classify(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut url = None;
    let mut text = None;
    let mut approved = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--url" => url = iter.next().cloned(),
            "--text" => text = iter.next().cloned(),
            "--approve" => approved = true,
            other => {
                if url.is_none() {
                    url = Some(other.to_string());
                }
            }
        }
    }

    let url = url.ok_or("browser-classify requires --url <url>")?;
    let mut params = serde_json::json!({
        "url": url,
        "requires_remote": false,
        "budget": {"cpu_percent": 5, "memory_mb": 64, "power_mw": 100}
    });
    let mut caps = vec!["ai.browser.classify".to_string()];
    if let Some(text) = text {
        if Path::new(&text).exists() {
            params["path"] = serde_json::json!(text);
            caps.push("fs.read".to_string());
        } else {
            params["text"] = serde_json::json!(text);
        }
    }

    let output = match execute_single_tool_plan(
        "browser_classify_page",
        "classify local browser/page content",
        params,
        caps,
        approved,
        vec!["ai.browser.classify", "fs.read"],
    )
    .await
    {
        Ok(output) => output,
        Err(error) => {
            let _ = write_dashboard_metric(
                &default_dashboard_metrics_path(),
                "ai_capability_denials_total",
                "last_capability_denial",
                &serde_json::json!({
                    "command": "browser-classify",
                    "reason": error.to_string(),
                }),
            )
            .await;
            return Err(error);
        }
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn run_runtime_plan(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut prompt = None;
    let mut prefer = None;
    let mut metrics_js = None;
    let mut workload = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--prompt" => prompt = iter.next().cloned(),
            "--prefer" => prefer = Some(parse_accelerator(iter.next().ok_or("--prefer requires cpu|igpu|npu")?)?),
            "--dashboard-metrics" | "--metrics-js" => metrics_js = iter.next().map(PathBuf::from),
            "--workload" => workload = iter.next().cloned(),
            other => {
                if prompt.is_none() {
                    prompt = Some(other.to_string());
                }
            }
        }
    }

    let prompt = prompt.ok_or("runtime-plan requires --prompt <text>")?;
    let mut metadata = HashMap::new();
    if let Some(workload) = workload {
        metadata.insert("workload".to_string(), workload);
    }
    let request = RuntimeRequest {
        prompt,
        max_tokens: Some(128),
        temperature: Some(0.0),
        stop_sequences: Vec::new(),
        metadata,
    };
    let policy = HegPolicy {
        preferred_accelerator: prefer,
        ..HegPolicy::default()
    };
    let runtime = RuntimeManager::new_mock();
    let plan = runtime.plan_execution_graph_with_policy(&request, &policy)?;
    aetheris_ai_core::metrics::record_heg_plan(&plan);

    let metrics_path = metrics_js.unwrap_or_else(default_dashboard_metrics_path);
    let total = write_dashboard_metric(
        &metrics_path,
        "ai_heg_plans_total",
        "last_heg_plan",
        &serde_json::to_value(&plan)?,
    )
    .await?;
    write_dashboard_metric_value(
        &metrics_path,
        "ai_heg_ddr_pressure_score",
        serde_json::json!(plan.ddr_pressure_score),
    )
    .await?;
    if plan.nodes.iter().any(|node| {
        node.operator == aetheris_ai_core::runtime::OperatorKind::Prefill
            && node.assigned == AcceleratorKind::Npu
    }) {
        let _ = write_dashboard_metric(
            &metrics_path,
            "ai_heg_prefill_to_npu_total",
            "last_heg_plan",
            &serde_json::to_value(&plan)?,
        )
        .await?;
    }
    if plan.nodes.iter().any(|node| {
        node.operator == aetheris_ai_core::runtime::OperatorKind::Decode
            && node.assigned == AcceleratorKind::Igpu
    }) {
        let _ = write_dashboard_metric(
            &metrics_path,
            "ai_heg_decode_to_igpu_total",
            "last_heg_plan",
            &serde_json::to_value(&plan)?,
        )
        .await?;
    }

    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "ai_heg_plans_total": total,
            "dashboard_metric": metrics_path,
            "plan": plan,
        }))?
    );
    Ok(())
}

fn parse_accelerator(value: &str) -> Result<AcceleratorKind, Box<dyn std::error::Error>> {
    match value.to_ascii_lowercase().as_str() {
        "cpu" => Ok(AcceleratorKind::Cpu),
        "igpu" | "gpu" => Ok(AcceleratorKind::Igpu),
        "npu" => Ok(AcceleratorKind::Npu),
        _ => Err(format!("unsupported accelerator '{value}', expected cpu|igpu|npu").into()),
    }
}

fn default_dashboard_metrics_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../ui/dashboard/ai_metrics.js")
}

async fn execute_single_tool_plan(
    tool_name: &str,
    description: &str,
    parameters: serde_json::Value,
    required_capabilities: Vec<String>,
    approved: bool,
    approval_capabilities: Vec<&str>,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let tool_registry = Arc::new(ToolRegistry::new_mock());
    let cap_manager = Arc::new(CapTokenManager::new_mock());
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_manager));
    let executor = StepExecutor::new(tool_registry, policy_enforcer);

    let plan = aetheris_ai_core::agent::TaskPlan {
        plan_id: format!("{}-cli", tool_name.replace('_', "-")),
        original_intent: description.to_string(),
        steps: vec![aetheris_ai_core::agent::TaskStep {
            step_id: "step-001".to_string(),
            description: description.to_string(),
            tool_name: tool_name.to_string(),
            parameters,
            required_capabilities,
            requires_approval: true,
            is_destructive: false,
            depends_on: Vec::new(),
            estimated_time_secs: 2,
        }],
        total_estimated_time_secs: 2,
        metadata: HashMap::new(),
        created_at: "cli-deterministic".to_string(),
    };

    let mut metadata = HashMap::new();
    if approved {
        metadata.insert("approved_plan".to_string(), serde_json::json!(true));
        metadata.insert(
            "capabilities".to_string(),
            serde_json::json!(approval_capabilities),
        );
    }
    let context = SystemActionContext {
        user_id: "cli-user".to_string(),
        session_id: "cli-browser-assist".to_string(),
        cap_token: None,
        metadata,
    };

    let results = executor.execute_plan(plan, context).await?;
    let payload = results
        .iter()
        .find_map(|step| step.result.as_ref())
        .and_then(|result| result.result.as_ref())
        .ok_or("browser tool completed without a result payload")?;
    Ok(serde_json::from_slice(payload)?)
}

async fn write_dashboard_metric(
    path: &Path,
    metric_name: &str,
    last_key: &str,
    payload_value: &serde_json::Value,
) -> Result<u64, Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut payload = read_dashboard_metrics(path).await.unwrap_or_else(default_dashboard_metrics);
    let previous = payload
        .get(metric_name)
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let total = previous.saturating_add(1);
    let updated_at_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    payload[metric_name] = serde_json::json!(total);
    payload["updated_at_unix"] = serde_json::json!(updated_at_unix);
    payload[last_key] = payload_value.clone();
    let js = format!(
        "window.POLYMERA_AI_METRICS = {};\n",
        serde_json::to_string_pretty(&payload)?
    );
    tokio::fs::write(path, js).await?;
    Ok(total)
}

async fn write_dashboard_metric_value(
    path: &Path,
    metric_name: &str,
    value: serde_json::Value,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut payload = read_dashboard_metrics(path).await.unwrap_or_else(default_dashboard_metrics);
    let updated_at_unix = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    payload[metric_name] = value;
    payload["updated_at_unix"] = serde_json::json!(updated_at_unix);
    let js = format!(
        "window.POLYMERA_AI_METRICS = {};\n",
        serde_json::to_string_pretty(&payload)?
    );
    tokio::fs::write(path, js).await?;
    Ok(())
}

fn default_dashboard_metrics() -> serde_json::Value {
    serde_json::json!({
        "ai_log_summaries_total": 0,
        "ai_plans_generated_total": 0,
        "ai_ltm_events_total": 0,
        "ai_browser_summaries_total": 0,
        "ai_capability_denials_total": 0,
        "ai_heg_plans_total": 0,
        "ai_heg_prefill_to_npu_total": 0,
        "ai_heg_decode_to_igpu_total": 0,
        "ai_heg_ddr_pressure_score": 0,
        "updated_at_unix": 0,
        "last_summary": null,
        "last_goal_plan": null,
        "last_ltm_event": null,
        "last_browser_summary": null,
        "last_heg_plan": null,
    })
}

async fn read_dashboard_metrics(path: &Path) -> Option<serde_json::Value> {
    let text = tokio::fs::read_to_string(path).await.ok()?;
    let start = text.find('{')?;
    let end = text.rfind('}')?;
    let mut json: serde_json::Value = serde_json::from_str(&text[start..=end]).ok()?;
    let defaults = default_dashboard_metrics();
    for key in [
        "ai_log_summaries_total",
        "ai_plans_generated_total",
        "ai_ltm_events_total",
        "ai_browser_summaries_total",
        "ai_capability_denials_total",
        "ai_heg_plans_total",
        "ai_heg_prefill_to_npu_total",
        "ai_heg_decode_to_igpu_total",
        "ai_heg_ddr_pressure_score",
        "updated_at_unix",
    ] {
        if json.get(key).is_none() {
            json[key] = defaults[key].clone();
        }
    }
    Some(json)
}

async fn run_plan_smoke(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut goal = None;
    let mut approved = false;
    let mut execute = false;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--goal" => goal = iter.next().cloned(),
            "--approve" => approved = true,
            "--execute" => execute = true,
            other => {
                if goal.is_none() {
                    goal = Some(other.to_string());
                }
            }
        }
    }

    let goal = goal.unwrap_or_else(|| "scan files then summarize results".to_string());
    let runtime_manager = Arc::new(RuntimeManager::new_mock());
    let tool_registry = Arc::new(ToolRegistry::new_mock());
    let cap_manager = Arc::new(CapTokenManager::new_mock());
    let planner = TaskPlanner::new(runtime_manager, tool_registry.clone());
    let policy_enforcer = Arc::new(PolicyEnforcer::new(cap_manager));
    let executor = StepExecutor::new(tool_registry, policy_enforcer);

    let mut metadata = HashMap::new();
    if approved {
        metadata.insert("approved_plan".to_string(), serde_json::json!(true));
    }
    let context = SystemActionContext {
        user_id: "cli-user".to_string(),
        session_id: "cli-session".to_string(),
        cap_token: None,
        metadata,
    };

    let plan = planner.generate_plan(&goal, &context).await?;
    println!("{}", serde_json::to_string_pretty(&plan)?);

    if execute {
        let results = executor.execute_plan(plan, context).await?;
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else if plan_requires_approval(&plan) && !approved {
        println!("plan requires approval; re-run with --approve --execute to run it");
    }

    Ok(())
}

fn plan_requires_approval(plan: &aetheris_ai_core::agent::TaskPlan) -> bool {
    plan.steps
        .iter()
        .any(|step| step.requires_approval || step.is_destructive)
}
