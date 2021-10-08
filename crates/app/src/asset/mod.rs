pub mod assets;
pub mod daemon;
pub mod handle;

pub use assets::*;
pub use daemon::*;
pub use handle::*;
use util::downcast_rs::{impl_downcast, Downcast};

pub trait Asset: AssetDyn {}
pub trait AssetDyn: Downcast + Send + Sync + 'static {}
impl_downcast!(AssetDyn);

impl<T> Asset for T where T: AssetDyn {}
impl<T> AssetDyn for T where T: Send + Sync + 'static {}
