# Docker Deployment Guide

This guide covers deploying `chaser-oxide-server` using Docker.

## Quick Start

### 1. Configure Environment

```bash
cp .env.example .env
# Edit .env with your configuration
```

### 2. Build and Run

**Option A: Using the helper script**
```bash
./docker-build.sh build
./docker-build.sh up
```

**Option B: Using docker-compose directly**
```bash
docker-compose build
docker-compose up -d
```

### 3. Check Status

```bash
./docker-build.sh status
# or
docker-compose ps
```

### 4. View Logs

```bash
./docker-build.sh logs
# or
docker-compose logs -f
```

## Management Commands

### Using Helper Script

```bash
./docker-build.sh build      # Build Docker image
./docker-build.sh up         # Start containers
./docker-build.sh down       # Stop containers
./docker-build.sh restart    # Restart containers
./docker-build.sh logs       # Show logs
./docker-build.sh status     # Show status
./docker-build.sh shell      # Open shell in container
./docker-build.sh rebuild    # Rebuild without cache
./docker-build.sh clean      # Remove everything (WARNING: deletes data)
```

### Using Docker Compose

```bash
# Build and start
docker-compose up -d

# Stop
docker-compose down

# View logs
docker-compose logs -f chaser-oxide

# Execute command in container
docker-compose exec chaser-oxide /bin/bash

# Rebuild
docker-compose build --no-cache
```

## Configuration

### Environment Variables

Key environment variables in `.env`:

- `RUST_LOG`: Log level (debug, info, warn, error)
- `CHASER_HOST`: Server host (default: 0.0.0.0)
- `CHASER_PORT`: Server port (default: 50051)
- `CHASER_HEADLESS`: Run browser in headless mode (default: true)
- `CHASER_BROWSER_PATH`: Path to browser executable
- `CHASER_ENABLE_STEALTH`: Enable stealth mode (default: true)

### Resource Limits

Default resource limits in `docker-compose.yml`:

- CPU: 2 cores (limit), 0.5 cores (reservation)
- Memory: 2GB (limit), 512MB (reservation)

Adjust these in `docker-compose.yml` based on your needs.

### Volumes

- `./data:/app/data`: Persistent data storage

## Troubleshooting

### Container Won't Start

1. Check logs: `docker-compose logs chaser-oxide`
2. Verify configuration: `cat .env`
3. Check port availability: `lsof -i :50051`

### Out of Memory

Increase memory limit in `docker-compose.yml`:

```yaml
deploy:
  resources:
    limits:
      memory: 4G  # Increase as needed
```

### Browser Issues

Ensure chromium is installed in the container:

```bash
docker-compose exec chaser-oxide which chromium
```

## Production Considerations

### Security

1. Use secrets management for sensitive data
2. Run as non-root user (modify Dockerfile)
3. Enable HTTPS/TLS for gRPC
4. Implement authentication/authorization

### Monitoring

1. Enable structured logging
2. Add metrics export (Prometheus)
3. Set up health checks
4. Monitor resource usage

### High Availability

1. Use container orchestration (Kubernetes, Docker Swarm)
2. Implement load balancing
3. Set up auto-scaling
4. Configure persistent storage

## Development

### Local Development

```bash
# Build with development tools
docker build --target builder -t chaser-oxide:dev .

# Run with hot-reload (requires additional setup)
docker-compose -f docker-compose.dev.yml up
```

### Testing

```bash
# Run tests in container
docker-compose run --rm chaser-oxide cargo test
```

## Support

For issues and questions:
- Check logs: `docker-compose logs -f`
- Open shell: `./docker-build.sh shell`
- Review configuration: `cat .env`
