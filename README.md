# Chaser-Oxide Server

[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![gRPC](https://img.shields.io/badge/gRPC-0.12-blue.svg)](https://github.com/hyperium/tonic)

**Chaser-Oxide Server** 是一个基于 Rust 的高性能浏览器自动化微服务，通过 gRPC 协议提供对 Chrome DevTools Protocol (CDP) 的抽象访问。服务设计强调高性能、类型安全和反检测能力。

本项目基于 [ccheshirecat/chaser-oxide](https://github.com/ccheshirecat/chaser-oxide) 开发，感谢原作者的开源贡献。

## 特性

- **高性能** - 基于 Tokio 异步运行时和 gRPC/HTTP2，支持高并发
- **类型安全** - 使用 Protocol Buffers 定义接口，提供强类型保证
- **会话隔离** - 每个页面有独立的会话 ID，支持多浏览器/多页面并发操作
- **实时事件** - 双向流式传输实现事件推送和订阅
- **隐身能力** - 完整的指纹配置和人类行为模拟，支持反检测
- **资源管理** - 自动清理不活跃会话，支持优雅关闭
- **跨平台** - 支持 Linux、macOS 和 Windows

## 架构概述

```
客户端 (Python/Go/Rust)
        │
        │ gRPC over HTTP/2
        ▼
┌─────────────────────────────┐
│   Chaser-Oxide gRPC Server  │
│  ┌─────────────────────────┐│
│  │  Browser Service        ││
│  │  Page Service           ││
│  │  Element Service        ││
│  │  Event Service          ││
│  │  Profile Service        ││
│  └─────────────────────────┘│
└──────────────┬──────────────┘
               │ WebSocket (CDP)
               ▼
        Chrome/Chromium Browser
```

## 快速开始

### 环境要求

- **Rust** 1.70 或更高版本
- **Chrome/Chromium** 浏览器（支持远程调试模式）
- **操作系统**：Linux、macOS 或 Windows

### 安装 Chrome/Chromium

#### Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y chromium-browser
```

#### macOS
```bash
brew install --cask chromium
```

#### Windows
从 [Chromium 官网](https://www.chromium.org/) 下载并安装。

### 启动 Chrome 远程调试

在使用服务之前，需要启动 Chrome 的远程调试模式：

```bash
# Linux/macOS
google-chrome --remote-debugging-port=9222 --no-first-run --no-default-browser-check

# Windows
chrome.exe --remote-debugging-port=9222 --no-first-run --no-default-browser-check
```

### 编译和运行

```bash
# 克隆仓库
git clone <repository-url>
cd 20260110.chaser-oxide-server

# 编译项目
cargo build --release

# 运行服务器（使用默认配置）
cargo run --release --bin chaser-oxide-server

# 自定义 CDP 端点
CHASER_CDP_ENDPOINT=ws://localhost:9222 cargo run --release --bin chaser-oxide-server

# 自定义服务器地址和端口
CHASER_HOST=0.0.0.0 CHASER_PORT=50052 cargo run --release --bin chaser-oxide-server
```

### 环境变量

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| `CHASER_HOST` | `127.0.0.1` | gRPC 服务器绑定地址 |
| `CHASER_PORT` | `50051` | gRPC 服务器端口 |
| `CHASER_CDP_ENDPOINT` | `ws://localhost:9222` | Chrome DevTools Protocol 端点 |
| `CHASER_LOG_LEVEL` | `info` | 日志级别（trace、debug、info、warn、error） |

### Docker 部署

```bash
# 构建镜像
docker build -t chaser-oxide-server:latest .

# 运行容器
docker run -d \
  -p 50051:50051 \
  -e CHASER_HOST=0.0.0.0 \
  -e CHASER_CDP_ENDPOINT=ws://chrome:9222 \
  --name chaser-oxide \
  chaser-oxide-server:latest
```

详细部署说明请参阅 [部署指南](docs/guides/deployment.md)。

## 基本使用示例

### Python 客户端示例

> **注意**: 使用 Python 客户端前需要先运行 protoc 生成 gRPC 代码。
> 完整示例请参考 [docs/examples/python/basic_client.py](docs/examples/python/basic_client.py)

**安装依赖**:
```bash
pip install grpcio grpcio-tools
```

**生成 gRPC 代码**:
```bash
# 生成 Python gRPC 代码到 chaser 包
cd docs/examples/python
python -m grpc_tools.protoc -I../../../protos \
    --python_out=. --grpc_python_out=. \
    ../../../protos/common.proto \
    ../../../protos/browser.proto \
    ../../../protos/page.proto \
    ../../../protos/element.proto \
    ../../../protos/profile.proto \
    ../../../protos/event.proto
```

生成的代码将位于 `chaser/oxide/v1/` 目录下。

**使用方法**:
```python
import grpc
from chaser.oxide.v1 import browser_pb2, browser_pb2_grpc, page_pb2, page_pb2_grpc

# 连接到服务器
channel = grpc.insecure_channel('localhost:50051')
browser_client = browser_pb2_grpc.BrowserServiceStub(channel)
page_client = page_pb2_grpc.PageServiceStub(channel)

# 启动浏览器
launch_request = browser_pb2.LaunchRequest(
    options=browser_pb2.BrowserOptions(
        headless=True,
        window_width=1920,
        window_height=1080
    )
)
launch_response = browser_client.Launch(launch_request)
browser_id = launch_response.browser_info.browser_id
print(f"Browser launched: {browser_id}")

# 创建页面
create_request = page_pb2.CreatePageRequest(browser_id=browser_id)
create_response = page_client.CreatePage(create_request)
page_id = create_response.page_info.page_id
print(f"Page created: {page_id}")

# 导航到 URL
navigate_request = page_pb2.NavigateRequest(
    page_id=page_id,
    url="https://example.com"
)
navigate_response = page_client.Navigate(navigate_request)
print(f"Navigated to: {navigate_response.url}")

# 获取页面内容
content_request = page_pb2.GetContentRequest(page_id=page_id)
content_response = page_client.GetContent(content_request)
print(f"Page content length: {len(content_response.content)}")

# 关闭资源
close_request = page_pb2.ClosePageRequest(page_id=page_id)
page_client.ClosePage(close_request)
```

### Go 客户端示例

> **注意**: 使用 Go 客户端前需要先运行 protoc 生成 gRPC 代码。
> 完整示例请参考 [docs/examples/go/basic_client.go](docs/examples/go/basic_client.go)

**安装 protoc 插件**:
```bash
go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest
```

**生成 gRPC 代码**:
```bash
# 创建输出目录
mkdir -p protos/go

# 生成 Go gRPC 代码
protoc -I./protos --go_out=protos/go --go_opt=paths=source_relative \
    --go-grpc_out=protos/go --go-grpc_opt=paths=source_relative \
    protos/common.proto \
    protos/browser.proto \
    protos/page.proto \
    protos/element.proto \
    protos/profile.proto \
    protos/event.proto
```

生成的代码将位于 `protos/go/` 目录下。

```go
package main

import (
    "context"
    "log"
    "time"

    "google.golang.org/grpc"
    "google.golang.org/protobuf/types/known/wrapperspb"

    // 请将 your-module-path 替换为您的实际模块路径
    pb "your-module-path/protos"
)

func main() {
    // 连接到服务器
    conn, err := grpc.Dial("localhost:50051", grpc.WithInsecure())
    if err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }
    defer conn.Close()

    browserClient := pb.NewBrowserServiceClient(conn)
    pageClient := pb.NewPageServiceClient(conn)

    ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
    defer cancel()

    // 启动浏览器
    launchResp, err := browserClient.Launch(ctx, &pb.LaunchRequest{
        Options: &pb.BrowserOptions{
            Headless:    true,
            WindowWidth: 1920,
            WindowHeight: 1080,
        },
    })
    if err != nil {
        log.Fatalf("Launch failed: %v", err)
    }
    browserID := launchResp.BrowserInfo.BrowserId
    log.Printf("Browser launched: %s", browserID)

    // 创建页面
    createResp, err := pageClient.CreatePage(ctx, &pb.CreatePageRequest{
        BrowserId: browserID,
    })
    if err != nil {
        log.Fatalf("CreatePage failed: %v", err)
    }
    pageID := createResp.Page.PageId
    log.Printf("Page created: %s", pageID)

    // 导航到 URL
    navResp, err := pageClient.Navigate(ctx, &pb.NavigateRequest{
        PageId: pageID,
        Url:    "https://example.com",
    })
    if err != nil {
        log.Fatalf("Navigate failed: %v", err)
    }
    log.Printf("Navigated to: %s", navResp.Url)
}
```

## 服务 API

Chaser-Oxide Server 提供以下 gRPC 服务：

- **BrowserService** - 浏览器生命周期管理
- **PageService** - 页面操作和导航
- **ElementService** - 元素查找和交互
- **EventService** - 实时事件订阅
- **ProfileService** - 隐身配置管理

详细的 API 文档请参阅 [API 使用文档](docs/api/api.md)。

## 开发指南

### 开发环境设置

```bash
# 安装 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆仓库
git clone <repository-url>
cd 20260110.chaser-oxide-server

# 检查代码
cargo check

# 运行测试
cargo test

# 运行特定测试
cargo test --test e2e_test test_browser_lifecycle

# 格式化代码
cargo fmt

# 代码检查
cargo clippy
```

### 项目结构

```
chaser-oxide-server/
├── protos/                 # Protocol Buffers 定义和生成代码
│   ├── browser.proto
│   ├── page.proto
│   ├── element.proto
│   ├── event.proto
│   ├── profile.proto
│   ├── common.proto
│   ├── python/             # Python gRPC 生成代码
│   │   ├── *_pb2.py
│   │   └── *_pb2_grpc.py
│   └── go/                 # Go gRPC 生成代码
│       ├── *_pb.go
│       └── *_grpc.pb.go
├── src/                    # Rust 源代码
│   ├── main.rs             # 服务器入口
│   ├── lib.rs              # 库入口
│   ├── cdp/                # Chrome DevTools Protocol 实现
│   │   ├── browser.rs      # CDP 浏览器
│   │   ├── client.rs       # CDP 客户端
│   │   ├── connection.rs   # WebSocket 连接
│   │   ├── mock.rs         # Mock 实现（测试）
│   └── traits.rs           # Trait 定义
│   ├── session/            # 会话管理层
│   │   ├── manager.rs      # 会话管理器
│   │   ├── browser.rs      # 浏览器上下文
│   │   ├── page.rs         # 页面上下文
│   │   ├── mock.rs         # Mock 实现（测试）
│   │   └── traits.rs       # Trait 定义
│   ├── services/           # gRPC 服务层
│   │   ├── browser/        # BrowserService
│   │   ├── page/           # PageService
│   │   ├── element/        # ElementService
│   │   ├── event/          # EventService
│   │   └── profile/        # ProfileService
│   └── stealth/            # 隐身引擎
│       ├── engine.rs       # 指纹生成引擎
│       ├── injector.rs     # 脚本注入器
│       └── traits.rs       # Trait 定义
├── tests/                  # Rust 集成测试
│   ├── common/             # 测试公共模块
│   │   └── mod.rs
│   ├── mock_chrome.rs      # Mock Chrome 服务器
│   └── e2e_test.rs         # 端到端测试
├── docs/                   # 文档目录
│   ├── examples/           # 客户端示例代码
│   │   ├── python/         # Python 客户端示例
│   │   │   ├── basic_client.py
│   │   │   └── stealth_client.py
│   │   └── go/             # Go 客户端示例
│   ├── api/                # API 文档
│   ├── guides/             # 使用指南
│   ├── implementation/     # 实现文档
│   └── reports/            # 测试报告
├── .github/                # GitHub 配置
│   └── workflows/          # CI/CD 工作流
├── Cargo.toml              # Rust 项目配置
├── Dockerfile              # Docker 镜像构建
├── docker-compose.yml      # Docker Compose 配置
├── CHANGELOG.md            # 版本变更日志
├── LICENSE                 # MIT 许可证
└── README.md               # 项目说明
```

详细的开发指南请参阅 [开发指南](docs/guides/development.md)。

### 本地 CI 验证

在提交代码前，建议运行以下命令以验证代码质量：

```bash
# 格式检查
cargo fmt --check

# 代码检查
cargo clippy -- -D warnings

# 运行测试
cargo test

# 构建发布版本
cargo build --release

# 运行完整 CI 检查（一键验证）
cargo fmt --check && cargo clippy -- -D warnings && cargo test && cargo build --release
```

## 测试

```bash
# 运行所有测试
cargo test

# 运行 E2E 测试
cargo test --test e2e_test

# 运行 Mock Chrome 测试
cargo test --test mock_chrome

# 运行单元测试并显示输出
cargo test -- --nocapture

# 运行测试并生成覆盖率报告
cargo tarpaulin --out Html
```

## 监控和日志

### 日志级别

服务使用 `tracing` 库进行结构化日志记录，支持以下日志级别：

- **trace** - 最详细的日志信息
- **debug** - 调试信息
- **info** - 一般信息（默认）
- **warn** - 警告信息
- **error** - 错误信息

### 日志输出示例

```
2024-01-10T10:30:45.123Z INFO chaser_oxide::server: Starting gRPC server
2024-01-10T10:30:45.456Z INFO chaser_oxide::server: Server listening on 127.0.0.1:50051
2024-01-10T10:30:50.789Z INFO chaser_oxide::browser: Browser launched: browser-uuid-1234
2024-01-10T10:30:51.012Z INFO chaser_oxide::page: Page created: page-uuid-5678
```

## 故障排查

### 问题 1: 无法连接到 Chrome

**错误信息**: `Failed to connect to Chrome DevTools Protocol`

**解决方案**:
1. 确保 Chrome 以远程调试模式启动
2. 检查 `CHASER_CDP_ENDPOINT` 环境变量是否正确
3. 验证端口 9222 是否可访问

### 问题 2: 浏览器启动失败

**错误信息**: `Failed to launch browser`

**解决方案**:
1. 检查 Chrome 可执行文件路径
2. 确保有足够的系统资源（内存、CPU）
3. 查看日志了解详细错误信息

### 问题 3: 元素查找失败

**错误信息**: `Element not found`

**解决方案**:
1. 确保选择器正确
2. 等待页面完全加载
3. 使用 `WaitFor` 方法等待元素出现

## 贡献指南

我们欢迎任何形式的贡献！请参阅 [贡献指南](docs/guides/contributing.md) 了解详细的贡献流程。

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 相关文档

### 用户文档
- [API 使用文档](docs/api/api.md) - 完整的 gRPC API 参考
- [部署指南](docs/guides/deployment.md) - 部署架构和运维指南
- [Docker 部署](docs/guides/docker.md) - 容器化部署说明

### 开发文档
- [开发指南](docs/guides/development.md) - 开发环境设置和规范
- [贡献指南](docs/guides/contributing.md) - 贡献流程和提交清单

### 设计文档
- [架构设计](docs/architecture.md) - 系统架构设计文档
- [API 设计](docs/api-design.md) - API 接口设计说明
- [实现计划](docs/implementation-plan.md) - 功能实现路线图
- [性能优化指南](docs/performance-optimization-guide.md) - 性能调优指南

### 实现文档
- [浏览器/页面实现](docs/implementation/browser-page.md) - 浏览器和页面服务实现
- [CDP 实现](docs/implementation/cdp.md) - Chrome DevTools Protocol 实现
- [隐身配置实现](docs/implementation/profile-stealth.md) - 隐身引擎实现
- [实现总结](docs/implementation/summary.md) - 项目实现总结

### 测试报告
- [验收测试报告](docs/reports/acceptance-test.md) - 功能和安全测试结果
- [性能测试报告](test_reports/latest_acceptance_test.md) - 性能和稳定性测试

### 其他
- [CHANGELOG.md](CHANGELOG.md) - 版本变更日志
- [LICENSE](LICENSE) - MIT 许可证

## 联系方式

- 问题反馈: [GitHub Issues](https://github.com/your-org/chaser-oxide-server/issues)

## 致谢

感谢以下开源项目：

- [ccheshirecat/chaser-oxide](https://github.com/ccheshirecat/chaser-oxide) - 本项目的基础，提供浏览器自动化核心架构
- [Tokio](https://tokio.rs/) - Rust 异步运行时
- [Tonic](https://github.com/hyperium/tonic) - gRPC 实现
- [Chromium](https://www.chromium.org/) - 开源浏览器
