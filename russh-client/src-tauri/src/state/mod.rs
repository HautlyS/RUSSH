//! Application state management

mod app_state;
mod session_state;

pub use app_state::{AppSettings, AppState, P2PNodeInfo, P2PPeerInfo, ProfileData};
pub use session_state::SessionState;
