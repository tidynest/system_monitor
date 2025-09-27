# System Monitor Dashboard

A high-performance, real-time system monitoring dashboard built with Rust and Actix-web. Monitor CPU, memory, disk, network, and process metrics through an intuitive web interface with live updates via Server-Sent Events (SSE).

## Features

### Core Monitoring Capabilities
- **CPU Metrics**: Real-time usage percentage, per-core utilization, frequency, and model information
- **Memory Tracking**: RAM and swap usage with available/used breakdown
- **Disk Analytics**: Usage statistics for all mounted filesystems with capacity visualization
- **Network Statistics**: Interface-level traffic monitoring with aggregate bandwidth tracking
- **Process Management**: Top consumers by CPU and memory with automatic deduplication

### Technical Highlights
- **Real-time Updates**: Server-Sent Events deliver metrics every second without polling overhead
- **Resource Efficient**: Singleton system state management minimizes collection overhead
- **Production Ready**: Comprehensive error handling, logging, and configurable deployment options
- **Zero JavaScript Dependencies**: Pure HTMX for dynamic content with minimal client footprint

## Installation

### Prerequisites
- Rust 1.70+ (2021 edition)
- Cargo build system

### Build from Source
```bash
# Clone the repository
git clone https://github.com/yourusername/system-monitor.git
cd system-monitor

# Build in release mode for optimal performance
cargo build --release

# Run the application
cargo run --release
```

The dashboard will be accessible at `http://localhost:8080`

## Configuration

Configure the application through environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `MONITOR_HOST` | `127.0.0.1` | Server bind address |
| `MONITOR_PORT` | `8080` | Server port |
| `MONITOR_UPDATE_INTERVAL` | `1` | Metrics refresh rate (seconds) |
| `MONITOR_MAX_PROCESSES` | `5` | Number of top processes to display |
| `MONITOR_LOG_LEVEL` | `warn` | Logging verbosity (error/warn/info/debug) |

### Example Configuration
```bash
export MONITOR_PORT=3000
export MONITOR_UPDATE_INTERVAL=2
export MONITOR_LOG_LEVEL=info
cargo run --release
```

## Architecture

### Project Structure
```
.
├── Cargo.lock              # Dependency lock file (tracked for binaries)
├── Cargo.toml              # Project manifest and dependencies  
├── LICENSE                 # MIT License file
├── README.md               # This documentation file
├── .gitignore              # Git ignore rules for build artifacts and temp files
├── src/                    # Source code directory
│   ├── main.rs             # Application entry point - HTTP server setup
│   ├── lib.rs              # Library root - public API exports
│   ├── collectors/         # System metric collection implementations
│   │   ├── mod.rs          # Collector module exports
│   │   ├── cpu.rs          # CPU usage and frequency collector
│   │   ├── memory.rs       # RAM and swap memory collector
│   │   ├── disk.rs         # Filesystem usage collector
│   │   ├── network.rs      # Network interface statistics collector
│   │   ├── process.rs      # Process list and top consumers collector
│   │   └── system.rs       # Orchestrates all collectors with singleton state
│   ├── models/             # Data structures for metrics (Serde serializable)
│   │   ├── mod.rs          # Model exports and type aliases
│   │   ├── cpu.rs          # CpuMetrics struct definition
│   │   ├── memory.rs       # MemoryMetrics struct definition
│   │   ├── disk.rs         # DiskMetrics struct definition
│   │   ├── network.rs      # NetworkMetrics and NetworkInterface structs
│   │   ├── process.rs      # ProcessMetrics and ProcessInfo structs
│   │   └── system.rs       # SystemMetrics aggregate struct
│   ├── routes/             # HTTP endpoint handlers and responses
│   │   ├── mod.rs          # Route configuration and registration
│   │   ├── dashboard.rs    # Serves dashboard.html static content
│   │   └── metrics.rs      # SSE stream and HTMX partial endpoints
│   ├── services/           # Service layer for dependency injection
│   │   ├── mod.rs          # Service module exports
│   │   └── metrics_service.rs  # MetricsService trait and implementations
│   ├── config/             # Application configuration
│   │   └── mod.rs          # Config struct and environment variable parsing
│   └── utils/              # Utility functions and formatters
│       └── mod.rs          # format_bytes() and format_uptime() helpers
└── static/                 # Static web assets
    └── html/               # HTML templates
        └── dashboard.html  # Main dashboard interface with HTMX
```

