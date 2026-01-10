# Changelog

All notable changes to Chaser-Oxide Server will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Planned
- Distributed session management
- GraphQL API support
- Advanced fingerprint randomization
- Performance optimizations

## [0.1.0] - 2026-01-10

### Added

#### Core Features
- gRPC server with 5 core services:
  - **BrowserService**: Browser lifecycle management
  - **PageService**: Page operations and navigation
  - **ElementService**: Element finding and interaction
  - **EventService**: Real-time event subscription
  - **ProfileService**: Stealth profile management
- Chrome DevTools Protocol (CDP) integration
- Session management with automatic cleanup
- Graceful shutdown mechanism

#### Browser Service
- `Launch` - Launch new browser instances
- `GetPages` - List all pages in a browser
- `Close` - Close browser instances
- `GetVersion` - Get browser version information
- `GetStatus` - Get browser runtime status
- `Connect` - Connect to existing Chrome instances

#### Page Service
- `CreatePage` - Create new pages
- `Navigate` - Navigate to URLs with configurable wait states
- `GetSnapshot` - Get accessibility tree snapshot
- `Screenshot` - Capture screenshots (PNG, JPEG, WebP)
- `Evaluate` - Execute JavaScript in page context
- `SetContent` - Set page HTML content
- `GetContent` - Get page HTML content
- `Reload` - Reload pages
- `GoBack` / `GoForward` - Navigate browser history
- `SetViewport` - Configure viewport size
- `EmulateDevice` - Emulate mobile devices
- `ClosePage` - Close pages
- `WaitFor` - Wait for page conditions
- Cookie management (GetCookies, SetCookies, ClearCookies)

#### Element Service
- `FindElement` / `FindElements` - Find elements by CSS/XPath
- `Click` - Click elements with human-like movement
- `Type` - Type text with realistic delays
- `Fill` - Fill form fields
- `GetAttribute` / `GetAttributes` - Get element attributes
- `GetText` - Get element text content
- `Hover` - Hover over elements
- `Focus` - Focus elements
- `SelectOption` - Select dropdown options
- `UploadFile` - Upload files
- `ScrollIntoView` - Scroll elements into view
- `GetBoundingBox` - Get element position and size
- `IsVisible` / `IsEnabled` - Check element state
- `WaitForElement` - Wait for element conditions
- `PressKey` - Send keyboard events
- `DragAndDrop` - Drag and drop elements

#### Event Service
- `Subscribe` - Bidirectional streaming for real-time events
- Event types:
  - PAGE_CREATED, PAGE_LOADED, PAGE_NAVIGATED, PAGE_CLOSED
  - CONSOLE_LOG, CONSOLE_ERROR
  - REQUEST_SENT, RESPONSE_RECEIVED
  - JS_EXCEPTION
  - DIALOG_OPENED
- Event filtering by URL pattern and resource type

#### Profile Service
- `CreateProfile` - Create fingerprint profiles
- `ApplyProfile` - Apply profiles to pages
- `GetPresets` - List predefined profile types
- `GetActiveProfile` - Get current active profile
- `CreateCustomProfile` - Create custom fingerprints
- `RandomizeProfile` - Randomize profile parameters

#### Stealth Features
- Navigator property spoofing (platform, vendor, hardwareConcurrency)
- WebGL fingerprint randomization
- Canvas fingerprint protection with noise injection
- Transport layer stealth (isolated worlds)
- Human behavior simulation:
  - Bezier curve mouse movements
  - Realistic typing patterns with random delays
  - Typing error simulation

#### Configuration
- Environment-based configuration
- Config file support (TOML)
- Configurable resource limits:
  - Max browsers
  - Max pages per browser
  - Session timeout
  - Cleanup interval

#### Logging & Monitoring
- Structured logging with `tracing`
- Configurable log levels
- Prometheus metrics:
  - Request count and duration
  - Active browsers and pages
  - Error tracking
- Health check endpoint

