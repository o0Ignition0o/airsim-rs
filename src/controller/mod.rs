#[cfg(feature = "keyboard")]
pub mod keyboard;

use std::io::Result;

#[async_trait]
pub trait Car {
    async fn setup(&self) -> Result<()>;
    fn run(self);
}
