use airsim::airsim::{CarControls, Client};
use async_std::task;
use core::time::Duration;
use std::io::Result;
use rust_webvr::{VRGamepadState, VRServiceManager};
const STEER_TRESHOLD: f32 = 0.1;


const REFRESH_INTERVAL: Duration = Duration::from_millis(16);
const STEERING_INTERVAL: f64 = 0.02;
const THROTTLE_INTERVAL: f64 = 0.2;

struct VrController {
    vr_car_state: Option<CarState>,
    client: Client,
    car_controls: CarControls,
}

impl VrController {
    pub fn get_car_controls(&mut self) -> CarControls {
        if let Some(car_state) = &self.vr_car_state {
        // Throttle management
        if car_state.accelerating {
            self.car_controls.brake = 0.;
            if self.car_controls.throttle <= 1. - THROTTLE_INTERVAL {
                self.car_controls.throttle += THROTTLE_INTERVAL;
            }
        }

        // Brakes management
        if car_state.braking {
            self.car_controls.throttle = 0.;
            if self.car_controls.brake <= 1. - THROTTLE_INTERVAL {
                self.car_controls.brake += THROTTLE_INTERVAL;
            }
        }

        // Steering
            if car_state.steering == Steering::Left {
                if self.car_controls.steering > 0. {
                    self.car_controls.steering = 0.;
                }
                if self.car_controls.steering >= -(1. - THROTTLE_INTERVAL) {
                    self.car_controls.steering -= STEERING_INTERVAL;
                }
            } else if car_state.steering == Steering::Right  {
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

    let mut vr = VRServiceManager::new();
    vr.register_defaults();
    vr.initialize_services();

    let mut vr_controller = VrController {
        client: Client::connect(address).await?,
        car_controls: CarControls::default(),
        vr_car_state: None,
    };

    vr_controller.client.reset().await?;

    loop {
        let gamepads = vr.get_gamepads();

        let gamepad_positions = gamepads
            .iter()
            .map(|gamepad| gamepad.borrow().state())
            .collect::<Vec<_>>();

        let car_state = CarState::from_raw(&gamepad_positions[0], &gamepad_positions[1]);

        if let Some(cs) = &car_state {
            println!("{}", cs);
        }

        vr_controller.vr_car_state = car_state;

        let car_controls = vr_controller.get_car_controls();
        println!("{:?}", car_controls);
        vr_controller
            .client
            .send_car_controls(&car_controls)
            .await?;

        std::thread::sleep(REFRESH_INTERVAL);
    }
}

fn main() -> Result<()> {
    task::block_on(run_car())
}



struct CarState {
    pub accelerating: bool,
    pub braking: bool,
    pub steering: Steering,
}

impl std::fmt::Display for CarState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Braking: {} | Accelerating: {} | Steering {}",
            self.braking, self.accelerating, self.steering
        )
    }
}

impl CarState {
    pub fn from_raw(
        first_gamepad_state: &VRGamepadState,
        second_gamepad_state: &VRGamepadState,
    ) -> Option<Self> {
        let first_controller_position =
            ControlerPosition::from_raw(first_gamepad_state.pose.position);
        let second_controller_position =
            ControlerPosition::from_raw(second_gamepad_state.pose.position);

        match (&first_controller_position, &second_controller_position) {
            (Some(fcp), Some(scp)) => {
                if fcp.side > scp.side {
                    let braking = first_gamepad_state.buttons[0].pressed;
                    let accelerating = second_gamepad_state.buttons[0].pressed;
            
                    Self {
                        accelerating,
                        braking,
                        steering: get_steering(&first_controller_position, &second_controller_position),
                    }.into()
                } else {
                    let accelerating = first_gamepad_state.buttons[0].pressed;
                    let braking = second_gamepad_state.buttons[0].pressed;
            
                    Self {
                        accelerating,
                        braking,
                        steering: get_steering(&second_controller_position, &first_controller_position),
                    }.into()
                }
                
            },
            _ => None
        }

    }
}
#[derive(Debug)]
struct ControlerPosition {
    pub depth: f32,
    pub height: f32,
    pub side: f32,
}

impl ControlerPosition {
    pub fn from_raw(raw_position: Option<[f32; 3]>) -> Option<Self> {
        raw_position.map(|position| Self {
            depth: position[0],
            height: position[1],
            side: position[2],
        })
    }
}

#[derive(PartialEq)]
enum Steering {
    Left,
    Right,
    Straight,
}

impl std::fmt::Display for Steering {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left => write!(f, "Left"),
            Self::Right => write!(f, "Right"),
            Self::Straight => write!(f, "Straight"),
        }
    }
}

// TODO: compute with more than just the height, this is not very reliable
fn get_steering(
    left_controller_position: &Option<ControlerPosition>,
    right_controler_position: &Option<ControlerPosition>,
) -> Steering {
    match (left_controller_position, right_controler_position) {
        (Some(left_pos), Some(right_pos)) => {
            let height_difference = left_pos.height - right_pos.height;
            if height_difference.abs() < STEER_TRESHOLD {
                Steering::Straight
            } else if height_difference < 0. {
                Steering::Left
            } else {
                Steering::Right
            }
        }
        _ => Steering::Straight,
    }
}
