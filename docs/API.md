# API Reference

## Stores

### useConnectionStore

Manages SSH connection profiles and active sessions.

```typescript
import { useConnectionStore } from '@/stores/connections';

const store = useConnectionStore();

// State
store.profiles           // ConnectionProfile[]
store.activeConnections  // Map<string, ConnectionState>
store.selectedProfileId  // string | null
store.isLoading         // boolean

// Getters
store.sortedProfiles    // Profiles sorted by folder/name
store.connectedProfiles // Currently connected profiles
store.folders           // Unique folder names
store.favoriteProfiles  // Profiles tagged as favorite
store.recentProfiles    // Last 5 connected profiles

// Actions
await store.loadProfiles()
await store.createProfile(profile)
await store.updateProfile(profile)
await store.deleteProfile(profileId)
await store.connect(profileId, password?)
await store.disconnect(profileId)
store.getConnection(profileId)
store.selectProfile(profileId)
```

### useTerminalStore

Manages terminal tabs and xterm instances.

```typescript
import { useTerminalStore } from '@/stores/terminals';

const store = useTerminalStore();

// State
store.tabs        // TerminalTab[]
store.activeTabId // string | null

// Getters
store.activeTab     // Current active tab
store.tabsBySession // Map<sessionId, TerminalTab[]>

// Actions
store.createTab(sessionId, profileId, title) // Returns tabId
store.closeTab(tabId)
store.closeTabsBySession(sessionId)
store.setActiveTab(tabId)
store.updateTabTitle(tabId, title)
store.setTerminalInstance(tabId, terminal)
store.getTerminalInstance(tabId)
store.nextTab()
store.previousTab()
```

### useSettingsStore

Manages application settings with auto-save.

```typescript
import { useSettingsStore } from '@/stores/settings';

const store = useSettingsStore();

// State
store.settings  // AppSettings
store.isLoading // boolean
store.isDirty   // boolean

// Actions
await store.loadSettings()
await store.saveSettings()
store.updateSettings(partial)
store.updateSetting(category, key, value)
store.resetToDefaults()
store.resetCategory(category)
await store.exportSettings() // Returns JSON string
await store.importSettings(jsonData)
```

### useP2PTerminalStore

Manages P2P conversations and message blocks.

```typescript
import { useP2PTerminalStore } from '@/stores/p2pTerminal';

const store = useP2PTerminalStore();

// State
store.conversations        // Map<peerId, Conversation>
store.activeConversationId // string | null
store.pendingMessages      // Map<messageId, P2PMessage>

// Getters
store.activeConversation // Current conversation
store.conversationList   // Sorted by last activity
store.totalUnread        // Total unread count

// Actions
store.createConversation(peer)
store.setActiveConversation(peerId)
store.addBlock(peerId, block)
store.updateBlock(peerId, blockId, updates)
store.addWidgetResponse(peerId, blockId, response)
store.setTypingIndicator(peerId, isTyping)
store.clearConversation(peerId)
store.removeConversation(peerId)
```

---

## Composables

### useSSH

SSH connection and command execution.

```typescript
import { useSSH } from '@/composables/useSSH';

const ssh = useSSH(sessionId?);

// State
ssh.isExecuting  // boolean
ssh.isConnecting // boolean
ssh.lastResult   // CommandResult | null
ssh.error        // string | null
ssh.sessionId    // string

// Actions
const sessionId = await ssh.connect({
  host: 'example.com',
  port: 22,
  username: 'user',
  authType: 'password',
  password: 'secret'
});

const result = await ssh.execute('ls -la', timeout?);
// result: { stdout, stderr, exitCode }

await ssh.disconnect();
```

### useTerminal

xterm.js terminal management.

```typescript
import { useTerminal } from '@/composables/useTerminal';

const terminal = useTerminal();

// State
terminal.terminal // Terminal instance
terminal.isReady  // boolean

// Actions
await terminal.initTerminal(container, options?);
await terminal.attachToSession(sessionId);
terminal.write(data);
terminal.resize();
terminal.focus();
terminal.clear();
terminal.copySelection();
terminal.paste(text);
terminal.searchNext(query);
terminal.searchPrevious(query);
terminal.destroyTerminal();
```

### useP2P

P2P node and peer management.

```typescript
import { useP2P } from '@/composables/useP2P';

const p2p = useP2P();

// State
p2p.nodeInfo   // P2PNodeInfo | null
p2p.peers      // P2PPeer[]
p2p.isOnline   // boolean
p2p.qrCodeData // string | null
p2p.isLoading  // boolean

// Actions
await p2p.initialize();
const peer = await p2p.connectToPeer(peerId);
await p2p.disconnectPeer(peerId);
await p2p.refreshPeers();
const qr = await p2p.generateQRCode();
await p2p.copyNodeId();
p2p.getConnectionQuality(peer); // 'excellent'|'good'|'fair'|'poor'
```

### useP2PMessaging

Real-time P2P message streaming.

