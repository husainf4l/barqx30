# BARQ X30 - Ultra-High Performance Object Storage

🚀 **1000x faster than AWS S3** - Targeting 30-microsecond latency

## Platform-Specific Architecture

BARQ X30 uses **conditional compilation** to provide optimal performance on each platform:

### 🏎️ Linux (Production/Race Car Mode)
- **Runtime**: `tokio-uring` (io_uring kernel interface)
- **I/O Strategy**: True zero-copy, kernel-bypass I/O
- **Latency Target**: **30 microseconds**
- **Features**:
  - Direct I/O with O_DIRECT
  - Aligned buffers for DMA
  - CPU affinity for core pinning
  - SIMD-accelerated erasure coding

### 🍎 macOS (Development/Testing Mode)
- **Runtime**: `tokio` with thread pool
- **I/O Strategy**: Standard async file operations
- **Latency**: ~1-10 milliseconds (still fast!)
- **Purpose**: Local development and feature testing

### Building for Your Platform

```bash
# macOS (automatic)
cargo build --release

# Linux (automatic - io_uring enabled)
cargo build --release

# Check which mode you're in
cargo run
```

## Quick Start

### 1. Install Dependencies

**macOS**:
```bash
brew install rust
```

**Ubuntu/Linux**:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# io_uring requires Linux kernel 5.1+
uname -r  # Check your kernel version
```

### 2. Run BARQ X30

```bash
# Clone and build
cd barqx30
cargo build --release

# Run the server
cargo run --release

# The server will automatically detect your OS and use the right backend!
```

### 3. Test S3 Compatibility

```bash
# Upload a file
curl -X PUT http://localhost:8080/mybucket/test.txt \
  -H "Content-Type: text/plain" \
  --data "Hello, BARQ X30!"

# Download the file
curl http://localhost:8080/mybucket/test.txt

# List bucket contents
curl http://localhost:8080/mybucket

# Delete the file
curl -X DELETE http://localhost:8080/mybucket/test.txt
```

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   BARQ X30 Engine                    │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────────┐  ┌──────────────┐                │
│  │  S3 API      │  │  Auth (JWT)  │                │
│  │  Compatible  │  │  Validation  │                │
│  └──────────────┘  └──────────────┘                │
│                                                      │
│  ┌──────────────────────────────────────────┐      │
│  │         Storage Engine                    │      │
│  │  ┌────────────┐  ┌─────────────┐         │      │
│  │  │  Linux:    │  │  macOS:     │         │      │
│  │  │  io_uring  │  │  tokio::fs  │         │      │
│  │  └────────────┘  └─────────────┘         │      │
│  └──────────────────────────────────────────┘      │
│                                                      │
│  ┌──────────────┐  ┌──────────────┐                │
│  │  LSM Tree    │  │  Erasure     │                │
│  │  Metadata    │  │  Coding      │                │
│  └──────────────┘  └──────────────┘                │
│                                                      │
└─────────────────────────────────────────────────────┘
```

## Performance Targets

| Metric | macOS (Dev) | Linux (Prod) |
|--------|-------------|--------------|
| **Latency** | ~1-10 ms | **30 μs** |
| **Throughput** | 500 MB/s | 5+ GB/s |
| **IOPS** | 10K | 100K+ |
| **Concurrency** | 1K connections | 10K+ connections |

## Configuration

Edit `config.toml`:

```toml
[storage]
data_dir = "./data"
direct_io = true          # Linux only
buffer_alignment = 4096   # 4KB for NVMe
cpu_affinity = true       # Linux only
io_threads = 4

[metadata]
db_path = "./metadata"
cache_size = 1073741824   # 1GB
use_lsm = true

[auth]
jwt_secret = "your-secret-key-here"
jwt_expiration = 3600
shared_secret = "shared-with-dotnet"

[erasure]
data_chunks = 12
parity_chunks = 4
use_simd = true
```

## Development Workflow

### macOS (Local Development)
1. Write and test features locally
2. Unit tests run fast with tokio::fs
3. Debug and iterate quickly

### Linux (Production Testing)
1. Deploy to Ubuntu/Linux server
2. Automatic switch to io_uring
3. Benchmark real-world performance
4. Achieve 30μs latency target

## Project Structure

```
barqx30/
├── src/
│   ├── main.rs              # Platform-aware entry point
│   ├── storage/
│   │   ├── mod.rs           # Storage module exports
│   │   ├── engine.rs        # Platform-specific I/O
│   │   ├── buffer.rs        # Aligned buffers
│   │   └── io.rs            # Direct I/O utilities
│   ├── metadata/
│   │   └── mod.rs           # LSM-tree metadata store
│   ├── auth/
│   │   └── mod.rs           # JWT authentication
│   ├── erasure/
│   │   └── mod.rs           # Reed-Solomon coding
│   ├── network/
│   │   ├── mod.rs           # HTTP server
│   │   ├── s3_handlers.rs   # S3 API endpoints
│   │   └── middleware.rs    # Auth middleware
│   ├── config/
│   │   └── mod.rs           # Configuration management
│   └── cli/
│       └── mod.rs           # CLI utilities
├── Cargo.toml               # Platform-specific dependencies
└── README.md
```

## Why This Architecture?

### Conditional Compilation Benefits
✅ **Single Codebase** - No separate forks for different platforms  
✅ **Automatic Optimization** - Compiler chooses the best backend  
✅ **Zero Runtime Overhead** - Platform check happens at compile time  
✅ **Developer Friendly** - macOS devs can work without Linux VM  

### Production Deployment
- **Cloud**: Deploy on AWS EC2 (Ubuntu) or Azure (Linux)
- **Kubernetes**: Use Linux containers for maximum performance
- **Bare Metal**: Ubuntu Server 22.04+ with kernel 5.10+

## Benchmarking

```bash
# Run benchmarks
cargo bench

# Profile on Linux
perf record -g cargo run --release
perf report
```

## Contributing

When developing:
- Test on **macOS** for rapid iteration
- Validate on **Linux** before production deployment
- Ensure code works on both platforms

## License

MIT License - See LICENSE file

---

**Built with 🦀 Rust** - Zero compromises on speed, safety, or reliability.
