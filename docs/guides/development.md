# Chaser-Oxide 开发指南

本文档详细说明 Chaser-Oxide Server 的开发环境设置、代码结构、开发规范和贡献流程。

## 目录

- [开发环境设置](#开发环境设置)
- [项目结构](#项目结构)
- [开发规范](#开发规范)
- [测试指南](#测试指南)
- [调试技巧](#调试技巧)
- [性能分析](#性能分析)
- [贡献流程](#贡献流程)

## 开发环境设置

### 前置要求

- **Rust**: 1.70 或更高版本
- **Chrome/Chromium**: 90 或更高版本
- **Git**: 版本控制
- **Make**: 构建工具（可选）

### 安装 Rust 工具链

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置环境
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### 克隆项目

```bash
# 克隆仓库
git clone <repository-url>
cd 20260110.chaser-oxide-server

# 安装依赖
cargo fetch
```

### 安装开发工具

```bash
# 安装 Rust 格式化工具
rustup component add rustfmt

# 安装 Rust 代码检查工具
rustup component add clippy

# 安装代码覆盖率工具
cargo install cargo-tarpaulin

# 安装文档生成工具
cargo install cargo-doc
```

### 配置 IDE

#### VS Code

推荐安装以下扩展：

- **rust-analyzer** - Rust 语言支持
- **CodeLLDB** - 调试支持
- **Even Better TOML** - TOML 文件支持
- **Protobuf** - Protocol Buffers 支持
- **GitLens** - Git 增强

配置 `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "files.watcherExclude": {
    "**/target": true
  }
}
```

#### IntelliJ IDEA

安装 **Rust 插件**并配置：

1. 启用外部 Linter：Settings → Rust → External Linter → Clippy
2. 启用格式化工具：Settings → Rust → Rustfmt
3. 配置测试运行器：Settings → Rust → Tests

## 项目结构

### 目录结构

```
chaser-oxide-server/
├── protos/                    # Protocol Buffers 定义
│   ├── browser.proto         # Browser 服务
│   ├── page.proto            # Page 服务
│   ├── element.proto         # Element 服务
│   ├── event.proto           # Event 服务
│   ├── profile.proto         # Profile 服务
│   └── common.proto          # 通用消息定义
├── src/
│   ├── main.rs              # 服务器入口
│   ├── lib.rs               # 库入口
│   ├── browser/             # Browser 服务实现
│   │   ├── mod.rs
│   │   ├── service.rs       # gRPC 服务
│   │   └── options.rs       # 启动选项
│   ├── page/                # Page 服务实现
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   └── navigation.rs    # 导航逻辑
│   ├── element/             # Element 服务实现
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   └── selector.rs      # 选择器实现
│   ├── event/               # Event 服务实现
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   └── dispatcher.rs    # 事件分发器
│   ├── profile/             # Profile 服务实现
│   │   ├── mod.rs
│   │   ├── service.rs
│   │   └── fingerprint.rs   # 指纹生成
│   ├── session/             # Session 管理
│   │   ├── mod.rs
│   │   ├── manager.rs       # Session Manager
│   │   └── manager_impl.rs  # Session Manager 实现
│   ├── cdp/                 # CDP 实现
│   │   ├── mod.rs
│   │   ├── traits.rs        # CDP 特征定义
│   │   ├── browser.rs       # CDP Browser
│   │   ├── page.rs          # CDP Page
│   │   └── element.rs       # CDP Element
│   ├── stealth/             # 隐身引擎
│   │   ├── mod.rs
│   │   ├── engine.rs        # 隐身引擎
│   │   ├── injector.rs      # 脚本注入
│   │   └── simulator.rs     # 行为模拟
│   ├── config/              # 配置管理
│   │   ├── mod.rs
│   │   └── config.rs
│   ├── error/               # 错误类型
│   │   ├── mod.rs
│   │   └── error.rs
│   └── utils/               # 工具函数
│       ├── mod.rs
│       └── helpers.rs
├── tests/                   # 集成测试
│   ├── common/
│   │   └── mod.rs          # 测试辅助函数
│   ├── mock_chrome.rs      # Mock Chrome 服务器
│   └── e2e_test.rs         # E2E 测试
├── docs/                    # 设计文档
│   ├── architecture.md
│   ├── api-design.md
│   └── implementation-plan.md
├── build.rs                 # 构建脚本
├── Cargo.toml              # 项目配置
└── README.md
```

### 代码组织原则

1. **模块化**: 每个服务独立模块
2. **分层清晰**: gRPC 服务 → 业务逻辑 → CDP 实现
3. **抽象隔离**: 使用 trait 定义接口
4. **职责单一**: 每个模块只负责一个功能

## 开发规范

### 代码风格

使用 `rustfmt` 进行代码格式化：

```bash
# 格式化所有代码
cargo fmt

# 检查格式
cargo fmt --check
```

配置 `.rustfmt.toml`:

```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
```

### 代码检查

使用 `clippy` 进行代码检查：

```bash
# 运行 clippy
cargo clippy

# 修复可自动修复的问题
cargo clippy --fix

# 检查所有目标
cargo clippy --all-targets --all-features
```

### 命名规范

遵循 Rust 命名规范：

| 类型 | 规范 | 示例 |
|------|------|------|
| 模块 | `snake_case` | `mod browser_service` |
| 类型 | `PascalCase` | `struct BrowserService` |
| 函数 | `snake_case` | `fn launch_browser()` |
| 常量 | `SCREAMING_SNAKE_CASE` | `const MAX_BROWSERS: usize` |
| 泛型 | `PascalCase` | `fn process<T>()` |

### 注释规范

#### 文档注释

使用 `///` 或 `//!` 添加文档注释：

```rust
/// Launches a new browser instance with the specified options.
///
/// # Arguments
///
/// * `options` - Browser launch options
///
/// # Returns
///
/// * `Result<BrowserId>` - Browser ID on success
///
/// # Errors
///
/// * `Error::BrowserLaunchFailed` - If browser fails to launch
///
/// # Examples
///
/// ```no_run
/// use chaser_oxide::BrowserService;
///
/// let service = BrowserService::new();
/// let browser_id = service.launch(BrowserOptions::default()).await?;
/// ```
pub async fn launch(&self, options: BrowserOptions) -> Result<BrowserId> {
    // 实现
}
```

#### 代码注释

使用 `//` 添加行内注释：

```rust
// Check if browser limit is reached
if self.active_browsers() >= self.max_browsers {
    return Err(Error::ResourceExhausted("Max browser limit reached"));
}

// Create new browser instance
let browser = Browser::new(options)?;
```

### 错误处理

使用 `thiserror` 定义错误类型：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChaserError {
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("Browser not found: {0}")]
    BrowserNotFound(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("CDP error: {0}")]
    CdpError(#[from] CdpError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

### 异步编程

使用 `tokio` 运行时：

```rust
use tokio::time::{timeout, Duration};

/// 超时控制
pub async fn navigate_with_timeout(&self, url: &str) -> Result<()> {
    timeout(
        Duration::from_secs(30),
        self.navigate(url)
    ).await?
}

/// 并发操作
pub async fn batch_navigate(&self, urls: Vec<String>) -> Vec<Result<()>> {
    let futures: Vec<_> = urls
        .into_iter()
        .map(|url| self.navigate(&url))
        .collect();

    futures::future::join_all(futures).await
}
```

### 日志记录

使用 `tracing` 进行结构化日志：

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn launch_browser(&self, options: BrowserOptions) -> Result<BrowserId> {
    info!(?options, "Launching browser");

    match self.do_launch(options).await {
        Ok(id) => {
            info!(browser_id = %id, "Browser launched successfully");
            Ok(id)
        }
        Err(e) => {
            error!(error = %e, "Failed to launch browser");
            Err(e)
        }
    }
}
```

### 测试规范

#### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_launch_browser() {
        let service = BrowserService::new();
        let result = service.launch(BrowserOptions::default()).await;

        assert!(result.is_ok());
        let browser_id = result.unwrap();
        assert!(!browser_id.value.is_empty());
    }

    #[tokio::test]
    async fn test_browser_limit() {
        let service = BrowserService::with_max_browsers(1);
        service.launch(BrowserOptions::default()).await.unwrap();

        let result = service.launch(BrowserOptions::default()).await;
        assert!(matches!(result, Err(ChaserError::ResourceExhausted(_))));
    }
}
```

#### 集成测试

```rust
// tests/e2e_test.rs
use chaser_oxide::BrowserService;

#[tokio::test]
async fn test_full_workflow() {
    let service = BrowserService::new();

    // 1. 启动浏览器
    let browser_id = service.launch(BrowserOptions::default()).await.unwrap();

    // 2. 创建页面
    let page_id = service.create_page(browser_id).await.unwrap();

    // 3. 导航
    service.navigate(page_id, "https://example.com").await.unwrap();

    // 4. 清理
    service.close_browser(browser_id).await.unwrap();
}
```

## 测试指南

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行单元测试
cargo test --lib

# 运行集成测试
cargo test --test e2e_test

# 运行特定测试
cargo test test_browser_lifecycle

# 显示测试输出
cargo test -- --nocapture

# 并行运行测试
cargo test -- --test-threads=4
```

### 测试覆盖率

```bash
# 生成覆盖率报告
cargo tarpaulin --out Html

# 生成覆盖率报告（包含行覆盖）
cargo tarpaulin --out Html --line-coverage

# 生成覆盖率报告（输出到终端）
cargo tarpaulin --out Stdout
```

### 测试组织

#### 单元测试

放在模块文件底部：

```rust
// src/browser/service.rs
impl BrowserService {
    pub async fn launch(&self, options: BrowserOptions) -> Result<BrowserId> {
        // 实现
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_launch() {
        // 测试
    }
}
```

#### 集成测试

放在 `tests/` 目录：

```rust
// tests/integration_test.rs
use chaser_oxide::BrowserService;

#[tokio::test]
async fn test_browser_service_integration() {
    // 集成测试
}
```

### Mock 测试

使用 Mock Chrome 进行测试：

```rust
// tests/mock_chrome.rs
use tokio_tungstenite::WebSocketStream;

pub struct MockChrome {
    server: TcpListener,
}

impl MockChrome {
    pub async fn start() -> Result<Self> {
        let server = TcpListener::bind("127.0.0.1:0").await?;
        Ok(MockChrome { server })
    }

    pub fn endpoint(&self) -> String {
        let addr = self.server.local_addr().unwrap();
        format!("ws://{}", addr)
    }
}

// tests/e2e_test.rs
#[tokio::test]
async fn test_with_mock_chrome() {
    let mock_chrome = MockChrome::start().await.unwrap();
    let service = BrowserService::with_cdp_endpoint(mock_chrome.endpoint());

    // 测试逻辑
}
```

## 调试技巧

### 日志调试

```bash
# 启用调试日志
RUST_LOG=chaser_oxide=debug cargo run

# 启用追踪日志
RUST_LOG=chaser_oxide=trace cargo run

# 启用特定模块日志
RUST_LOG=chaser_oxide::browser=debug cargo run
```

### VS Code 调试

配置 `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug unit tests",
      "cargo": {
        "args": [
          "test",
          "--no-run",
          "--bin=chaser-oxide-server"
        ]
      },
      "cwd": "${workspaceFolder}"
    },
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug example",
      "cargo": {
        "args": [
          "build",
          "--example=example"
        ]
      },
      "cwd": "${workspaceFolder}"
    }
  ]
}
```

### GDB 调试

```bash
# 编译调试版本
cargo build

# 使用 GDB 调试
gdb target/debug/chaser-oxide-server

# GDB 常用命令
(gdb) break main            # 设置断点
(gdb) run                   # 运行程序
(gdb) next                  # 单步执行
(gdb) print variable        # 打印变量
(gdb) backtrace             # 查看调用栈
```

### 性能分析

#### CPU 性能分析

```bash
# 使用 flamegraph
cargo install flamegraph
cargo flamegraph --bin chaser-oxide-server

# 使用 perf (Linux)
perf record -g target/release/chaser-oxide-server
perf report
```

#### 内存分析

```bash
# 使用 heaptrack
cargo install heaptrack
heaptrack target/release/chaser-oxide-server

# 分析堆使用
heaptrack_print heaptrack.log
```

### 网络调试

```bash
# 抓取 gRPC 流量
tcpdump -i lo port 50051 -w grpc.pcap

# 使用 Wireshark 分析
wireshark grpc.pcap

# 查看 CDP 通信
wscat -c ws://localhost:9222
```

## 性能分析

### 基准测试

```rust
// benches/browser_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_launch(c: &mut Criterion) {
    c.bench_function("launch_browser", |b| {
        b.iter(|| {
            let service = BrowserService::new();
            black_box(service.launch(BrowserOptions::default()))
        })
    });
}

criterion_group!(benches, benchmark_launch);
criterion_main!(benches);
```

运行基准测试：

```bash
cargo bench
```

### 性能优化建议

1. **减少克隆**: 使用引用或 `Arc`
2. **异步并发**: 使用 `tokio::spawn`
3. **连接复用**: 保持连接池
4. **批量操作**: 合并多个请求
5. **缓存结果**: 避免重复计算

## 贡献流程

### Fork 仓库

1. Fork 项目仓库
2. 克隆到本地

```bash
git clone https://github.com/your-username/chaser-oxide-server.git
cd chaser-oxide-server
```

### 创建分支

```bash
# 更新主分支
git checkout master
git pull upstream master

# 创建功能分支
git checkout -b feature/your-feature-name
```

### 开发流程

1. **编写代码**
   ```bash
   # 创建功能分支
   git checkout -b feature/add-new-api

   # 编写代码
   # ...

   # 格式化代码
   cargo fmt

   # 代码检查
   cargo clippy
   ```

2. **编写测试**
   ```bash
   # 运行测试
   cargo test

   # 检查覆盖率
   cargo tarpaulin --out Html
   ```

3. **编写文档**
   ```bash
   # 生成文档
   cargo doc --open

   # 检查文档链接
   cargo doc --document-private-items
   ```

### 提交代码

```bash
# 查看更改
git status
git diff

# 暂存文件
git add .

# 提交更改
git commit -m "feat: add new API for browser automation"

# 推送到远程
git push origin feature/add-new-api
```

### 提交信息规范

遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**类型 (type)**:
- `feat`: 新功能
- `fix`: Bug 修复
- `docs`: 文档更新
- `style`: 代码格式（不影响功能）
- `refactor`: 重构
- `perf`: 性能优化
- `test`: 测试相关
- `chore`: 构建/工具相关

**示例**:

```
feat(browser): add headless mode support

Add support for running browsers in headless mode with
configurable window size and device scale factor.

Closes #123
```

### Pull Request

1. 创建 Pull Request
2. 填写 PR 模板

```markdown
## Description
Brief description of the changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review performed
- [ ] Comments added to complex code
- [ ] Documentation updated
- [ ] No new warnings generated
- [ ] Tests pass locally
```

3. 等待代码审查
4. 根据反馈修改
5. 合并到主分支

### 代码审查

审查要点：

- **功能**: 是否实现预期功能
- **测试**: 是否有充分的测试
- **文档**: 是否更新了文档
- **风格**: 是否符合代码规范
- **性能**: 是否有性能问题
- **安全**: 是否有安全漏洞

## 发布流程

### 版本号

遵循 [Semantic Versioning](https://semver.org/):

- `MAJOR.MINOR.PATCH`
- MAJOR: 不兼容的 API 变更
- MINOR: 向后兼容的功能新增
- PATCH: 向后兼容的 Bug 修复

### 发布步骤

1. **更新版本号**

```toml
# Cargo.toml
[package]
version = "0.2.0"
```

2. **更新 CHANGELOG**

```markdown
## [0.2.0] - 2024-01-15

### Added
- New API for browser automation
- Support for headless mode

### Fixed
- Fixed memory leak in session manager
```

3. **创建 Git 标签**

```bash
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0
```

4. **发布到 crates.io**（可选）

```bash
cargo publish
```

## 相关文档

- [README.md](README.md) - 项目介绍
- [API.md](API.md) - API 使用文档
- [DEPLOYMENT.md](DEPLOYMENT.md) - 部署指南
- [docs/architecture.md](docs/architecture.md) - 架构设计
- [docs/api-design.md](docs/api-design.md) - API 设计

## 获取帮助

- **GitHub Issues**: [报告问题](https://github.com/your-org/chaser-oxide-server/issues)
- **Discussions**: [讨论](https://github.com/your-org/chaser-oxide-server/discussions)
- **邮件**: dev@chaser-oxide.com

感谢您的贡献！
