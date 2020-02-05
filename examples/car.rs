use airsim::car::Car;
use async_std::task;
use std::time::Duration;

async fn run_car() -> std::io::Result<()> {
    let address = "127.0.0.1:41451";
    let mut car = Car::connect(address).await?;
    car.reset().await?;
    println!("server version is: {}", car.get_server_version().await?);
    task::sleep(Duration::from_secs(5)).await;
    car.turn_right().await?;
    task::sleep(Duration::from_secs(5)).await;
    car.turn_left().await?;
    task::sleep(Duration::from_secs(5)).await;
    car.stop().await?;
    println!("Hammertime!");

    Ok(())
}

fn main() -> std::io::Result<()> {
    task::block_on(run_car())
}
