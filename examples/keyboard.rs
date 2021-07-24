use airsim::airsim::Client;
use airsim::controller::Car;
use airsim::errors::NetworkResult;
use async_std::task;

#[cfg(feature = "keyboard")]
use airsim::controller::keyboard::Controller;

#[cfg(feature = "keyboard")]
async fn run_car() -> NetworkResult<()> {
    let address = "127.0.0.1:41451";
    let controller = Controller::new(Client::connect(address).await?);
    controller.setup().await?;
    controller.run();
    Ok(())
}

#[cfg(not(feature = "keyboard"))]
async fn run_car() -> NetworkResult<()> {
    panic!("you must run this example with the keyboard features")
}

fn main() -> NetworkResult<()> {
    task::block_on(run_car())
}
