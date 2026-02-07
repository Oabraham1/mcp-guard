//! Client-specific discovery implementations.

mod claude_code;
mod claude_desktop;
mod cline;
mod continue_dev;
mod cursor;
mod generic;
mod roo_code;
mod vscode;
mod windsurf;
mod zed;

pub use claude_code::ClaudeCodeDiscovery;
pub use claude_desktop::ClaudeDesktopDiscovery;
pub use cline::ClineDiscovery;
pub use continue_dev::ContinueDiscovery;
pub use cursor::CursorDiscovery;
pub use generic::GenericDiscovery;
pub use roo_code::RooCodeDiscovery;
pub use vscode::VsCodeDiscovery;
pub use windsurf::WindsurfDiscovery;
pub use zed::ZedDiscovery;
