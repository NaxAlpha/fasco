# Fasco

> "If Claude Code were 100x faster"

Fast AI chat client powered by Cerebras GLM-4.7 (1,500+ tokens/sec).

## Overview
Rust-based REPL chat client using Cerebras.ai's GLM-4.7 model for ultra-fast streaming responses.

## Tech Stack

| Component | Choice |
|-----------|--------|
| Model | `zai-glm-4.7` via Cerebras |
| HTTP | `reqwest` with streaming |
| Async | `tokio` runtime |
| REPL | `rustyline` with history |

## Architecture

```
src/
├── main.rs     - Entry point, REPL loop, commands (/exit, /clear, /help)
├── client.rs   - Cerebras HTTP client with SSE streaming
└── models.rs   - Request/response types (Message, ChatRequest, StreamChunk)
```

## Getting Started

```bash
# Set API key
export CEREBRAS_API_KEY=sk-...
# or create .env: CEREBRAS_API_KEY=sk-...

# Run
cargo run
```

Get your key at: https://cloud.cerebras.ai/

## REPL Commands
- `/help` - Show help
- `/clear` - Clear conversation
- `/exit` - Quit

## Code Style
- Flat structure, files <1000 lines
- Test-first, >80% coverage
- `cargo clippy -- -D warnings`
- `cargo fmt`
- Explicit names: `sendChatRequest()` not `send()`

## External Docs
- [Cerebras Docs](https://inference-docs.cerebras.ai/introduction)
- [GLM-4.7 Blog](https://www.cerebras.ai/blog/glm-4-7)
