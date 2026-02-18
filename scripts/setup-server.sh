#!/bin/bash
set -euo pipefail

# Button Attach Bot - Server Setup Script for Netcup VPS
# Run this script on a fresh Ubuntu/Debian server

APP_NAME="button-attach-bot"
APP_DIR="/opt/${APP_NAME}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check if running as root
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root"
    exit 1
fi

log_info "Starting server setup..."

# Update system
log_info "Updating system packages..."
apt-get update
apt-get upgrade -y

# Install Docker
if ! command -v docker &> /dev/null; then
    log_info "Installing Docker..."
    curl -fsSL https://get.docker.com | sh
    systemctl enable docker
    systemctl start docker
else
    log_info "Docker already installed"
fi

# Install Docker Compose plugin
if ! docker compose version &> /dev/null; then
    log_info "Installing Docker Compose..."
    apt-get install -y docker-compose-plugin
else
    log_info "Docker Compose already installed"
fi

# Create application directory
log_info "Creating application directory..."
mkdir -p "${APP_DIR}/data"
mkdir -p "${APP_DIR}/backups"
mkdir -p "${APP_DIR}/scripts"

# Set permissions
chmod 755 "${APP_DIR}"
chmod 700 "${APP_DIR}/data"
chmod 700 "${APP_DIR}/backups"

# Create systemd service for auto-restart
log_info "Creating systemd service..."
cat > /etc/systemd/system/${APP_NAME}.service << EOF
[Unit]
Description=Button Attach Bot
Requires=docker.service
After=docker.service

[Service]
Type=oneshot
RemainAfterExit=yes
WorkingDirectory=${APP_DIR}
ExecStart=/usr/bin/docker compose -f docker-compose.prod.yml up -d
ExecStop=/usr/bin/docker compose -f docker-compose.prod.yml down
TimeoutStartSec=0

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable ${APP_NAME}

# Setup automatic backup cron job
log_info "Setting up daily backup cron job..."
cat > /etc/cron.daily/${APP_NAME}-backup << EOF
#!/bin/bash
BACKUP_DIR="${APP_DIR}/backups"
DB_FILE="${APP_DIR}/data/bot.db"
BACKUP_FILE="\${BACKUP_DIR}/bot_\$(date +%Y%m%d).db"

if [ -f "\${DB_FILE}" ]; then
    cp "\${DB_FILE}" "\${BACKUP_FILE}"
    # Keep only last 7 days
    find "\${BACKUP_DIR}" -name "*.db" -mtime +7 -delete
fi
EOF
chmod +x /etc/cron.daily/${APP_NAME}-backup

# Setup log rotation
log_info "Setting up log rotation..."
cat > /etc/logrotate.d/${APP_NAME} << EOF
/var/lib/docker/containers/*/${APP_NAME}*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    copytruncate
}
EOF

# Setup firewall (if ufw is available)
if command -v ufw &> /dev/null; then
    log_info "Configuring firewall..."
    ufw allow ssh
    ufw --force enable
fi

# Print next steps
echo ""
log_info "Server setup completed!"
echo ""
echo "Next steps:"
echo "1. Copy your project files to ${APP_DIR}/"
echo "2. Create ${APP_DIR}/.env.prod with your configuration"
echo "3. Run: cd ${APP_DIR} && docker compose -f docker-compose.prod.yml up -d"
echo ""
echo "Or use the deploy script:"
echo "  ./scripts/deploy.sh build"
echo "  ./scripts/deploy.sh start"
echo ""
