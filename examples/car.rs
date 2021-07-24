use airsim::{airsim::Client, car::Car, errors::NetworkResult};
use async_std::task;
use std::time::Duration;

async fn run_car() -> NetworkResult<()> {
    let address = "127.0.0.1:41451";
    let client = Client::connect(address).await?;
    client.reset().await?;
    let mut car = Car::new(client);
    task::sleep(Duration::from_secs(1)).await;
    car.go_forward().await?;
    task::sleep(Duration::from_secs(3)).await;
    car.go_left().await?;
    task::sleep(Duration::from_secs(3)).await;
    car.go_right().await?;
    task::sleep(Duration::from_secs(3)).await;
    car.stop().await?;
    println!("Hammertime!");

    Ok(())
}

fn main() -> NetworkResult<()> {
    task::block_on(run_car())
}
