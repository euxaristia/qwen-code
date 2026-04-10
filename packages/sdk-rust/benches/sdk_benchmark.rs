//! Benchmark comparing Rust SDK vs Java SDK cold startup + JSON serialization throughput.
//!
//! Run with: `cargo bench --bench sdk_benchmark`

use std::time::Instant;

use acp_sdk::protocol::jsonrpc::Message;
use acp_sdk::protocol::session_updates::SessionUpdate;
use acp_sdk::protocol::*;
use acp_sdk::Session;

fn main() {
    println!("=== ACP SDK Benchmark (Rust) ===\n");

    // 1. Cold startup time
    let start = Instant::now();
    let _ = InitializeRequestParams::default();
    let startup = start.elapsed();
    println!("Cold startup time:     {:?}", startup);

    // 2. JSON-RPC serialization throughput
    let iterations = 1_000_000;

    // Measure Message serialization
    let start = Instant::now();
    for _ in 0..iterations {
        let msg = Message::with_id("test-id".to_string());
        let json = serde_json::to_string(&msg).unwrap();
        assert!(!json.is_empty());
    }
    let elapsed = start.elapsed();
    println!(
        "Message serialization: {:?} ({}/sec)",
        elapsed,
        (iterations as f64 / elapsed.as_secs_f64()) as u64
    );

    // Measure ContentBlock serialization
    let start = Instant::now();
    for _ in 0..iterations {
        let block = ContentBlock::text("Hello, world! This is a test message for benchmarking.");
        let json = serde_json::to_string(&block).unwrap();
        assert!(!json.is_empty());
    }
    let elapsed = start.elapsed();
    println!(
        "ContentBlock ser/de:   {:?} ({}/sec)",
        elapsed,
        (iterations as f64 / elapsed.as_secs_f64()) as u64
    );

    // Measure SessionUpdate parsing
    let json_update = r#"{"type":"agent_message_chunk","content":{"text":"Hello world from the agents side of the conversation"}}"#;
    let start = Instant::now();
    for _ in 0..iterations {
        let update: SessionUpdate = serde_json::from_str(json_update).unwrap();
        match update {
            SessionUpdate::AgentMessageChunk(_) => {}
            _ => panic!("Expected AgentMessageChunk"),
        }
    }
    let elapsed = start.elapsed();
    println!(
        "SessionUpdate parsing: {:?} ({}/sec)",
        elapsed,
        (iterations as f64 / elapsed.as_secs_f64()) as u64
    );

    // Measure InitializeRequestParams serialization
    let start = Instant::now();
    for _ in 0..iterations {
        let params = InitializeRequestParams::default();
        let json = serde_json::to_string(&params).unwrap();
        assert!(!json.is_empty());
    }
    let elapsed = start.elapsed();
    println!(
        "InitParams ser/de:     {:?} ({}/sec)",
        elapsed,
        (iterations as f64 / elapsed.as_secs_f64()) as u64
    );

    // Measure AgentCapabilities serialization
    let caps = AgentCapabilities {
        load_session: Some(true),
        prompt_capabilities: Some(PromptCapabilities {
            image: Some(true),
            audio: Some(true),
            embedded_context: Some(true),
        }),
        mcp_capabilities: Some(serde_json::json!({"servers": 5})),
    };
    let start = Instant::now();
    for _ in 0..iterations {
        let json = serde_json::to_string(&caps).unwrap();
        assert!(!json.is_empty());
    }
    let elapsed = start.elapsed();
    println!(
        "AgentCaps ser/de:      {:?} ({}/sec)",
        elapsed,
        (iterations as f64 / elapsed.as_secs_f64()) as u64
    );

    // Measure full prompt request building
    let prompts = vec![
        ContentBlock::text("Write a function that sorts an array"),
        ContentBlock::text("Use quicksort"),
    ];
    let start = Instant::now();
    for _ in 0..iterations / 10 {
        let json = Session::build_prompt_request("sess-123", &prompts);
        assert!(!json.is_empty());
    }
    let elapsed = start.elapsed();
    println!(
        "Prompt build:          {:?} ({}/sec)",
        elapsed,
        ((iterations / 10) as f64 / elapsed.as_secs_f64()) as u64
    );

    // Measure line parsing (agent response parsing)
    let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"agent_message_chunk","content":{"text":"Here is the code you requested"}}}}"#;
    let start = Instant::now();
    for _ in 0..iterations {
        let mut handler = |_update: SessionUpdate| false;
        Session::parse_agent_line(line, &mut handler);
    }
    let elapsed = start.elapsed();
    println!(
        "Line parsing:          {:?} ({}/sec)",
        elapsed,
        (iterations as f64 / elapsed.as_secs_f64()) as u64
    );

    // Measure ToolCallUpdate parsing
    let line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"update":{"type":"tool_call_update","toolCallId":"tc-123","content":{"status":"running"}}}}"#;
    let start = Instant::now();
    for _ in 0..iterations {
        let mut handler = |_update: SessionUpdate| false;
        Session::parse_agent_line(line, &mut handler);
    }
    let elapsed = start.elapsed();
    println!(
        "ToolCall parsing:      {:?} ({}/sec)",
        elapsed,
        (iterations as f64 / elapsed.as_secs_f64()) as u64
    );

    println!("\n=== Done ===");
}
