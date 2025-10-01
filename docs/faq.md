## FAQ

### OpenAI released a model called Codex in 2021 - is this related?

In 2021, OpenAI released Codex, an AI system designed to generate code from natural language prompts. That original Codex model was deprecated as of March 2023 and is separate from the CLI tool.

### Do I need to log in with ChatGPT to use the CLI?

No. The Rust CLI now uses the open-source `gpt-oss:20b` model served locally via [Ollama](https://ollama.com/) by default.
Start the Ollama daemon with `ollama serve` and Codex will download the model automatically the first time you run it.
No ChatGPT authentication is required for this workflow.

### Which models are supported?

The CLI ships with the open-source `gpt-oss:20b` model by default. It runs locally and does not use any hosted
OpenAI resources. You can switch providers at runtime with `codex --provider <id>` (aliases: `--provider oss`,
`--provider openai`, or the shortcut `--oss`). The same options are available via environment variables:

- `CODEX_PROVIDER` (`oss`, `ollama`, or `openai`)
- `CODEX_MODEL`
- `OLLAMA_HOST` (defaults to `http://localhost:11434`)

If you prefer to use OpenAI-hosted models (such as GPT-5 or `o4-mini`), set the provider in `~/.codex/config.toml`:

```toml
model_provider = "openai"
model = "gpt-5-codex"
```

The default reasoning level is medium, and you can upgrade to high for complex tasks with the `/model` command.

You can also use older models by using API-based auth and launching codex with the `--model` flag.

### Why does `o3` or `o4-mini` not work for me?

It's possible that your [API account needs to be verified](https://help.openai.com/en/articles/10910291-api-organization-verification) in order to start streaming responses and seeing chain of thought summaries from the API. If you're still running into issues, please let u know!

### How do I stop Codex from editing my files?

By default, Codex can modify files in your current working directory (Auto mode). To prevent edits, run `codex` in read-only mode with the CLI flag `--sandbox read-only`. Alternatively, you can change the approval level mid-conversation with `/approvals`.

### Does it work on Windows?

Running Codex directly on Windows may work, but is not officially supported. We recommend using [Windows Subsystem for Linux (WSL2)](https://learn.microsoft.com/en-us/windows/wsl/install). 
