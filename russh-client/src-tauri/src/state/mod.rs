//! Application state management

mod app_state;
mod session_state;

pub use app_state::{AppState, ProfileData, P2PNodeInfo, P2PPeerInfo, AppSettings};
pub use session_state::{SessionState, SessionInfo};
