#!/bin/bash
set -euo pipefail

# Button Attach Bot - Deployment Script for Netcup VPS
# Usage: ./scripts/deploy.sh [--build|--pull|--restart|--logs|--status]

APP_NAME="button-attach-bot"
APP_DIR="/opt/${APP_NAME}"
COMPOSE_FILE="docker-compose.prod.yml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running as root or with sudo
check_permissions() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root or with sudo"
        exit 1
    fi
}

# Initialize application directory
init_app_dir() {
    log_info "Initializing application directory..."
    mkdir -p "${APP_DIR}/data"
    chmod 755 "${APP_DIR}"
    chmod 700 "${APP_DIR}/data"
}

# Build the Docker image
build() {
    log_info "Building Docker image..."
    cd "${APP_DIR}"
    docker compose -f "${COMPOSE_FILE}" build --no-cache
    log_info "Build completed successfully"
}

# Start or restart the bot
start() {
    log_info "Starting ${APP_NAME}..."
    cd "${APP_DIR}"
    docker compose -f "${COMPOSE_FILE}" up -d
    log_info "${APP_NAME} started successfully"
}

# Stop the bot
stop() {
    log_info "Stopping ${APP_NAME}..."
    cd "${APP_DIR}"
    docker compose -f "${COMPOSE_FILE}" down
    log_info "${APP_NAME} stopped"
}

# Restart the bot
restart() {
    log_info "Restarting ${APP_NAME}..."
    stop
    start
}

# Show logs
logs() {
    cd "${APP_DIR}"
    docker compose -f "${COMPOSE_FILE}" logs -f --tail=100
}

# Show status
status() {
    cd "${APP_DIR}"
    echo ""
    log_info "Container Status:"
    docker compose -f "${COMPOSE_FILE}" ps
    echo ""
    log_info "Resource Usage:"
    docker stats --no-stream "${APP_NAME}" 2>/dev/null || log_warn "Container not running"
    echo ""
    log_info "Database Size:"
    du -h "${APP_DIR}/data/bot.db" 2>/dev/null || log_warn "Database not found"
}

# Backup database
backup() {
    local backup_file="${APP_DIR}/backups/bot_$(date +%Y%m%d_%H%M%S).db"
    mkdir -p "${APP_DIR}/backups"

    log_info "Creating backup: ${backup_file}"
    cp "${APP_DIR}/data/bot.db" "${backup_file}"

    # Keep only last 7 backups
    ls -t "${APP_DIR}/backups/"*.db 2>/dev/null | tail -n +8 | xargs -r rm

    log_info "Backup completed"
}

# Update and rebuild
update() {
    log_info "Updating ${APP_NAME}..."
    backup
    cd "${APP_DIR}"
    git pull origin main
    build
    restart
    log_info "Update completed"
}

# Show help
show_help() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  init      Initialize application directory"
    echo "  build     Build Docker image"
    echo "  start     Start the bot"
    echo "  stop      Stop the bot"
    echo "  restart   Restart the bot"
    echo "  logs      Show live logs"
    echo "  status    Show container status"
    echo "  backup    Backup database"
    echo "  update    Pull, rebuild, and restart"
    echo "  help      Show this help"
}

# Main
case "${1:-help}" in
    init)
        check_permissions
        init_app_dir
        ;;
    build)
        check_permissions
        build
        ;;
    start)
        check_permissions
        start
        ;;
    stop)
        check_permissions
        stop
        ;;
    restart)
        check_permissions
        restart
        ;;
    logs)
        logs
        ;;
    status)
        status
        ;;
    backup)
        check_permissions
        backup
        ;;
    update)
        check_permissions
        update
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        log_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