**Source Code Overview:**
- [**main.rs**](src/main.rs): Entry point - sets up Actix-web server, middleware, and routes
- [**collectors/**](src/collectors/): Platform-specific implementations using `sysinfo` crate
  - [cpu.rs](src/collectors/cpu.rs): Global and per-core CPU usage tracking
  - [memory.rs](src/collectors/memory.rs): RAM and swap utilization
  - [disk.rs](src/collectors/disk.rs): Filesystem usage statistics
  - [network.rs](src/collectors/network.rs): Interface traffic monitoring
  - [process.rs](src/collectors/process.rs): Top resource consumers with deduplication
  - [system.rs](src/collectors/system.rs): Singleton state management for efficiency
- [**models/**](src/models/): Serde-serializable data structures for JSON API
- [**routes/**](src/routes/): HTTP request handlers
  - [metrics.rs](src/routes/metrics.rs): SSE stream with compression bypass fix
  - [dashboard.rs](src/routes/dashboard.rs): Static HTML serving
- [**services/**](src/services/): Abstraction layer enabling dependency injection
- [**config/**](src/config/): Environment-based configuration with sensible defaults
- [**utils/**](src/utils/): Human-readable formatting for bytes and uptime

### Key Components

- **Collectors**: Platform-specific metric gathering using the `sysinfo` crate
- **SSE Stream**: Real-time push updates with automatic compression bypass
- **Service Layer**: Abstraction for testing and modularity
- **State Management**: Efficient singleton pattern for system resource access

### Technology Stack
- **Backend**: Rust, Actix-web 4.x
- **System Info**: sysinfo crate for cross-platform metrics
- **Frontend**: HTMX for dynamic updates, pure CSS for styling
- **Serialization**: Serde with JSON for data exchange

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Main dashboard interface |
| `/metrics/stream` | GET | SSE endpoint for real-time metrics |
| `/metrics/processes` | GET | Process list (CPU or memory sorted) |
| `/metrics/disks` | GET | Disk usage information |
| `/metrics/network` | GET | Network interface statistics |

## Development

### Running Tests
```bash
# Run all tests
cargo test

# Run with logging output
RUST_LOG=debug cargo test -- --nocapture

# Run specific test module
cargo test collectors::tests
```

### Building Documentation
```bash
cargo doc --no-deps --open
```

### Performance Profiling
The application includes built-in timing for metric collection. Enable debug logging to see collection performance:
```bash
MONITOR_LOG_LEVEL=debug cargo run
```

## Troubleshooting

### SSE Connection Issues
If real-time updates aren't working:
1. Ensure no proxy/reverse proxy is buffering responses
2. Check browser console for connection errors
3. Verify compression middleware bypass is active

### High CPU Usage
- Increase `MONITOR_UPDATE_INTERVAL` to reduce collection frequency
- Check for runaway processes in the process list

### Permission Errors
Some system metrics may require elevated permissions on certain platforms. The application will gracefully degrade if specific metrics are unavailable.

## Contributing

Contributions are warmly welcomed! This project thrives on community input and collaboration.

### How to Contribute
1. **Open an issue first** for major changes to discuss the approach
2. Fork the repository and create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass with `cargo test`
5. Submit a pull request referencing the issue

### Development Guidelines
- Follow Rust standard formatting with `cargo fmt`
- Lint code with `cargo clippy`
- Document public APIs with rustdoc comments
- Keep commits focused and atomic

## Roadmap

The following enhancements are planned for future releases. See the [Issues](https://github.com/yourusername/system-monitor/issues) page for detailed discussions:

### Performance & Efficiency
- **Configurable metric collection** (#1): Allow users to disable specific collectors for reduced overhead
- **WebSocket support** (#2): Alternative to SSE for bidirectional communication and better proxy compatibility
- **Metric history storage** (#3): SQLite integration for historical data and trend analysis

### Features
- **Alert system** (#4): Configurable thresholds with email/webhook notifications
- **Docker monitoring** (#5): Container-specific metrics and resource usage
- **GPU metrics** (#6): NVIDIA/AMD GPU utilization and temperature monitoring
- **Custom dashboards** (#7): User-configurable layouts and metric selections

### Platform Support
- **Windows service integration** (#8): Native Windows service installation
- **macOS launchd support** (#9): Automatic startup on macOS
- **ARM optimization** (#10): Raspberry Pi and ARM server optimizations

### UI/UX Improvements
- **Dark/light theme toggle** (#11): User preference persistence
- **Export capabilities** (#12): CSV/JSON export for metric data
- **Mobile responsive design** (#13): Optimized layouts for mobile devices

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Actix-web](https://actix.rs/) - A powerful, pragmatic web framework
- System metrics via [sysinfo](https://github.com/GuillaumeGomez/sysinfo) - Cross-platform system information
- Dynamic updates powered by [HTMX](https://htmx.org/) - High power tools for HTML

---

**Current Version**: v1.0.2 | **Minimum Rust Version**: 1.70 | **Platform Support**: Linux, macOS, Windows