use airsim::controller::Car;
use airsim::{airsim::Client, controller::keyboard::Controller};
use async_std::task;
use std::io::Result;

async fn run_car() -> Result<()> {
    let address = "127.0.0.1:41451";
    let controller = Controller::new(Client::connect(address).await?);
    controller.setup().await?;
    controller.run();
    Ok(())
}

fn main() -> Result<()> {
    task::block_on(run_car())
}
