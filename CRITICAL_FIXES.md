# RUSSH - Critical Fixes Checklist

## 🔴 CRITICAL - Must Fix Before Production

### 1. Security: Enable Host Key Verification
**File:** `russh-client/src-tauri/src/commands/ssh.rs:108`

**Current (INSECURE):**
```rust
host_key_check: HostKeyCheck::None, // TODO: Make configurable
```

**Fix:**
```rust
host_key_check: HostKeyCheck::Strict,
known_hosts_path: Some(
    dirs::home_dir()
        .ok_or(AppError::InternalError("No home dir".into()))?
        .join(".ssh/known_hosts")
),
```

**Impact:** Prevents MITM attacks  
**Effort:** 2 hours  
**Priority:** P0

---

### 2. Security: Encrypt Stored Credentials
**File:** `russh-client/src-tauri/src/state/app_state.rs:269-276`

**Current (INSECURE):**
```rust
pub async fn save_profile(&self, id: String, mut profile: ProfileData) -> Result<(), AppError> {
    // Stores passwords in plaintext JSON
    let content = serde_json::to_string_pretty(profiles)?;
    std::fs::write(path, content)?;
}
```

**Fix:**
```rust
// Add to Cargo.toml
[dependencies]
keyring = "2.0"

// Implementation
pub async fn save_profile(&self, id: String, mut profile: ProfileData) -> Result<(), AppError> {
    // Store password in system keyring
    if let Some(password) = &profile.password {
        let entry = keyring::Entry::new("russh", &id)?;
        entry.set_password(password)?;
        profile.password = None; // Don't store in JSON
    }
    
    // Save profile without sensitive data
    let content = serde_json::to_string_pretty(&profile)?;
    std::fs::write(path, content)?;
    Ok(())
}
```

**Impact:** Protects user credentials  
**Effort:** 1 day  
**Priority:** P0

---

### 3. Stability: Remove Unwrap() Calls
**Files:** Multiple (235+ instances found)

**Critical locations:**
- `russh-ssh/src/encryption/cipher.rs:138`
- `russh-client/src-tauri/src/main.rs:72`
- `russh-ssh/src/encryption/secure_channel.rs:466`

**Fix Pattern:**
```rust
// BEFORE (BAD)
let value = some_operation().unwrap();

// AFTER (GOOD)
let value = some_operation()
    .map_err(|e| AppError::InternalError(format!("Operation failed: {}", e)))?;
```

**Add to all Cargo.toml:**
```toml
[lints.clippy]
unwrap_used = "deny"
expect_used = "warn"
```

**Impact:** Prevents panics in production  
**Effort:** 2 days  
**Priority:** P0

---

### 4. Stability: Add Panic Handler
**File:** `russh-client/src-tauri/src/main.rs`

**Add before `.run()`:**
```rust
use std::panic;

fn main() {
    // Set up panic handler
    panic::set_hook(Box::new(|panic_info| {
        let payload = panic_info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic".to_string()
        };
        
        let location = panic_info.location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        
        tracing::error!("PANIC at {}: {}", location, message);
        
        // TODO: Send to error tracking service (Sentry, etc.)
    }));
    
    tauri::Builder::default()
        // ... rest of setup
}
```

**Impact:** Graceful panic handling and logging  
**Effort:** 2 hours  
**Priority:** P0

---

## 🟡 HIGH PRIORITY - Fix Soon

### 5. Session Persistence
**File:** `russh-client/src-tauri/src/state/app_state.rs`

**Add:**
```rust
#[derive(Serialize, Deserialize)]
struct SessionSnapshot {
    session_id: String,
    profile_id: String,
    host: String,
    username: String,
    connected_at: DateTime<Utc>,
}

impl AppState {
    pub async fn save_active_sessions(&self) -> Result<(), AppError> {
        let sessions = self.sessions.read().await;
        let snapshots: Vec<SessionSnapshot> = sessions
            .iter()
            .map(|(id, s)| SessionSnapshot {
                session_id: id.clone(),
                profile_id: s.info.profile_id.clone(),
                host: s.info.host.clone(),
                username: s.info.username.clone(),
                connected_at: s.info.connected_at,
            })
            .collect();
        
        let path = self.data_dir.join("active_sessions.json");
        let content = serde_json::to_string_pretty(&snapshots)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    pub async fn restore_sessions(&self) -> Result<Vec<String>, AppError> {
        let path = self.data_dir.join("active_sessions.json");
        if !path.exists() {
            return Ok(vec![]);
        }
        
        let content = std::fs::read_to_string(&path)?;
        let snapshots: Vec<SessionSnapshot> = serde_json::from_str(&content)?;
        
        let mut restored = vec![];
        for snapshot in snapshots {
            // Attempt to reconnect
            if let Ok(session_id) = self.reconnect_session(snapshot).await {
                restored.push(session_id);
            }
        }
        
        Ok(restored)
    }
}
```

