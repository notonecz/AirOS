pub mod manager;
pub mod snap;
pub mod window;
pub mod workspace;

pub use manager::WindowManager;
pub use snap::{compute_snap, snap_rect, SnapZone};
pub use window::{ManagedWindow, WindowState};
pub use workspace::{Workspace, WorkspaceManager};
