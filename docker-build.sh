#!/bin/bash

# Chaser Oxide Server - Docker Build & Deploy Script
# This script builds and deploys the chaser-oxide-server using Docker

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Functions
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if .env exists
if [ ! -f .env ]; then
    print_warn ".env file not found. Creating from .env.example..."
    cp .env.example .env
    print_info "Please edit .env file with your configuration before running the server"
    exit 1
fi

# Parse command line arguments
COMMAND=${1:-"build"}

case $COMMAND in
    build)
        print_info "Building Docker image..."
        docker build -t chaser-oxide-server:latest .
        print_info "Build completed successfully!"
        ;;

    up|start)
        print_info "Starting containers..."
        docker-compose up -d
        print_info "Containers started!"
        print_info "Check logs with: docker-compose logs -f"
        ;;

    down|stop)
        print_info "Stopping containers..."
        docker-compose down
        print_info "Containers stopped!"
        ;;

    restart)
        print_info "Restarting containers..."
        docker-compose restart
        print_info "Containers restarted!"
        ;;

    logs)
        print_info "Showing logs..."
        docker-compose logs -f
        ;;

    status)
        print_info "Container status:"
        docker-compose ps
        ;;

    clean)
        print_warn "This will remove all containers, images, and volumes"
        read -p "Are you sure? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_info "Cleaning up..."
            docker-compose down -v --rmi all
            print_info "Cleanup completed!"
        else
            print_info "Cleanup cancelled"
        fi
        ;;

    shell)
        print_info "Opening shell in container..."
        docker-compose exec chaser-oxide /bin/bash
        ;;

    rebuild)
        print_info "Rebuilding Docker image (no cache)..."
        docker-compose build --no-cache
        print_info "Rebuild completed!"
        ;;

    *)
        echo "Usage: $0 {build|up|down|restart|logs|status|clean|shell|rebuild}"
        echo ""
        echo "Commands:"
        echo "  build      - Build Docker image"
        echo "  up/start   - Start containers"
        echo "  down/stop  - Stop containers"
        echo "  restart    - Restart containers"
        echo "  logs       - Show container logs"
        echo "  status     - Show container status"
        echo "  clean      - Remove all containers, images, and volumes"
        echo "  shell      - Open shell in running container"
        echo "  rebuild    - Rebuild image without cache"
        exit 1
        ;;
esac