**Call on app startup and shutdown**

**Impact:** Better UX, session recovery  
**Effort:** 1 day  
**Priority:** P1

---

### 6. Complete P2P Custom Relay
**File:** `russh-ssh/src/p2p/endpoint.rs:127-131`

**Current:**
```rust
P2PRelayMode::Custom(_urls) => {
    tracing::warn!("Custom relay configuration not yet implemented, using default");
    builder.relay_mode(RelayMode::Default)
}
```

**Fix:**
```rust
P2PRelayMode::Custom(urls) => {
    let relay_map = RelayMap::from_urls(urls.iter().cloned())?;
    builder.relay_mode(RelayMode::Custom(relay_map))
}
```

**Impact:** Deployment flexibility  
**Effort:** 4 hours  
**Priority:** P1

---

### 7. Add Connection Timeouts
**File:** `russh-client/src-tauri/src/commands/ssh.rs:234`

**Add to terminal_start:**
```rust
pub async fn terminal_start(
    state: State<'_, AppState>,
    window: Window,
    session_id: String,
) -> Result<(), AppError> {
    // ... existing code ...
    
    let terminal_task = tokio::spawn(async move {
        let mut keepalive_interval = tokio::time::interval(Duration::from_secs(30));
        let mut last_activity = Instant::now();
        let timeout = Duration::from_secs(300); // 5 minutes
        
        loop {
            tokio::select! {
                _ = keepalive_interval.tick() => {
                    if last_activity.elapsed() > timeout {
                        tracing::warn!("Terminal timeout for session: {}", sid);
                        break;
                    }
                    // Send keepalive
                    if shell.write(b"").await.is_err() {
                        break;
                    }
                }
                Some(data) = input_rx.recv() => {
                    last_activity = Instant::now();
                    // ... handle input ...
                }
                // ... rest of loop ...
            }
        }
    });
}
```

**Impact:** Prevents zombie connections  
**Effort:** 3 hours  
**Priority:** P1

---

## 🟢 RECOMMENDED - Quality Improvements

### 8. Add Security Scanning to CI
**File:** `.github/workflows/ci.yml`

**Add job:**
```yaml
security:
  name: Security Audit
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    
    - name: Install cargo-audit
      run: cargo install cargo-audit
    
    - name: Run security audit
      run: cargo audit
    
    - name: Install cargo-deny
      run: cargo install cargo-deny
    
    - name: Check dependencies
      run: cargo deny check
```

**Impact:** Automated vulnerability detection  
**Effort:** 1 hour  
**Priority:** P2

---

### 9. Add File Permissions Check
**File:** `russh-client/src-tauri/src/state/app_state.rs`

**Add:**
```rust
#[cfg(unix)]
fn ensure_secure_permissions(path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    
    let metadata = std::fs::metadata(path)?;
    let permissions = metadata.permissions();
    
    if permissions.mode() & 0o077 != 0 {
        // File is readable by group/others - fix it
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
        tracing::warn!("Fixed insecure permissions on {}", path.display());
    }
    
    Ok(())
}

async fn persist_profiles(&self, profiles: &HashMap<String, ProfileData>) -> Result<(), AppError> {
    let path = self.data_dir.join("profiles.json");
    let content = serde_json::to_string_pretty(profiles)?;
    std::fs::write(&path, content)?;
    
    #[cfg(unix)]
    ensure_secure_permissions(&path)?;
    
    Ok(())
}
```

**Impact:** Prevents credential exposure  
**Effort:** 2 hours  
**Priority:** P2

---

## Testing Checklist

After implementing fixes, verify:

- [ ] SSH connection with strict host key checking works
- [ ] Credentials are not visible in profiles.json
- [ ] App doesn't panic on invalid input
- [ ] Sessions restore after app restart
- [ ] Custom P2P relay servers work
- [ ] Terminal sessions timeout properly
- [ ] Security audit passes in CI
- [ ] File permissions are 0600 on Unix

---

## Quick Commands

```bash
# Run security audit
cargo audit

# Check for unwrap usage
rg "\.unwrap\(\)" --type rust russh-client/src-tauri/src/

# Run tests
cargo test --workspace

# Build release
cd russh-client && pnpm tauri build

# Check file permissions
ls -la ~/.local/share/russh-client/profiles.json
```

---

**Last Updated:** February 13, 2026  
**Status:** Ready for implementation
