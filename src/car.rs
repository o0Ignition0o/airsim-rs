use crate::airsim::{CarControls, Client};
use crate::errors::NetworkResult;

pub struct Car {
    client: Client,
    pub controls: CarControls,
}

impl Car {
    pub fn new(client: Client) -> Self {
        Car {
            client,
            controls: CarControls::default(),
        }
    }

    pub async fn send_controls(&mut self) -> NetworkResult<()> {
        self.client.send_car_controls(&self.controls).await
    }

    pub async fn go_right(&mut self) -> NetworkResult<()> {
        self.steer_right();
        self.throttle_down();
        self.send_controls().await
    }

    pub async fn go_left(&mut self) -> NetworkResult<()> {
        self.steer_left();
        self.throttle_up();
        self.send_controls().await
    }

    pub async fn go_forward(&mut self) -> NetworkResult<()> {
        self.steer_straight();
        self.throttle_up();
        self.send_controls().await
    }

    pub async fn go_backwards(&mut self) -> NetworkResult<()> {
        self.controls.manual_gear = -1;
        self.controls.throttle = -1.;
        self.controls.steering = 0.;
        self.controls.handbrake = false;
        self.send_controls().await
    }

    pub async fn stop(&mut self) -> NetworkResult<()> {
        self.steer_straight();
        self.throttle_down();
        self.send_controls().await
    }

    pub fn steer_straight(&mut self) {
        self.controls.steering = 0.;
    }

    pub fn steer_right(&mut self) {
        if self.controls.steering < 0. {
            self.controls.steering = 0.;
        }
        if self.controls.steering <= 0.8 {
            self.controls.steering += 0.2;
        }
    }

    pub fn steer_left(&mut self) {
        if self.controls.steering > 0. {
            self.controls.steering = 0.;
        }
        if self.controls.steering >= -0.8 {
            self.controls.steering -= 0.2;
        }
    }

    pub fn throttle_up(&mut self) {
        if self.controls.throttle <= 0.8 {
            self.controls.throttle += 0.2;
        }
    }

    pub fn throttle_down(&mut self) {
        if self.controls.throttle >= 0.2 {
            self.controls.throttle -= 0.2;
        }
    }
}
