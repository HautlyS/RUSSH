# P2P Interactive Terminal Guide

The P2P Terminal is a modern, block-based communication system that enables real-time interaction between connected peers.

## Overview

Unlike traditional chat applications, the P2P Terminal uses a block-based approach where each message is a discrete, interactive unit. This enables rich content sharing including code snippets, files, and interactive widgets.

## Getting Started

### Connecting to Peers

1. Navigate to **P2P Network** from the sidebar
2. Your Node ID and QR code are displayed for sharing
3. Connected peers appear in the peer list
4. Click **Open Terminal** to start messaging

### Starting a Conversation

1. In the P2P Terminal, select a peer from the sidebar
2. The conversation area opens with the block composer at the bottom
3. Type your message and press Enter to send

## Block Types

### Text Blocks

Basic text messages with optional markdown formatting.

**Features:**
- Plain text or markdown format
- Inline code with backticks
- Bold, italic, and links
- Emoji support
- Copy to clipboard

**Usage:**
```
Just type normally for plain text.

Use **bold** and *italic* for emphasis.
Use `code` for inline code.
```

### Code Blocks

Syntax-highlighted code snippets with language detection.

**Features:**
- 14+ supported languages
- Line numbers (toggleable)
- One-click copy
- Collapsible for long code (20+ lines)
- Filename display

**Usage:**
1. Type `/code` or click the code icon
2. Select language from dropdown
3. Optionally add filename
4. Paste or type your code
5. Press Ctrl+Enter or click Send

**Supported Languages:**
- JavaScript, TypeScript
- Python, Rust, Go
- Java, C, C++
- HTML, CSS, JSON, YAML
- Bash, SQL

### File Blocks

Share files with preview and transfer progress.

**Features:**
- Drag-and-drop support
- Image preview for images
- File size and type display
- Transfer progress indicator
- Download button for recipients
- Retry on failure

**Usage:**
1. Drag a file into the composer, or
2. Click the file icon and select a file
3. File uploads automatically

**Supported:**
- Images (with preview)
- Documents
- Archives
- Any file type

### Widget Blocks

Interactive elements for collaboration.

#### Button Widget
A clickable button that notifies the sender when pressed.

```
Label: "Approve"
Action: "approve-request"
```

#### Input Widget
Collect text responses from peers.

```
Label: "What's your name?"
Placeholder: "Enter name..."
```

#### Poll Widget
Create polls with multiple options.

```
Question: "Which framework?"
Options:
  - Vue
  - React
  - Svelte
```

#### Progress Widget
Display progress status.

```
Label: "Upload Progress"
Value: 75
Max: 100
```

#### Confirm Widget
Request confirmation with Yes/No buttons.

```
Message: "Delete this file?"
Confirm: "Yes, delete"
Cancel: "No, keep it"
```

**Creating Widgets:**
1. Type `/widget` or click the puzzle icon
2. Select widget type
3. Configure options
4. Click Send Widget

### System Blocks

Automatic status messages.

- Connection established
- Peer disconnected
- Transfer completed
- Errors and warnings

## Slash Commands

Quick access to block types:

| Command | Action |
|---------|--------|
| `/code` | Open code block composer |
| `/file` | Open file picker |
| `/widget` | Open widget builder |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift+Enter` | New line |
| `Ctrl+Enter` | Send code block |
| `/` | Open command palette |

## Real-Time Features

### Typing Indicators

When you start typing, peers see a "typing..." indicator. This automatically clears after 3 seconds of inactivity.

### Message Delivery

Messages are delivered with sub-second latency on direct connections. The system handles:
- Message ordering
- Deduplication
- Delivery acknowledgment

### Connection Status

The sidebar shows connection quality:
- 🟢 Excellent (<50ms)
- 🟡 Good (<100ms)
- 🟠 Fair (<200ms)
- 🔴 Poor (>200ms)

## Best Practices

### Code Sharing
- Always specify the language for proper highlighting
- Use filenames for context
- Keep snippets focused and relevant

### File Transfers
- Check file size before sending large files
- Use compression for large transfers
- Verify transfer completion

### Widgets
- Use polls for quick decisions
- Use confirm for destructive actions
- Keep button labels clear and actionable

## Troubleshooting

### Messages Not Sending
1. Check peer connection status
2. Verify P2P node is online
3. Try reconnecting to peer

### File Transfer Failed
1. Check file size limits
2. Verify peer is still connected
3. Use retry button

### Widgets Not Responding
1. Ensure peer has latest version
2. Check for widget type compatibility
3. Resend widget if needed

## Architecture

```
┌─────────────────────────────────────────┐
│           P2PTerminal.vue               │
│  ┌─────────────┐  ┌─────────────────┐   │
│  │  Peer List  │  │  Message Area   │   │
│  │             │  │  ┌───────────┐  │   │
│  │  [Peer 1]   │  │  │TextBlock  │  │   │
│  │  [Peer 2]   │  │  │CodeBlock  │  │   │
│  │  [Peer 3]   │  │  │FileBlock  │  │   │
│  │             │  │  │WidgetBlock│  │   │
│  └─────────────┘  │  └───────────┘  │   │
│                   └─────────────────┘   │
│  ┌─────────────────────────────────┐    │
│  │        BlockComposer            │    │
│  └─────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

### Data Flow

```
User Input → BlockComposer → useP2PMessaging
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                              ▼
            p2pTerminalStore              Tauri IPC
            (local display)               (send to peer)
                                               │
                                               ▼
                                          QUIC Stream
                                               │
                                               ▼
                                          Peer Device
                                               │
                                               ▼
                                    listen('p2p-message')
                                               │
                                               ▼
                                      p2pTerminalStore
                                      (remote display)
```

## Security

- All P2P traffic is encrypted via QUIC/TLS
- File data is transferred directly between peers
- No messages are stored on servers
- Conversation history is local only