```typescript
import { useP2PMessaging } from '@/composables/useP2PMessaging';

const messaging = useP2PMessaging();

// State
messaging.isConnected // boolean
messaging.localNodeId // string

// Actions
await messaging.initialize(nodeId);

// Send blocks
await messaging.sendTextBlock(peerId, content, format?);
await messaging.sendCodeBlock(peerId, content, language, filename?);
await messaging.sendFileBlock(peerId, file);
await messaging.sendWidgetBlock(peerId, widgetType, config);
await messaging.sendSystemBlock(peerId, message, level?);

// Interactions
await messaging.sendTypingIndicator(peerId, isTyping);
await messaging.sendWidgetResponse(peerId, blockId, value);

messaging.dispose();
```

### useFileTransfer

SFTP file operations.

```typescript
import { useFileTransfer } from '@/composables/useFileTransfer';

const transfer = useFileTransfer();

// State
transfer.transfers   // TransferItem[]
transfer.isLoading   // boolean
transfer.error       // string | null

// Getters
transfer.activeTransfers    // In-progress transfers
transfer.completedTransfers // Completed transfers

// Actions
await transfer.initialize();
const files = await transfer.listFiles(sessionId, path);
await transfer.uploadFile(sessionId, remotePath, file);
await transfer.downloadFile(sessionId, remotePath, filename);
await transfer.deleteFile(sessionId, path);
await transfer.createDirectory(sessionId, path);
transfer.cancelTransfer(id);
transfer.clearCompleted();
transfer.dispose();
```

### useVisualEffects

Visual effects configuration with reduced motion support.

```typescript
import { useVisualEffects } from '@/composables/useVisualEffects';

const effects = useVisualEffects();

// State
effects.visualEffects       // VisualEffectsSettings
effects.prefersReducedMotion // boolean
effects.effectsEnabled      // boolean (respects system preference)
effects.accentColor         // string

// Individual effect checks
effects.isClickSparkEnabled
effects.isNoiseEnabled
effects.isLightningEnabled
effects.isElectricBorderEnabled
// ... etc

// Location-specific checks
const isEnabled = effects.isLightningEnabledFor('terminal');

// Actions
effects.updateVisualEffects(partial);
effects.toggleEffect('clickSpark');
effects.toggleGlobalEffects();
effects.setReducedMotion(value);
effects.getThemeAwareColor(lightColor, darkColor);
```

---

## Types

### Connection Types

```typescript
interface ConnectionProfile {
  id: string;
  name: string;
  host: string;
  port: number;
  username: string;
  authType: 'password' | 'key' | 'agent';
  keyPath?: string;
  tags: string[];
  folder?: string;
  color?: string;
  autoReconnect: boolean;
  lastConnected?: string;
  useCount: number;
}

interface ConnectionState {
  sessionId: string;
  profileId: string;
  status: 'connecting' | 'connected' | 'disconnected' | 'reconnecting' | 'error';
  connectedAt?: Date;
  error?: string;
  stats: ConnectionStats;
}

interface ConnectionRequest {
  host: string;
  port: number;
  username: string;
  authType?: string;
  password?: string;
  keyPath?: string;
}
```

### Block Types

```typescript
type BlockType = 'text' | 'code' | 'file' | 'widget' | 'system';

interface TextBlock extends BaseBlock {
  type: 'text';
  content: string;
  format?: 'plain' | 'markdown';
}

interface CodeBlock extends BaseBlock {
  type: 'code';
  content: string;
  language: string;
  filename?: string;
  showLineNumbers?: boolean;
}

interface FileBlock extends BaseBlock {
  type: 'file';
  filename: string;
  size: number;
  mimeType: string;
  previewUrl?: string;
  transferProgress?: number;
  transferStatus: 'pending' | 'transferring' | 'completed' | 'failed';
}

interface WidgetBlock extends BaseBlock {
  type: 'widget';
  widgetType: 'button' | 'input' | 'poll' | 'progress' | 'confirm';
  config: WidgetConfig;
  responses?: WidgetResponse[];
}
```

### P2P Types

```typescript
interface P2PNodeInfo {
  nodeId: string;
  relayUrl?: string;
  directAddresses: string[];
  isOnline: boolean;
}

interface P2PPeer {
  id: string;
  peerId?: string;
  name?: string;
  deviceType?: 'desktop' | 'mobile' | 'tablet';
  connectionType: 'direct' | 'relayed' | 'hole_punched';
  latencyMs?: number;
  connectedAt: string;
}
```

---

## Events

### Tauri Events

```typescript
// Listen for terminal output
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen<string>(`terminal-output-${sessionId}`, (event) => {
  terminal.write(event.payload);
});

// P2P events
await listen<{ isOnline: boolean }>('p2p-status', (event) => {
  isOnline.value = event.payload.isOnline;
});

await listen<{ peerId: string; message: P2PMessage }>('p2p-message', (event) => {
  handleMessage(event.payload);
});

// Transfer progress
await listen<TransferProgress>('transfer-progress', (event) => {
  updateProgress(event.payload);
});
```

### Custom DOM Events

```typescript
// Open command palette
document.dispatchEvent(new CustomEvent('open-command-palette'));

// Toggle sidebar
document.dispatchEvent(new CustomEvent('toggle-sidebar'));

// New terminal tab
document.dispatchEvent(new CustomEvent('new-terminal-tab'));
```
