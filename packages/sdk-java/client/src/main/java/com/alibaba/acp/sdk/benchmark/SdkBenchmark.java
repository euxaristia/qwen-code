package com.alibaba.acp.sdk.benchmark;

import com.alibaba.fastjson2.JSON;
import com.alibaba.acp.sdk.protocol.client.request.InitializeRequest;
import com.alibaba.acp.sdk.protocol.client.request.InitializeRequest.InitializeRequestParams;
import com.alibaba.acp.sdk.protocol.domain.content.block.TextContent;
import com.alibaba.acp.sdk.protocol.jsonrpc.Message;

/**
 * Micro-benchmark for Java SDK JSON serialization throughput.
 * Run: mvn compile exec:java -Dexec.mainClass="com.alibaba.acp.sdk.benchmark.SdkBenchmark"
 */
public class SdkBenchmark {

    public static void main(String[] args) {
        System.out.println("=== ACP SDK Benchmark (Java) ===\n");

        int iterations = 1_000_000;

        // Warmup - trigger class loading + JIT warmup
        for (int i = 0; i < 100_000; i++) {
            Message msg = new Message();
            msg.setId("warmup");
            String json = JSON.toJSONString(msg);
            JSON.parseObject(json, Message.class);
        }

        // Cold startup
        long start = System.nanoTime();
        InitializeRequestParams params = new InitializeRequestParams();
        long startupUs = (System.nanoTime() - start) / 1000;
        System.out.printf("Cold startup time:     %dµs%n", startupUs);

        // Message serialization
        start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            Message msg = new Message();
            msg.setId("test-id");
            String json = JSON.toJSONString(msg);
            if (json.isEmpty()) throw new RuntimeException("empty");
        }
        long elapsed = System.nanoTime() - start;
        System.out.printf("Message serialization: %.2fms (%d/sec)%n",
                elapsed / 1_000_000.0, (iterations * 1_000_000_000L) / elapsed);

        // ContentBlock (TextContent) serialization
        start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            TextContent content = new TextContent();
            content.setText("Hello, world! This is a test message for benchmarking.");
            String json = JSON.toJSONString(content);
            if (json.isEmpty()) throw new RuntimeException("empty");
        }
        elapsed = System.nanoTime() - start;
        System.out.printf("ContentBlock ser/de:   %.2fms (%d/sec)%n",
                elapsed / 1_000_000.0, (iterations * 1_000_000_000L) / elapsed);

        // SessionUpdate-like JSON parsing
        String jsonUpdate = "{\"type\":\"agent_message_chunk\",\"content\":{\"text\":\"Hello world from the agents side of the conversation\"}}";
        start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            Object obj = JSON.parseObject(jsonUpdate, Object.class);
            if (obj == null) throw new RuntimeException("null");
        }
        elapsed = System.nanoTime() - start;
        System.out.printf("SessionUpdate parsing: %.2fms (%d/sec)%n",
                elapsed / 1_000_000.0, (iterations * 1_000_000_000L) / elapsed);

        // InitializeRequestParams serialization
        params = new InitializeRequestParams();
        start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            String json = JSON.toJSONString(params);
            if (json.isEmpty()) throw new RuntimeException("empty");
        }
        elapsed = System.nanoTime() - start;
        System.out.printf("InitParams ser/de:     %.2fms (%d/sec)%n",
                elapsed / 1_000_000.0, (iterations * 1_000_000_000L) / elapsed);

        // Prompt request building (simplified)
        start = System.nanoTime();
        for (int i = 0; i < iterations / 10; i++) {
            java.util.Map<String, Object> req = new java.util.HashMap<>();
            req.put("jsonrpc", "2.0");
            req.put("method", "prompt");
            java.util.Map<String, Object> p = new java.util.HashMap<>();
            p.put("sessionId", "sess-123");
            java.util.Map<String, String> block = new java.util.HashMap<>();
            block.put("type", "text");
            block.put("text", "Write a function");
            p.put("prompts", java.util.Arrays.asList(block));
            req.put("params", p);
            String json = JSON.toJSONString(req);
            if (json.isEmpty()) throw new RuntimeException("empty");
        }
        elapsed = System.nanoTime() - start;
        System.out.printf("Prompt build:          %.2fms (%d/sec)%n",
                elapsed / 1_000_000.0, ((iterations / 10) * 1_000_000_000L) / elapsed);

        // Line parsing
        String line = "{\"jsonrpc\":\"2.0\",\"method\":\"session/update\",\"params\":{\"update\":{\"type\":\"agent_message_chunk\",\"content\":{\"text\":\"Here is the code you requested\"}}}}";
        start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            Object obj = JSON.parse(line);
            if (obj == null) throw new RuntimeException("null");
        }
        elapsed = System.nanoTime() - start;
        System.out.printf("Line parsing:          %.2fms (%d/sec)%n",
                elapsed / 1_000_000.0, (iterations * 1_000_000_000L) / elapsed);

        // ToolCallUpdate parsing
        String toolLine = "{\"jsonrpc\":\"2.0\",\"method\":\"session/update\",\"params\":{\"update\":{\"type\":\"tool_call_update\",\"toolCallId\":\"tc-123\",\"content\":{\"status\":\"running\"}}}}";
        start = System.nanoTime();
        for (int i = 0; i < iterations; i++) {
            Object obj = JSON.parse(toolLine);
            if (obj == null) throw new RuntimeException("null");
        }
        elapsed = System.nanoTime() - start;
        System.out.printf("ToolCall parsing:      %.2fms (%d/sec)%n",
                elapsed / 1_000_000.0, (iterations * 1_000_000_000L) / elapsed);

        System.out.println("\n=== Done ===");
    }
}
