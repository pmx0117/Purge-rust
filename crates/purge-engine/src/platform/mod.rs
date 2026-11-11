mod applications;
#[cfg(unix)]
mod applications_unix;
#[cfg(unix)]
pub use applications_unix::desktop_entry;
mod icons;
mod metadata;
mod paths;
mod system;
mod volumes;
pub use applications::*;
pub use icons::{ApplicationIcon, application_icons};
pub use metadata::*;
pub use paths::*;
pub use system::*;
pub use volumes::*;
