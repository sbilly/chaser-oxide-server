# Chaser-Oxide 部署指南

本文档详细说明 Chaser-Oxide Server 的部署架构、配置和运维。

## 目录

- [部署架构](#部署架构)
- [环境准备](#环境准备)
- [Docker 部署](#docker-部署)
- [Docker Compose 部署](#docker-compose-部署)
- [Kubernetes 部署](#kubernetes-部署)
- [配置管理](#配置管理)
- [监控和日志](#监控和日志)
- [性能优化](#性能优化)
- [故障排查](#故障排查)

## 部署架构

### 单机部署

适合开发、测试和小规模生产环境。

```
┌─────────────────────────────────────┐
│         chaser-oxide-server         │
│  ┌─────────────────────────────┐   │
│  │  gRPC Server (tokio)        │   │
│  │  - 4 workers                │   │
│  │  - 2GB RAM limit            │   │
│  └─────────────────────────────┘   │
│  ┌─────────────────────────────┐   │
│  │  Browser Pool               │   │
│  │  - Max 10 browsers          │   │
│  │  - Max 50 pages             │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

**资源要求**:
- CPU: 2 核心以上
- 内存: 4GB 以上
- 磁盘: 10GB 以上（包括浏览器缓存）

### 集群部署

适合大规模生产环境，支持负载均衡和高可用。

```
                      Load Balancer
                            │
            ┌───────────────┼───────────────┐
            │               │               │
    ┌───────▼───────┐ ┌─────▼──────┐ ┌────▼─────┐
    │  Instance 1   │ │ Instance 2 │ │Instance 3 │
    │  gRPC Server  │ │ gRPC Server│ │gRPC Server│
    └───────┬───────┘ └─────┬──────┘ └────┬─────┘
            │               │               │
            └───────────────┼───────────────┘
                            │
                    ┌───────▼────────┐
                    │ Shared Storage │
                    │ - Redis (state)│
                    │ - PostgreSQL   │
                    └────────────────┘
```

**资源要求**（每实例）:
- CPU: 4 核心以上
- 内存: 8GB 以上
- 磁盘: 20GB 以上

## 环境准备

### 系统要求

- **操作系统**: Linux (Ubuntu 20.04+, CentOS 8+, Debian 11+), macOS 11+, Windows Server 2019+
- **Rust**: 1.70 或更高版本
- **Chrome/Chromium**: 90 或更高版本
- **Docker**: 20.10 或更高版本（如果使用 Docker 部署）
- **Kubernetes**: 1.22 或更高版本（如果使用 K8s 部署）

### 依赖安装

#### Ubuntu/Debian

```bash
# 更新系统
sudo apt-get update && sudo apt-get upgrade -y

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装 Chrome
sudo apt-get install -y chromium-browser

# 安装 Docker（可选）
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 安装 Docker Compose（可选）
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

#### CentOS/RHEL

```bash
# 更新系统
sudo yum update -y

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 安装 Chrome
sudo yum install -y chromium

# 安装 Docker（可选）
sudo yum install -y docker
sudo systemctl start docker
sudo systemctl enable docker
```

#### macOS

```bash
# 安装 Homebrew（如果没有）
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 安装 Rust
brew install rust

# 安装 Chrome
brew install --cask chromium

# 安装 Docker（可选）
brew install --cask docker
```

## Docker 部署

### 构建镜像

```bash
# 克隆仓库
git clone <repository-url>
cd 20260110.chaser-oxide-server

# 构建镜像
docker build -t chaser-oxide-server:latest .

# 查看镜像
docker images | grep chaser-oxide-server
```

### Dockerfile 示例

```dockerfile
FROM rust:1.70 as builder

WORKDIR /app
COPY . .

# 编译项目
RUN cargo build --release --bin chaser-oxide-server

# 运行时镜像
FROM debian:bookworm-slim

# 安装 Chrome 和依赖
RUN apt-get update && apt-get install -y \
    chromium \
    libnss3 \
    libxss1 \
    libasound2 \
    libxtst6 \
    libxrandr2 \
    libglib2.0-0 \
    libpangocairo-1.0-0 \
    libatk1.0-0 \
    libatk-bridge2.0-0 \
    libcups2 \
    && rm -rf /var/lib/apt/lists/*

# 复制二进制文件
COPY --from=builder /app/target/release/chaser-oxide-server /usr/local/bin/

# 创建日志目录
RUN mkdir -p /var/log/chaser-oxide

# 设置环境变量
ENV CHASER_HOST=0.0.0.0
ENV CHASER_PORT=50051
ENV CHASER_LOG_LEVEL=info

# 暴露端口
EXPOSE 50051

# 启动服务
CMD ["chaser-oxide-server"]
```

### 运行容器

```bash
# 基本运行
docker run -d \
  --name chaser-oxide \
  -p 50051:50051 \
  -e CHASER_HOST=0.0.0.0 \
  -e CHASER_LOG_LEVEL=info \
  chaser-oxide-server:latest

# 挂载卷
docker run -d \
  --name chaser-oxide \
  -p 50051:50051 \
  -v /data/chaser-oxide/logs:/var/log/chaser-oxide \
  -v /data/chaser-oxide/cache:/tmp/chromium-cache \
  -e CHASER_HOST=0.0.0.0 \
  chaser-oxide-server:latest

# 连接到外部 Chrome
docker run -d \
  --name chaser-oxide \
  -p 50051:50051 \
  --network host \
  -e CHASER_CDP_ENDPOINT=ws://localhost:9222 \
  chaser-oxide-server:latest

# 资源限制
docker run -d \
  --name chaser-oxide \
  -p 50051:50051 \
  --memory="4g" \
  --cpus="2" \
  -e CHASER_HOST=0.0.0.0 \
  chaser-oxide-server:latest
```

### 容器管理

```bash
# 查看日志
docker logs -f chaser-oxide

# 进入容器
docker exec -it chaser-oxide bash

# 停止容器
docker stop chaser-oxide

# 启动容器
docker start chaser-oxide

# 重启容器
docker restart chaser-oxide

# 删除容器
docker rm -f chaser-oxide

# 查看资源使用
docker stats chaser-oxide
```

## Docker Compose 部署

### docker-compose.yml

```yaml
version: '3.8'

services:
  chaser-oxide:
    build: .
    container_name: chaser-oxide-server
    ports:
      - "50051:50051"
    environment:
      - CHASER_HOST=0.0.0.0
      - CHASER_PORT=50051
      - CHASER_CDP_ENDPOINT=ws://chrome:9222
      - CHASER_LOG_LEVEL=info
    volumes:
      - ./logs:/var/log/chaser-oxide
      - chrome-cache:/tmp/chromium-cache
    depends_on:
      - chrome
    restart: unless-stopped
    networks:
      - chaser-network

  chrome:
    image: selenium/standalone-chrome:latest
    container_name: chrome-browser
    ports:
      - "9222:9222"
    environment:
      - SE_NODE_SESSION_TIMEOUT=300
      - SE_NODE_MAX_SESSIONS=10
    shm_size: 2gb
    restart: unless-stopped
    networks:
      - chaser-network

  # 可选：Redis 用于状态共享
  redis:
    image: redis:7-alpine
    container_name: chaser-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    restart: unless-stopped
    networks:
      - chaser-network

  # 可选：PostgreSQL 用于持久化
  postgres:
    image: postgres:15-alpine
    container_name: chaser-postgres
    ports:
      - "5432:5432"
    environment:
      - POSTGRES_DB=chaser_oxide
      - POSTGRES_USER=chaser
      - POSTGRES_PASSWORD=your-password
    volumes:
      - postgres-data:/var/lib/postgresql/data
    restart: unless-stopped
    networks:
      - chaser-network

networks:
  chaser-network:
    driver: bridge

volumes:
  chrome-cache:
  redis-data:
  postgres-data:
```

### 部署命令

```bash
# 构建并启动所有服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f chaser-oxide

# 扩展服务实例
docker-compose up -d --scale chaser-oxide=3

# 停止所有服务
docker-compose down

# 停止并删除卷
docker-compose down -v
```

### 生产环境配置

```yaml
version: '3.8'

services:
  chaser-oxide:
    image: chaser-oxide-server:latest
    deploy:
      mode: replicated
      replicas: 3
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G
      restart_policy:
        condition: on-failure
        delay: 5s
        max_attempts: 3
    environment:
      - CHASER_HOST=0.0.0.0
      - CHASER_PORT=50051
      - CHASER_LOG_LEVEL=warn
    networks:
      - chaser-network
    ports:
      - "50051-50053:50051"

  nginx:
    image: nginx:alpine
    ports:
      - "50051:50051"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
    depends_on:
      - chaser-oxide
    networks:
      - chaser-network

networks:
  chaser-network:
    driver: overlay
    attachable: true
```

## Kubernetes 部署

### Namespace

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: chaser-oxide
```

### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: chaser-oxide-config
  namespace: chaser-oxide
data:
  CHASER_HOST: "0.0.0.0"
  CHASER_PORT: "50051"
  CHASER_LOG_LEVEL: "info"
  CHASER_CDP_ENDPOINT: "ws://chrome-service:9222"
```

### Deployment

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: chaser-oxide-server
  namespace: chaser-oxide
  labels:
    app: chaser-oxide
spec:
  replicas: 3
  selector:
    matchLabels:
      app: chaser-oxide
  template:
    metadata:
      labels:
        app: chaser-oxide
    spec:
      containers:
      - name: chaser-oxide
        image: chaser-oxide-server:latest
        imagePullPolicy: Always
        ports:
        - containerPort: 50051
          name: grpc
          protocol: TCP
        envFrom:
        - configMapRef:
            name: chaser-oxide-config
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          exec:
            command:
            - /bin/sh
            - -c
            - "pgrep chaser-oxide-server"
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          exec:
            command:
            - /bin/sh
            - -c
            - "nc -z localhost 50051"
          initialDelaySeconds: 5
          periodSeconds: 5
        volumeMounts:
        - name: logs
          mountPath: /var/log/chaser-oxide
      volumes:
      - name: logs
        emptyDir: {}
```

### Service

```yaml
# service.yaml
apiVersion: v1
kind: Service
metadata:
  name: chaser-oxide-service
  namespace: chaser-oxide
spec:
  type: ClusterIP
  ports:
  - port: 50051
    targetPort: 50051
    protocol: TCP
    name: grpc
  selector:
    app: chaser-oxide
```

### Ingress

```yaml
# ingress.yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: chaser-oxide-ingress
  namespace: chaser-oxide
  annotations:
    nginx.ingress.kubernetes.io/backend-protocol: "GRPC"
    nginx.ingress.kubernetes.io/grpc-backend: "true"
spec:
  ingressClassName: nginx
  rules:
  - host: chaser-oxide.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: chaser-oxide-service
            port:
              number: 50051
```

### 部署命令

```bash
# 创建命名空间
kubectl apply -f namespace.yaml

# 部署配置
kubectl apply -f configmap.yaml
kubectl apply -f deployment.yaml
kubectl apply -f service.yaml
kubectl apply -f ingress.yaml

# 查看部署状态
kubectl get pods -n chaser-oxide
kubectl get services -n chaser-oxide
kubectl get ingress -n chaser-oxide

# 查看日志
kubectl logs -f deployment/chaser-oxide-server -n chaser-oxide

# 扩展副本
kubectl scale deployment chaser-oxide-server --replicas=5 -n chaser-oxide
```

## 配置管理

### 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `CHASER_HOST` | `127.0.0.1` | gRPC 服务器绑定地址 |
| `CHASER_PORT` | `50051` | gRPC 服务器端口 |
| `CHASER_CDP_ENDPOINT` | `ws://localhost:9222` | CDP WebSocket 端点 |
| `CHASER_LOG_LEVEL` | `info` | 日志级别 |
| `CHASER_MAX_BROWSERS` | `10` | 最大浏览器数量 |
| `CHASER_MAX_PAGES` | `50` | 最大页面数量 |
| `CHASER_SESSION_TIMEOUT` | `300` | Session 超时时间（秒） |
| `CHASER_CLEANUP_INTERVAL` | `300` | 清理间隔（秒） |

### 配置文件

创建 `config.toml`:

```toml
[server]
host = "0.0.0.0"
port = 50051
max_connections = 1000
timeout = 30

[browser]
executable_path = "/usr/bin/chromium"
headless = true
window_width = 1920
window_height = 1080
max_instances = 10
args = [
    "--disable-gpu",
    "--no-sandbox",
    "--disable-setuid-sandbox",
    "--disable-dev-shm-usage"
]

[cdp]
endpoint = "ws://localhost:9222"
timeout = 30
retry_attempts = 3
retry_delay = 1000

[limits]
max_browsers = 10
max_pages_per_browser = 5
max_pages_total = 50
session_timeout = 300
cleanup_interval = 300

[logging]
level = "info"
format = "json"
output = "/var/log/chaser-oxide/server.log"
max_size = 100  # MB
max_backups = 10
max_age = 30    # days

[monitoring]
metrics_enabled = true
metrics_port = 9090
health_check_enabled = true
health_check_path = "/health"

[stealth]
enabled = true
profile_randomization = true
human_behavior_simulation = true
```

### 使用配置文件

```bash
# 通过配置文件启动
chaser-oxide-server --config config.toml

# 或设置环境变量
export CHASER_CONFIG=/path/to/config.toml
chaser-oxide-server
```

## 监控和日志

### Prometheus 指标

服务暴露以下 Prometheus 指标：

| 指标名 | 类型 | 描述 |
|--------|------|------|
| `chaser_requests_total` | Counter | 总请求数 |
| `chaser_request_duration_seconds` | Histogram | 请求耗时 |
| `chaser_active_browsers` | Gauge | 活跃浏览器数 |
| `chaser_active_pages` | Gauge | 活跃页面数 |
| `chaser_errors_total` | Counter | 错误总数 |
| `chaser_session_cleanup_duration_seconds` | Histogram | Session 清理耗时 |

### Prometheus 配置

```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'chaser-oxide'
    static_configs:
      - targets: ['chaser-oxide:9090']
    metrics_path: /metrics
```

### Grafana Dashboard

创建 Grafana 仪表盘监控：

- 请求速率和成功率
- 响应时间分布
- 活跃浏览器和页面数量
- 错误率
- 资源使用情况

### 日志管理

#### 日志格式

服务使用结构化 JSON 日志：

```json
{
  "timestamp": "2024-01-10T10:30:45.123Z",
  "level": "info",
  "service": "chaser-oxide",
  "module": "browser",
  "message": "Browser launched",
  "browser_id": "browser-uuid-1234",
  "duration_ms": 1250
}
```

#### 日志聚合

使用 ELK Stack 或 Loki 进行日志聚合：

```yaml
# filebeat.yml
filebeat.inputs:
- type: container
  paths:
    - '/var/lib/docker/containers/*/*.log'
  processors:
  - add_docker_metadata:

output.elasticsearch:
  hosts: ["elasticsearch:9200"]
```

### 健康检查

```bash
# HTTP 健康检查
curl http://localhost:9090/health

# 响应
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime": 3600,
  "browsers": {
    "active": 5,
    "max": 10
  },
  "pages": {
    "active": 23,
    "max": 50
  }
}
```

## 性能优化

### 资源限制

```yaml
# Kubernetes 资源限制
resources:
  requests:
    memory: "2Gi"
    cpu: "1000m"
  limits:
    memory: "4Gi"
    cpu: "2000m"
```

### 连接池配置

```toml
[server]
max_connections = 1000
keepalive_interval = 60
keepalive_timeout = 5
```

### 缓存配置

```toml
[cache]
enabled = true
type = "redis"
endpoint = "redis:6379"
ttl = 3600
max_size = 10000
```

### 并发控制

```toml
[limits]
max_concurrent_requests = 100
max_concurrent_browsers = 10
max_concurrent_pages = 50
```

## 故障排查

### 常见问题

#### 1. 无法启动

**症状**: 服务启动失败

**诊断**:
```bash
# 查看日志
docker logs chaser-oxide

# 检查配置
docker exec chaser-oxide chaser-oxide-server --validate-config
```

**解决方案**:
- 检查环境变量配置
- 验证 Chrome 可执行文件路径
- 确认端口未被占用

#### 2. 连接 Chrome 失败

**症状**: `Failed to connect to CDP endpoint`

**诊断**:
```bash
# 测试 CDP 连接
wscat -c ws://localhost:9222

# 检查 Chrome 进程
ps aux | grep chromium
```

**解决方案**:
- 确保 Chrome 以远程调试模式启动
- 检查防火墙规则
- 验证 CDP_ENDPOINT 配置

#### 3. 内存泄漏

**症状**: 内存使用持续增长

**诊断**:
```bash
# 监控内存使用
docker stats chaser-oxide

# 检查浏览器进程
docker exec chaser-oxide ps aux
```

**解决方案**:
- 减少最大浏览器数量
- 启用自动清理
- 定期重启容器

#### 4. 性能下降

**症状**: 响应时间增加

**诊断**:
```bash
# 查看指标
curl http://localhost:9090/metrics

# 分析日志
grep "duration_ms" /var/log/chaser-oxide/server.log
```

**解决方案**:
- 扩展实例数量
- 优化资源限制
- 启用缓存

### 调试模式

```bash
# 启用调试日志
CHASER_LOG_LEVEL=debug chaser-oxide-server

# 启用追踪
RUST_LOG=chaser_oxide=trace chaser-oxide-server
```

### 性能分析

```bash
# CPU 性能分析
docker run --rm --cpuset-cpus="0" \
  -v /path/to/profile:/profile \
  chaser-oxide-server:latest \
  flamegraph /profile/flamegraph.svg

# 内存分析
docker run --rm \
  -v /path/to/profile:/profile \
  chaser-oxide-server:latest \
  heaptrack /profile/heaptrack.log
```

## 备份和恢复

### 数据备份

```bash
# 备份配置
tar -czf config-backup-$(date +%Y%m%d).tar.gz config.toml

# 备份日志
tar -czf logs-backup-$(date +%Y%m%d).tar.gz /var/log/chaser-oxide

# 备份 Redis（如果使用）
docker exec redis redis-cli SAVE
docker cp redis:/data/dump.rdf ./redis-backup-$(date +%Y%m%d).rdb
```

### 数据恢复

```bash
# 恢复配置
tar -xzf config-backup-20240110.tar.gz -C /

# 恢复日志
tar -xzf logs-backup-20240110.tar.gz -C /

# 恢复 Redis
docker cp redis-backup-20240110.rdb redis:/data/dump.rdb
docker restart redis
```

## 安全加固

### TLS/SSL 配置

```toml
[server.tls]
enabled = true
cert_file = "/etc/chaser-oxide/certs/server.crt"
key_file = "/etc/chaser-oxide/certs/server.key"
ca_file = "/etc/chaser-oxide/certs/ca.crt"
client_auth = "require-and-verify"
```

### 认证配置

```toml
[auth]
enabled = true
type = "jwt"
secret = "your-secret-key"
expiration = 3600
algorithm = "HS256"
```

### 网络安全

```yaml
# Kubernetes NetworkPolicy
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: chaser-oxide-netpol
  namespace: chaser-oxide
spec:
  podSelector:
    matchLabels:
      app: chaser-oxide
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: trusted-namespace
    ports:
    - protocol: TCP
      port: 50051
```

## 相关文档

- [README.md](README.md) - 项目介绍
- [API.md](API.md) - API 使用文档
- [DEVELOPMENT.md](DEVELOPMENT.md) - 开发指南
- [docs/architecture.md](docs/architecture.md) - 架构设计
