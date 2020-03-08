use airsim::airsim::{CarControls, Client};
use async_std::task;
use core::time::Duration;
use gilrs::{Axis, Button, Event, Gilrs};
use std::io::Result;

const REFRESH_INTERVAL: Duration = Duration::from_millis(16);
const STEERING_INTERVAL: f64 = 0.2;
const THROTTLE_INTERVAL: f64 = 0.2;
#[derive(Default, Debug)]
struct Gamepad {
    south: bool,
    west: bool,
    dpad_left: bool,
    dpad_right: bool,
    analog_steer: f32,
    analog_throttle: f32,
    analog_break: f32,
    start: bool,
}

struct GamepadController {
    gamepad: Gamepad,
    client: Client,
    car_controls: CarControls,
}

impl GamepadController {
    pub fn get_car_controls(&mut self) -> CarControls {
        // Throttle management
        if self.gamepad.south {
            self.car_controls.brake = 0.;
            if self.car_controls.throttle <= 1. - THROTTLE_INTERVAL {
                self.car_controls.throttle += THROTTLE_INTERVAL;
            }
        } else {
            self.car_controls.throttle = self.gamepad.analog_throttle.into()
        }

        // Brakes management
        if self.gamepad.west {
            self.car_controls.throttle = 0.;
            if self.car_controls.brake <= 1. - THROTTLE_INTERVAL {
                self.car_controls.brake += THROTTLE_INTERVAL;
            }
        } else {
            self.car_controls.brake = self.gamepad.analog_break.into()
        }

        // Steering
        if self.gamepad.analog_steer != 0. {
            self.car_controls.steering = self.gamepad.analog_steer.into();
        } else {
            if self.gamepad.dpad_left {
                if self.car_controls.steering > 0. {
                    self.car_controls.steering = 0.;
                }
                if self.car_controls.steering >= -(1. - THROTTLE_INTERVAL) {
                    self.car_controls.steering -= STEERING_INTERVAL;
                }
            } else if self.gamepad.dpad_right {
                if self.car_controls.steering < 0. {
                    self.car_controls.steering = 0.;
                }
                if self.car_controls.steering <= 1. - THROTTLE_INTERVAL {
                    self.car_controls.steering += STEERING_INTERVAL;
                }
            } else {
                self.car_controls.steering = 0.;
            }
        }
        self.car_controls.clone()
    }
}

async fn run_car() -> std::io::Result<()> {
    let address = "127.0.0.1:41451";

    let mut gilrs = Gilrs::new().unwrap();

    let mut gamepad_controller = GamepadController {
        gamepad: Gamepad::default(),
        client: Client::connect(address).await?,
        car_controls: CarControls::default(),
    };

    gamepad_controller.client.reset().await?;

    let mut active_gamepad = None;
    loop {
        std::thread::sleep(REFRESH_INTERVAL);

        // get_controller_state()
        while let Some(Event { id, .. }) = gilrs.next_event() {
            active_gamepad = Some(id);
        }
        if let Some(gamepad) = active_gamepad.map(|id| gilrs.gamepad(id)) {
            // Throttle management
            if gamepad.is_pressed(Button::South) {
                gamepad_controller.gamepad.south = true;
            } else {
                gamepad_controller.gamepad.south = false;
            }
            if gamepad.is_pressed(Button::West) {
                gamepad_controller.gamepad.west = true;
            } else {
                gamepad_controller.gamepad.west = false;
            }

            if let Some(right_trigger) = gamepad.button_code(Button::RightTrigger2) {
                gamepad_controller.gamepad.analog_throttle = gamepad.state().value(right_trigger);
            }

            
            if let Some(left_trigger) = gamepad.button_code(Button::LeftTrigger2) {
                gamepad_controller.gamepad.analog_break = gamepad.state().value(left_trigger);
            }

            // Steering management
            if let Some(data) = gamepad.axis_data(Axis::LeftStickX) {
                gamepad_controller.gamepad.analog_steer = data.value();
            }
            if gamepad.is_pressed(Button::DPadLeft) {
                gamepad_controller.gamepad.dpad_left = true;
            } else {
                gamepad_controller.gamepad.dpad_left = false;
            }
            if gamepad.is_pressed(Button::DPadRight) {
                gamepad_controller.gamepad.dpad_right = true;
            } else {
                gamepad_controller.gamepad.dpad_right = false;
            }
            if gamepad.is_pressed(Button::Start) {
                gamepad_controller.gamepad.start = true;
            } else {
                gamepad_controller.gamepad.start = false;
            }
        }

        if gamepad_controller.gamepad.start == true {
            return Ok(());
        }

        // get_car_controls()
        let car_controls = gamepad_controller.get_car_controls();
        // send_car_controls()
        gamepad_controller
            .client
            .send_car_controls(&car_controls)
            .await?;
    }
}

fn main() -> Result<()> {
    task::block_on(run_car())
}
