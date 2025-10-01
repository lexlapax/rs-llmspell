#!/bin/bash
# docker-fleet.sh - Docker-based fleet management for LLMSpell kernels
# Provides simple commands for Docker orchestration

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

# Configuration
COMPOSE_FILE="docker-compose.yml"
IMAGE_NAME="llmspell:latest"
NETWORK_NAME="llmspell-network"

# Show usage
usage() {
    cat << EOF
$(echo -e "${GREEN}docker-fleet - Docker-based LLMSpell Fleet Management${NC}")

Usage:
    ./docker-fleet.sh <command> [arguments]

Commands:
    build               Build Docker image
    up [profile]        Start fleet (optional: dev, javascript, registry)
    down                Stop and remove containers
    scale <service> <n> Scale a service to n instances
    logs [service]      Show logs (all or specific service)
    ps                  List running containers
    health              Check health of all containers
    shell <container>   Open shell in container
    clean               Remove all containers and images

Examples:
    ./docker-fleet.sh build                    # Build image
    ./docker-fleet.sh up                       # Start default fleet
    ./docker-fleet.sh up dev                   # Start with dev profile
    ./docker-fleet.sh scale kernel-lua-openai 3 # Scale to 3 instances
    ./docker-fleet.sh logs kernel-lua-openai   # View specific logs
    ./docker-fleet.sh health                   # Check health status

EOF
}

# Build Docker image
build_image() {
    echo -e "${GREEN}Building Docker image...${NC}"

    # Build from repo root with context
    docker build -f Dockerfile -t ${IMAGE_NAME} ../.. || {
        echo -e "${RED}Build failed!${NC}"
        echo "Attempting alternative build from current directory..."
        # Alternative: create a temporary build script
        cat > /tmp/build-llmspell.sh << 'EOF'
#!/bin/bash
cd /Users/spuri/projects/lexlapax/rs-llmspell
docker build -f scripts/fleet/Dockerfile -t llmspell:latest .
EOF
        chmod +x /tmp/build-llmspell.sh
        /tmp/build-llmspell.sh
        rm /tmp/build-llmspell.sh
    }

    echo -e "${GREEN}✓ Image built successfully${NC}"
    docker images | grep llmspell
}

# Start fleet
start_fleet() {
    local profile="$1"

    echo -e "${GREEN}Starting Docker fleet...${NC}"

    # Check if image exists
    if ! docker images | grep -q "${IMAGE_NAME}"; then
        echo -e "${YELLOW}Image not found, building...${NC}"
        build_image
    fi

    # Start with optional profile
    if [ -n "$profile" ]; then
        echo -e "${BLUE}Using profile: $profile${NC}"
        docker-compose --profile "$profile" up -d
    else
        docker-compose up -d
    fi

    # Wait for containers to be ready
    echo -e "${YELLOW}Waiting for containers to be ready...${NC}"
    sleep 3

    # Show status
    docker-compose ps
    echo -e "${GREEN}✓ Fleet started${NC}"
}

# Stop fleet
stop_fleet() {
    echo -e "${RED}Stopping Docker fleet...${NC}"
    docker-compose down
    echo -e "${GREEN}✓ Fleet stopped${NC}"
}

# Scale service
scale_service() {
    local service="$1"
    local count="$2"

    if [ -z "$service" ] || [ -z "$count" ]; then
        echo -e "${RED}Usage: ./docker-fleet.sh scale <service> <count>${NC}"
        exit 1
    fi

    echo -e "${BLUE}Scaling $service to $count instances...${NC}"
    docker-compose up -d --scale "$service=$count"
    echo -e "${GREEN}✓ Scaled $service to $count instances${NC}"
}

# Show logs
show_logs() {
    local service="$1"

    if [ -n "$service" ]; then
        docker-compose logs -f "$service"
    else
        docker-compose logs -f
    fi
}

# List containers
list_containers() {
    echo -e "${GREEN}Docker Fleet Status:${NC}"
    echo "────────────────────"
    docker-compose ps
}

# Health check
health_check() {
    echo -e "${GREEN}Fleet Health Check:${NC}"
    echo "──────────────────"

    # Get container IDs
    local containers=$(docker-compose ps -q)

    if [ -z "$containers" ]; then
        echo -e "${YELLOW}No containers running${NC}"
        return
    fi

    # Check each container
    for container in $containers; do
        local name=$(docker inspect -f '{{.Name}}' "$container" | sed 's/\///')
        local status=$(docker inspect -f '{{.State.Status}}' "$container")
        local health=$(docker inspect -f '{{.State.Health.Status}}' "$container" 2>/dev/null || echo "no health check")

        # Format output
        printf "%-30s Status: %-10s Health: " "$name" "$status"

        case "$health" in
            healthy)
                echo -e "${GREEN}✓ $health${NC}"
                ;;
            unhealthy)
                echo -e "${RED}✗ $health${NC}"
                ;;
            starting)
                echo -e "${YELLOW}⟳ $health${NC}"
                ;;
            *)
                echo -e "${BLUE}- $health${NC}"
                ;;
        esac
    done

    # Show resource usage
    echo ""
    echo -e "${BLUE}Resource Usage:${NC}"
    docker stats --no-stream --format "table {{.Container}}\t{{.CPUPerc}}\t{{.MemUsage}}" $(docker-compose ps -q)
}

# Open shell in container
open_shell() {
    local container="$1"

    if [ -z "$container" ]; then
        echo -e "${RED}Usage: ./docker-fleet.sh shell <container>${NC}"
        echo "Available containers:"
        docker-compose ps --services
        exit 1
    fi

    echo -e "${BLUE}Opening shell in $container...${NC}"
    docker-compose exec "$container" /bin/bash
}

# Clean everything
clean_all() {
    echo -e "${RED}Cleaning Docker fleet...${NC}"

    # Stop and remove containers
    docker-compose down -v

    # Remove image
    docker rmi ${IMAGE_NAME} 2>/dev/null || true

    # Prune system
    docker system prune -f

    echo -e "${GREEN}✓ Docker fleet cleaned${NC}"
}

# Main command dispatcher
case "${1:-help}" in
    build)
        build_image
        ;;
    up|start)
        start_fleet "$2"
        ;;
    down|stop)
        stop_fleet
        ;;
    scale)
        scale_service "$2" "$3"
        ;;
    logs|log)
        show_logs "$2"
        ;;
    ps|list)
        list_containers
        ;;
    health|status)
        health_check
        ;;
    shell|exec)
        open_shell "$2"
        ;;
    clean)
        clean_all
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        usage
        exit 1
        ;;
esac