#[cfg(feature = "keyboard")]
pub mod keyboard;

use crate::airsim::CarControls;
use std::io::Result;

#[async_trait]
pub trait Car {
    async fn setup(&self) -> Result<()>;
    fn get_car_controls(&mut self) -> CarControls;
    async fn send_car_controls(&self, controls: CarControls) -> Result<()>;
}
