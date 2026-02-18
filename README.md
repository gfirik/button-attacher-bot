# ButtonAttachBot

Telegram bot that attaches customizable styled buttons to channel/group posts.

![Rust](https://img.shields.io/badge/Rust-000000?style=flat&logo=rust&logoColor=white)
![Docker](https://img.shields.io/badge/Docker-2496ED?style=flat&logo=docker&logoColor=white)

## How It Works

```
Send any content → Pick destination → Add buttons → Publish!
     📷 🎥 📄          📢 Channel           🔵 🟢 🔴        ✅ Done!
                       👥 Group
                       📨 Yourself
```

**Example:** Send a photo to the bot → Choose your channel → Add a "Buy Now" button with URL → Published with a styled button that works forever!

## Features

- Attach URL buttons to any content (photos, videos, text, documents)
- Button styling: Primary (blue), Success (green), Danger (red)
- Custom emoji icons on buttons
- Analytics dashboard & admin commands
- Buttons work permanently even after bot is offline

## Quick Start (Docker)

```bash
git clone https://github.com/yourusername/button-attach-bot.git
cd button-attach-bot

cp .env.example .env
nano .env  # Add your BOT_TOKEN and ADMIN_USER_IDS
```

Get your Telegram User ID from [@userinfobot](https://t.me/userinfobot).

```bash
docker compose up -d --build
```

## Configuration

| Variable | Required | Description |
|----------|----------|-------------|
| `BOT_TOKEN` | Yes | From @BotFather |
| `ADMIN_USER_IDS` | No | Your Telegram user ID(s), comma-separated |
| `RUST_LOG` | No | Log level: `error`, `warn`, `info`, `debug` |

## Docker Commands

```bash
docker compose up -d --build   # Build and start
docker compose logs -f         # Watch logs
docker compose restart         # Restart
docker compose down            # Stop
docker compose ps              # Check status
```

## VPS Deployment

```bash
# On fresh VPS (Ubuntu/Debian)
apt update && apt install -y docker.io docker-compose-plugin git

# Clone and setup
git clone https://github.com/yourusername/button-attach-bot.git /opt/button-attach-bot
cd /opt/button-attach-bot
cp .env.example .env
nano .env  # Add secrets

# Run
docker compose up -d --build
```

## Admin Commands

| Command | Description |
|---------|-------------|
| `/stats` | Bot statistics |
| `/users` | List users |
| `/broadcast <msg>` | Message all users |
| `/block <id>` | Block user |
| `/unblock <id>` | Unblock user |

## User Commands

| Command | Description |
|---------|-------------|
| `/start` | Welcome message |
| `/help` | Help info |
| `/cancel` | Cancel operation |

## Troubleshooting

- **Bot not responding**: Check `BOT_TOKEN` in `.env`, check logs with `docker compose logs -f`
- **Can't post to channel**: Add bot as admin with "Post Messages" permission
- **Admin commands don't work**: Add your user ID to `ADMIN_USER_IDS`

## License

MIT
