# ButtonAttachBot

A lightweight Rust bot that attaches customizable styled buttons to your Telegram channel and group posts. Run locally, post, shut down — buttons work forever.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)
![Telegram Bot API 9.4](https://img.shields.io/badge/Bot%20API-9.4-blue?logo=telegram)

## Features

- Attach URL buttons with custom text to ANY content type (photos, videos, text, documents, audio, stickers)
- 🎨 Button color styling: Primary (blue), Success (green), Danger (red)
- 🎭 Custom emoji icons on buttons
- ➕ Multiple buttons per post
- 📢 Native Telegram chat picker — choose destination from your own groups/channels
- 🏠 Run locally on your machine — no server, no hosting, no cost
- ♾️ Buttons work permanently even after bot is offline
- 💻 Cross-platform: Windows, macOS, Linux

## How It Works

```
You send content → Pick destination → Configure button(s) → Published! 🎉
        │                  │                    │
   Any media or      Native Telegram       Set label, URL,
   text message       chat picker         color & emoji
```

## Quick Start

### Step 1: Install Rust

> You only need to do this once.

**Windows:**
1. Go to [rustup.rs](https://rustup.rs)
2. Download and run `rustup-init.exe`
3. Follow the prompts (default options are fine)
4. Restart your terminal/command prompt

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
Follow the prompts, then restart your terminal or run `source $HOME/.cargo/env`.

### Step 2: Create Your Bot

1. Open Telegram and search for `@BotFather`
2. Send `/newbot`
3. Choose a name (e.g., "My Post Button Bot")
4. Choose a username (e.g., `my_post_button_bot`)
5. **Copy the token** BotFather gives you — you'll need it next

### Step 3: Add Bot to Your Channel/Group

1. Go to your channel/group settings
2. Add the bot as an administrator
3. Make sure it has "Post Messages" permission

### Step 4: Download and Run

```bash
# Clone the repository
git clone https://github.com/yourusername/button-attach-bot.git
cd button-attach-bot

# Set up your bot token
cp .env.example .env
```

Now open `.env` in any text editor and replace `your_bot_token_here` with your actual token:
```
BOT_TOKEN=123456789:ABCdefGHIjklMNOpqrsTUVwxyz
RUST_LOG=info
```

Then build and run:
```bash
cargo run --release
```

First run will take 1-2 minutes to compile. After that it starts instantly.

### Step 5: Use It!

1. Open Telegram, find your bot
2. Send `/start`
3. Send any content (photo, video, text, anything)
4. Tap "📢 Choose a chat" to pick where to post, or "📨 Send back to me"
5. Type the button label (e.g., "Visit Our Website")
6. Send the URL (e.g., `https://example.com`)
7. Pick a color style
8. Add more buttons or hit Publish
9. Done! Press `Ctrl+C` in terminal when finished

## Button Styles

| Style | Color | Best For |
|-------|-------|----------|
| 🔵 Primary | Blue/accent | Main actions, links |
| 🟢 Success | Green | Positive actions, join links |
| 🔴 Danger | Red | Urgent, important, limited offers |
| ⚪ Default | Standard | Neutral, secondary links |

> **Note:** Exact colors depend on the user's Telegram theme (light/dark mode). Button styling requires Telegram clients updated to support Bot API 9.4+.

## Configuration

The bot uses a `.env` file for configuration:

```env
# Required: Your Telegram bot token from @BotFather
BOT_TOKEN=your_bot_token_here

# Optional: Log level (error, warn, info, debug, trace)
RUST_LOG=info
```

## Building from Source

```bash
# Debug build (faster compile, slower runtime)
cargo build

# Release build (slower compile, optimized runtime)
cargo build --release

# The binary will be at:
# Linux/macOS: target/release/button-attach-bot
# Windows: target\release\button-attach-bot.exe
```

You can copy this single binary file anywhere and run it — just make sure `.env` is in the same directory.

## FAQ / Troubleshooting

**Q: Do buttons stop working when I close the bot?**
A: No! URL buttons are stored on Telegram's servers. They work forever, even if you delete the bot.

**Q: Bot isn't responding**
- Check your `BOT_TOKEN` in `.env` is correct
- Make sure you have internet connection
- Try running with `RUST_LOG=debug cargo run` for detailed logs

**Q: Can't post to my channel**
- The bot must be added as an **administrator** to the channel
- It needs at least "Post Messages" permission

**Q: Chat picker doesn't show my channel/group**
- Make sure the bot is already added to that chat as an admin
- The chat picker only shows chats where the bot has posting rights

**Q: Button colors don't show up**
- Button styling requires Bot API 9.4 — users need a recent version of Telegram
- On older Telegram clients, buttons will show with default styling but still work as links

**Q: Can I add multiple buttons?**
- Yes! After configuring each button, choose "Add another button" to add more before publishing.

## Project Structure

```
button-attach-bot/
├── src/
│   ├── main.rs              # Bot initialization and dispatcher setup
│   ├── config.rs            # Environment configuration
│   ├── state.rs             # Conversation states (FSM)
│   ├── handlers/            # Message and callback handlers
│   │   ├── start.rs         # /start, /help, /cancel commands
│   │   ├── content.rs       # Receives user content
│   │   ├── destination.rs   # Chat selection
│   │   ├── configure.rs     # Button configuration flow
│   │   └── publish.rs       # Sends the final post
│   ├── keyboard/            # Keyboard construction
│   │   ├── builder.rs       # Inline button builder with style support
│   │   └── destination.rs   # Chat picker keyboard
│   └── models/
│       └── session.rs       # Data structures
├── tests/
│   └── integration_test.rs  # Automated tests
├── .env.example
├── .gitignore
├── LICENSE
└── README.md
```

## Contributing

Contributions are welcome! Feel free to:
- 🐛 Report bugs via Issues
- 💡 Suggest features
- 🔀 Submit Pull Requests

## License

MIT License — see [LICENSE](LICENSE) for details.
