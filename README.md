# RUSSH

A modern, cross-platform SSH client built with Rust and Tauri, featuring P2P connectivity, interactive terminal blocks, and stunning visual effects.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)

## Features

- **Secure SSH Connections** - Password and key-based authentication with encrypted sessions
- **P2P Networking** - Direct peer-to-peer connections with NAT traversal
- **Interactive Terminal** - Full-featured terminal with xterm.js and WebGL rendering
- **Block-Based Messaging** - Share code, files, and interactive widgets with peers
- **Visual Effects** - Lightning, electric borders, click sparks, and more
- **Cross-Platform** - Native apps for Windows, macOS, and Linux
- **Mobile Ready** - Responsive design with touch support

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.70+
- [pnpm](https://pnpm.io/) (recommended) or npm

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/russh.git
cd russh

# Install frontend dependencies
cd russh-client
pnpm install

# Run in development mode
pnpm tauri dev
```

### Building for Production

```bash
pnpm tauri build
```

## Project Structure

```
russh/
├── russh-ssh/          # Core SSH library (Rust)
├── russh-ssh-cli/      # CLI tool (Rust)
├── russh-client/       # Tauri + Vue.js frontend
│   ├── src/
│   │   ├── components/ # Vue components
│   │   ├── composables/# Vue composables
│   │   ├── stores/     # Pinia stores
│   │   ├── types/      # TypeScript types
│   │   ├── views/      # Page views
│   │   └── utils/      # Utilities
│   └── src-tauri/      # Tauri backend (Rust)
└── docs/               # Documentation
```

## Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [API Reference](docs/API.md)
- [P2P Terminal Guide](docs/P2P_TERMINAL.md)
- [Visual Effects Guide](docs/VISUAL_EFFECTS.md)
- [Contributing Guide](docs/CONTRIBUTING.md)

## Key Technologies

| Component | Technology |
|-----------|------------|
| Backend | Rust, Tokio, russh |
| Frontend | Vue 3, TypeScript, Tailwind CSS |
| Desktop | Tauri 2.0 |
| Terminal | xterm.js with WebGL |
| State | Pinia |
| Animations | Motion-V, Custom WebGL |

## Screenshots

*Coming soon*

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](docs/CONTRIBUTING.md) first.
# sex 02 jan 2026 18:03:54 -03
