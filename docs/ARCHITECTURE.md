# Architecture Overview

RUSSH is built with a layered architecture separating concerns between the Rust backend and Vue.js frontend.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Vue.js Frontend                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────────┐ │
│  │  Views  │  │Components│  │ Stores  │  │   Composables   │ │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────────┬────────┘ │
│       └────────────┴───────────┴─────────────────┘          │
│                           │                                  │
│                    Tauri IPC Bridge                          │
└───────────────────────────┬─────────────────────────────────┘
                            │
┌───────────────────────────┴─────────────────────────────────┐
│                      Tauri Backend                           │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  Commands   │  │   Events    │  │   State Manager     │  │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘  │
│         └────────────────┴───────────────────┬┘             │
└──────────────────────────────────────────────┬──────────────┘
                                               │
┌──────────────────────────────────────────────┴──────────────┐
│                     russh-ssh Library                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │
│  │Connection│  │ Session  │  │   P2P    │  │File Transfer│  │
│  └──────────┘  └──────────┘  └──────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

## Frontend Architecture

### Component Hierarchy

```
App.vue
├── AppHeader.vue
├── AppSidebar.vue
├── <router-view>
│   ├── DashboardView.vue
│   ├── ConnectionsView.vue
│   │   ├── ConnectionCard.vue
│   │   └── ConnectionList.vue
│   ├── TerminalView.vue
│   │   └── TerminalContainer.vue
│   ├── P2PView.vue
│   │   ├── P2PStatus.vue
│   │   ├── PeerList.vue
│   │   └── QRCodeShare.vue
│   ├── P2PTerminalView.vue
│   │   └── P2PTerminal.vue
│   │       ├── BlockComposer.vue
│   │       └── blocks/
│   │           ├── TextBlockView.vue
│   │           ├── CodeBlockView.vue
│   │           ├── FileBlockView.vue
│   │           └── WidgetBlockView.vue
│   └── SettingsView.vue
├── CommandPalette.vue
└── NotificationContainer.vue
```

### State Management (Pinia)

| Store | Purpose |
|-------|---------|
| `connections` | SSH connection profiles and active sessions |
| `terminals` | Terminal tabs and xterm instances |
| `settings` | Application settings and preferences |
| `notifications` | In-app notification queue |
| `p2pTerminal` | P2P conversations and message blocks |

### Composables

| Composable | Purpose |
|------------|---------|
| `useSSH` | SSH connection and command execution |
| `useTerminal` | xterm.js terminal management |
| `useP2P` | P2P node and peer management |
| `useP2PMessaging` | Real-time P2P message streaming |
| `useFileTransfer` | SFTP file operations |
| `useVisualEffects` | Visual effects configuration |
| `useTheme` | Theme switching and system detection |
| `useKeyboard` | Global keyboard shortcuts |

## Backend Architecture

### Rust Crates

```
russh/
├── russh-ssh/           # Core library
│   ├── connection/      # SSH connection handling
│   ├── session/         # Session management
│   ├── p2p/             # P2P networking (QUIC)
│   ├── streaming/       # Data streaming
│   └── vdfs/            # Virtual filesystem
├── russh-ssh-cli/       # CLI application
└── russh-client/
    └── src-tauri/       # Tauri commands
```

### Tauri Commands

```rust
// Connection commands
ssh_connect(request: ConnectionRequest) -> SessionId
ssh_disconnect(session_id: String)
ssh_execute(request: CommandRequest) -> CommandResult

// Terminal commands
terminal_start(session_id: String)
terminal_input(session_id: String, data: String)
terminal_resize(session_id: String, cols: u16, rows: u16)

// P2P commands
p2p_get_node_info() -> P2PNodeInfo
p2p_connect(peer_id: String) -> P2PPeer
p2p_send_message(peer_id: String, message: String)

// File commands
file_list(session_id: String, path: String) -> Vec<FileEntry>
file_upload(session_id: String, remote_path: String, data: Vec<u8>)
file_download(session_id: String, remote_path: String)
```

### Event System

```rust
// Terminal events
emit("terminal-output-{session_id}", data: String)

// P2P events
emit("p2p-status", { isOnline: bool })
emit("p2p-message", { peerId: String, message: P2PMessage })
emit("p2p-typing", { peerId: String, isTyping: bool })

// Transfer events
emit("transfer-progress", TransferProgress)
```

## Data Flow

### SSH Connection Flow

```
User clicks Connect
        │
        ▼
ConnectionCard.vue
        │
        ▼
connectionStore.connect(profileId)
        │
        ▼
invoke('ssh_connect', request)
        │
        ▼
Tauri Command Handler
        │
        ▼
russh-ssh::Connection::connect()
        │
        ▼
SSH Handshake & Auth
        │
        ▼
Return SessionId
        │
        ▼
Update connectionStore.activeConnections
        │
        ▼
Navigate to TerminalView
```

### P2P Message Flow

```
User types message
        │
        ▼
BlockComposer.vue
        │
        ▼
useP2PMessaging.sendTextBlock()
        │
        ▼
p2pTerminalStore.addBlock()
        │
        ▼
invoke('p2p_send_message')
        │
        ▼
QUIC Stream to Peer
        │
        ▼
Peer receives via listen('p2p-message')
        │
        ▼
handleIncomingMessage()
        │
        ▼
p2pTerminalStore.addBlock()
        │
        ▼
P2PTerminal.vue renders block
```

## Security Considerations

1. **SSH Keys** - Stored in system keychain, never in plain text
2. **Passwords** - Never persisted, only held in memory during session
3. **P2P Encryption** - All P2P traffic encrypted via QUIC/TLS
4. **IPC Security** - Tauri's secure IPC with capability-based permissions
5. **Input Sanitization** - All user input validated before processing
