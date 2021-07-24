use airsim::errors::NetworkResult;
use airsim::{airsim::Client, car::Car};
use async_std::task;
use std::time::Duration;

async fn drive_to_main_road(car: &mut Car) -> NetworkResult<()> {
    // start
    car.go_forward().await?;
    task::sleep(Duration::from_secs(1)).await;
    // first turn
    car.go_left().await?;
    task::sleep(Duration::from_millis(300)).await;

    car.go_forward().await?;
    task::sleep(Duration::from_millis(900)).await;
    // second turn
    car.go_right().await?;
    task::sleep(Duration::from_millis(250)).await;
    car.go_forward().await?;
    task::sleep(Duration::from_millis(1750)).await;
    car.go_right().await?;
    task::sleep(Duration::from_millis(750)).await;
    car.go_forward().await?;
    println!("I should be on the main road now");
    Ok(())
}

async fn drive_first_long_left_turn(car: &mut Car) -> NetworkResult<()> {
    car.go_left().await?;
    task::sleep(Duration::from_millis(400)).await;
    car.go_forward().await?;
    task::sleep(Duration::from_millis(1750)).await;
    println!("Start turning left now");
    car.go_left().await?;
    task::sleep(Duration::from_millis(500)).await;
    car.go_forward().await?;
    task::sleep(Duration::from_millis(750)).await;
    car.go_left().await?;
    task::sleep(Duration::from_millis(500)).await;
    car.go_forward().await?;
    Ok(())
}

async fn drive_sharp_right(car: &mut Car) -> NetworkResult<()> {
    car.go_forward().await?;
    task::sleep(Duration::from_secs(1)).await;
    println!("Start turning sharp right now");
    car.go_right().await?;
    task::sleep(Duration::from_millis(1500)).await;
    car.go_forward().await?;
    Ok(())
}

async fn drive_sharp_left(car: &mut Car) -> NetworkResult<()> {
    car.go_forward().await?;
    task::sleep(Duration::from_millis(500)).await;
    println!("Start turning sharp left now");
    car.go_left().await?;
    task::sleep(Duration::from_secs(2)).await;
    car.go_forward().await?;
    Ok(())
}

async fn run_car() -> NetworkResult<()> {
    let address = "127.0.0.1:41451";
    let client = Client::connect(address).await?;
    client.reset().await?;
    println!("server version is: {}", client.get_server_version().await?);
    task::sleep(Duration::from_secs(5)).await;
    let mut car = Car::new(client);
    drive_to_main_road(&mut car).await?;
    drive_first_long_left_turn(&mut car).await?;
    drive_sharp_right(&mut car).await?;
    drive_sharp_left(&mut car).await?;

    car.stop().await?;
    println!("Hammertime!");
    Ok(())
}

fn main() -> NetworkResult<()> {
    task::block_on(run_car())
}
