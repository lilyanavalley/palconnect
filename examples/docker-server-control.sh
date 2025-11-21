#!/bin/bash

# Custom script example for Docker-based PalWorld server
# This demonstrates how to use custom scripts with PalConnect

CONTAINER_NAME="palworld-server"
IMAGE_NAME="palworld:latest"
PALWORLD_PORT="8211"
PALWORLD_PLAYERS="32"
DATA_DIR="/opt/palworld-data"

start_server() {
    echo "Starting PalWorld server container..."
    
    # Check if container exists and is running
    if docker ps | grep -q "$CONTAINER_NAME"; then
        echo "Container $CONTAINER_NAME is already running"
        exit 0
    fi
    
    # Check if container exists but is stopped
    if docker ps -a | grep -q "$CONTAINER_NAME"; then
        echo "Starting existing container..."
        docker start "$CONTAINER_NAME"
    else
        echo "Creating new container..."
        docker run -d \
            --name "$CONTAINER_NAME" \
            -p "${PALWORLD_PORT}:8211" \
            -v "${DATA_DIR}:/palworld/data" \
            -e PLAYERS="$PALWORLD_PLAYERS" \
            "$IMAGE_NAME"
    fi
    
    echo "PalWorld server started successfully"
}

stop_server() {
    echo "Stopping PalWorld server container..."
    
    if docker ps | grep -q "$CONTAINER_NAME"; then
        docker stop "$CONTAINER_NAME"
        echo "PalWorld server stopped successfully"
    else
        echo "Container $CONTAINER_NAME is not running"
    fi
}

force_stop_server() {
    echo "Force stopping PalWorld server container..."
    
    if docker ps -a | grep -q "$CONTAINER_NAME"; then
        docker kill "$CONTAINER_NAME" 2>/dev/null || true
        echo "PalWorld server force stopped"
    else
        echo "Container $CONTAINER_NAME does not exist"
    fi
}

case "${1:-}" in
    start)
        start_server
        ;;
    stop)
        stop_server
        ;;
    force-stop)
        force_stop_server
        ;;
    *)
        echo "Usage: $0 {start|stop|force-stop}"
        exit 1
        ;;
esac