#### Testing
- Comprehensive test suite:
  - 16 E2E tests
  - Mock Chrome server for isolated testing
  - Test helper functions
- Test coverage for:
  - Browser lifecycle
  - Page operations
  - Element interaction
  - Concurrent operations
  - Error handling
  - Session cleanup

#### Documentation
- README with quick start guide
- API documentation with examples
- Deployment guide (Docker, Docker Compose, Kubernetes)
- Development guide with contribution guidelines
- Architecture design document
- API design document

### Technical Details

#### Dependencies
- **Runtime**: Tokio 1.0 (async runtime)
- **gRPC**: tonic 0.12, prost 0.13
- **WebSocket**: tokio-tungstenite 0.24
- **Serialization**: serde 1.0, serde_json 1.0
- **Error Handling**: thiserror 2.0, anyhow 1.0
- **Logging**: tracing 0.1, tracing-subscriber 0.3
- **UUID**: uuid 1.0
- **Configuration**: config 0.15, toml 0.8
- **Stealth**: rand 0.8, bezier-rs 0.3
- **Time**: chrono 0.4
- **Testing**: tokio-test 0.4, futures-util 0.3

#### Architecture
- Layered architecture:
  - API Gateway (gRPC Server)
  - Service Layer (5 services)
  - Business Logic (Session Manager, Event Dispatcher, Stealth Engine)
  - Data Access (Chaser-Oxide Core)
  - Chrome Browser (CDP)
- Thread-safe design with Arc<RwLock>
- Async/await throughout
- Resource management with automatic cleanup

### Performance
- High concurrency support
- HTTP/2 multiplexing
- Connection pooling
- Efficient memory usage with weak references
- Configurable resource limits

### Security
- Sandbox isolation for browser processes
- Configurable proxy support
- TLS/SSL ready
- Input validation through Protocol Buffers
- Error handling without information leakage

### Known Issues
- DeviceType enum variant naming issues in page/service.rs (non-blocking)
- Requires manual Chrome startup for remote debugging

### Migration Notes
- First release - no migration needed

## [0.0.1] - 2024-01-05

### Added
- Initial project setup
- Basic gRPC service structure
- Protocol Buffer definitions
- Core CDP integration

---

## Version History Summary

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-01-10 | Initial stable release with full feature set |
| 0.0.1 | 2024-01-05 | Initial project setup |

## Release Notes

### 0.1.0 Highlights

This is the first stable release of Chaser-Oxide Server, providing a complete browser automation solution with the following key features:

- **Full gRPC API**: 5 services covering browser, page, element, event, and profile operations
- **Stealth Capabilities**: Advanced fingerprint randomization and human behavior simulation
- **Production Ready**: Comprehensive testing, monitoring, and documentation
- **Easy Deployment**: Docker, Docker Compose, and Kubernetes support
- **Developer Friendly**: Clear documentation and contribution guidelines

### Breaking Changes

None (first release)

### Upgrade Instructions

No upgrade needed for first release.

### Deprecations

None

## Future Roadmap

### 0.2.0 (Planned)
- Distributed session management with Redis
- GraphQL API support
- Advanced fingerprint randomization
- Performance optimizations (connection pooling, caching)
- Enhanced monitoring and observability

### 0.3.0 (Planned)
- Plugin system for custom extensions
- Multi-cloud deployment support
- Advanced stealth techniques
- Machine learning-based behavior simulation

### 1.0.0 (Future)
- Full API stability guarantees
- Complete test coverage (100%)
- Enterprise support
- SLA guarantees

## Contributing

We welcome contributions! Please see [DEVELOPMENT.md](DEVELOPMENT.md) for guidelines.

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/chaser-oxide-server/issues)
- **Discussions**: [GitHub Discussions](https://github.com/your-org/chaser-oxide-server/discussions)
- **Email**: support@chaser-oxide.com

## License

MIT License - see [LICENSE](LICENSE) file for details.

---

**Note**: This project is under active development. APIs may change in future releases until version 1.0.0.
