# Telegram AI Moderator Bot

AI-powered Telegram bot deisnged to protect your chat groups from spam and malicious users. It uses a combination of blocklist checking and advanced AI analysis to keep your community safe.

## Features:

* **AI-Powered Content Analysis**: Checks message text using powerful AI models to detect scams, phishing, and other policy violations.
* **CAS Integration**: Automatically bans users who are globally blacklisted by the [Combot Anti-Spam (CAS)](https://cas.chat/) system.
* **User Quarantine**: New users are placed in a temporary quarantine where their first few messages are strictly monitored.
* **Dynamic Admin Controls**: Chat administrators can inject real-time rules and directives into the AI's system prompt without restarting the bot.
* **Multi-Language Support**: The bot's user-facing replies can be configured for English or Russian.
* **Flexible AI Provider**: Natively supports [OpenRouter](https://openrouter.ai/), allowing you to use almost any cloud AI model (from Google, Anthropic, OpenAI, etc.) through a single interface.
* **Persistent State**: The bot's state (chat settings, user message counts) is saved to a local database, ensuring it can resume its duties after a restart.

## How to Use

### 1. Prerequisites

* Rust toolchain (the version stated at `Cargo.toml -> package.rust-version`)
* A Telegram Bot Token (get one from [@BotFather](https://t.me/BotFather))
* An OpenRouter API Key (get one from [openrouter.ai](https://openrouter.ai/))

### 2. Configuration

The bot is configured using environment variables. You can set them directly in your shell or create a `.env` file in the project root.

**Create a `.env` file:**
```env
# Your bot token from @BotFather
TELEGRAM_BOT_API_KEY="123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"

# Your API key from OpenRouter
OPENROUTER_API_KEY="sk-or-v1-..."
```

### 3. Running the Bot

Once the configuration is in place, you can compile and run the bot with Cargo:

```bash
cargo run --release
```

The bot will start polling for updates from Telegram. Add it to your chat group and grant it administrator permissions (especially "Ban users").

### 4. Admin Commands

These commands can only be used by chat administrators:

* `/set_mode [ban|notify]` - Sets the bot's action on detecting spam. `ban` is the default.
* `/set_lang [en|ru]` - Sets the language for the bot's public replies in the chat.
* `/inject <text>` - Injects a temporary rule or directive into AI's system prompt.
* `/toggle_inject` - Activates or deactivates the use of injected admin directives.
* `/clear_injects` - Clears all currently active admin directives.
* `/status` - Displays a debug prinout of the current chat's internal state.

## How to Use. Docker edition

This repository provides `Dockerfile` and `docker-compose.yml` for easier deployment.

Just like the manual setup, the Docker container gets its configuration from a `.env` file by default.

## Future Improvements & TODOs

Pull Requests are welcome!

* **Better Admin Injects**: Implement a more robust system for managing admin directives, perhaps with expiry times or unique IDs.
* **Fine-Grained Admin Access**: Implement a more robust system for managing what admins have access to the bot's settings.
* **Webhooks Support**: Add an alternative to long polling by implementing support for Telegram Bot API webhooks.
* **More AI Providers**: Add direct support for other AI providers (non-priority).
* **Local AI**: Add support for local AI models via `ollama` or similar frameworks (non-priority).

<sub>This project is licensed under Apache License 2.0 ([LICENSE-APACHE](./LICENSE-APACHE) or [https://www.apache.org/licenses/LICENSE-2.0.txt](https://www.apache.org/licenses/LICENSE-2.0.txt)) or MIT license ([LICENSE-MIT](./LICENSE-MIT) or [https://opensource.org/license/mit](https://opensource.org/license/mit)) at your option.</sub>