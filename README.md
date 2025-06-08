# Telegram AI Moderator Bot

An AI-powered Telegram bot designed to protect your chat groups from spam and malicious users. It uses a combination of blocklist checking and advanced AI analysis to keep your community safe.

## Features

* **AI-Powered Content Analysis**: Checks message text using powerful AI models to detect scams, phishing, and other policy violations.
* **CAS Integration**: Automatically bans users who are globally blacklisted by the [Combot Anti-Spam (CAS)](https://cas.chat/) system.
* **User Quarantine**: New users are placed in a temporary quarantine where their first few messages are strictly monitored by the AI.
* **Dynamic Admin Controls**: Chat administrators can inject real-time rules and directives into the AI's system prompt without restarting the bot.
* **Multi-Language Support**: The bot's user-facing replies can be configured for English or Russian.
* **Flexible AI Provider**: Natively supports [OpenRouter](https://openrouter.ai/), allowing you to use almost any cloud AI model (from Google, Anthropic, OpenAI, etc.) through a single interface.
* **Persistent State**: The bot's state (chat settings, user message counts) is saved to a local database, ensuring it can resume its duties after a restart.
* **Efficient Webhook Mode**: Offers a high-performance webhook option to receive updates instantly, significantly reducing bandwidth usage and server load compared to long polling.

## How to Use

### 1. Prerequisites

* Rust toolchain (the version stated at `Cargo.toml -> package.rust-version`)
* A Telegram Bot Token (get one from [@BotFather](https://t.me/BotFather))
* An OpenRouter API Key (get one from [openrouter.ai](https://openrouter.ai/))
* For webhook mode: a publicly accessible server with a domain name and a configured reverse proxy (like Nginx or Caddy).

### 2. Configuration

The bot is configured using environment variables and/or command-line arguments. Create a `.env` file in the project root to store your secrets.

**`.env.example`:**
```env
# Your bot token from @BotFather
TELEGRAM_BOT_API_KEY="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"

# Your API key from OpenRouter
OPENROUTER_API_KEY="sk-or-v1-..."

# (Optional) A secret token for webhook mode to verify requests are from Telegram.
TELEGRAM_WEBHOOK_SECRET="your_very_long_and_random_secret_string_here"
```

Create your own `.env` file by copying the example: `cp .env.example .env` and filling in the values.

### 3. Running the Bot (Manual Setup)

Add the bot to your chat group and grant it administrator permissions (especially "Ban users").

#### Option A: Long Polling

This is the simplest method. The bot will continuously ask Telegram for new messages.
```bash
cargo run --release
```

#### Option B: Webhooks

Telegram will send updates to your public endpoint as they happen. This is more efficient. You must provide your public URL. The bot will listen on `0.0.0.0:8080` by default.

```bash
cargo run --release -- \
    --webhook-url "https://your.public.domain/your-webhook-path" \
    --webhook-listen-addr "0.0.0.0:8080"
```
*   `--webhook-url`: The full, public URL that Telegram will send updates to.
*   `--webhook-listen-addr`: The internal IP and port the bot's server will bind to.

### 4. Deployment with Docker

#### Option A: Polling Mode with Docker

This is the simplest Docker deployment. For that you can use the provided [polling.docker-compose.yml](./polling.docker-compose.yml) from the get go.

#### Option B: Webhook Mode with Docker

This requires exposing a port and configuring the webhook URL. Create a file named `webhook.docker-compose.yml`. Look into [webhook.docker-compose.yml](./webhook.docker-compose.yml) for reference.

### 5. Admin Commands

These commands can only be used by chat administrators:

* `/set_mode [ban|notify]` - Sets the bot's action on detecting spam. `ban` is the default.
* `/set_lang [en|ru]` - Sets the language for the bot's public replies in the chat.
* `/inject <text>` - Injects a temporary rule or directive into the AI's system prompt.
* `/toggle_inject` - Activates or deactivates the use of injected admin directives.
* `/clear_injects` - Clears all currently active admin directives.
* `/status` - Displays a debug printout of the current chat's internal state.
* `/help` - Shows build information about the bot.

## Future Improvements & TODOs

Pull Requests are welcome!

* **Better Admin Injects**: Implement a more robust system for managing admin directives, perhaps with expiry times or unique IDs.
* **Fine-Grained Admin Access**: Implement a system for managing which admins have access to the bot's settings.
* **More AI Providers**: Add direct support for other AI providers (non-priority).
* **Local AI**: Add support for local AI models via `ollama` or similar frameworks (non-priority).

---
<sub>This project is licensed under Apache License 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or [https://www.apache.org/licenses/LICENSE-2.0.txt](https://www.apache.org/licenses/LICENSE-2.0.txt)) or MIT license ([LICENSE-MIT](./LICENSE-MIT) or [https://opensource.org/license/mit](https://opensource.org/license/mit)) at your option.</sub>