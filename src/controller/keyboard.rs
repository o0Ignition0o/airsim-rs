use crate::controller;
use crate::controller::Car;
use crate::airsim::{CarControls, Client};
use async_std::task;
use async_trait::async_trait;
use glutin::event::VirtualKeyCode;
use glutin::event_loop::{ControlFlow, EventLoop};
use std::io::Result;
use std::time::{Duration, Instant};

const REFRESH_INTERVAL: Duration = Duration::from_millis(16);
const STEERING_INTERVAL: f64 = 0.2;
const THROTTLE_INTERVAL: f64 = 0.2;
pub struct Controller {
    client: Client,
    keyboard: Keyboard,
    car_controls: CarControls,
}

#[async_trait]
impl controller::Car for Controller {
    async fn setup(&self) -> Result<()> {
        self.client.reset().await
    }
    fn get_car_controls(&mut self) -> CarControls {
        if self.keyboard.up {
            if self.car_controls.throttle <= 0.8 {
                self.car_controls.throttle += THROTTLE_INTERVAL;
            }
        } else if self.keyboard.down && self.car_controls.throttle >= 0.2 {
            self.car_controls.throttle -= THROTTLE_INTERVAL;
        }
        if self.keyboard.left {
            if self.car_controls.steering > 0. {
                self.car_controls.steering = 0.;
            }
            if self.car_controls.steering >= -0.8 {
                self.car_controls.steering -= STEERING_INTERVAL;
            }
        } else if self.keyboard.right {
            if self.car_controls.steering < 0. {
                self.car_controls.steering = 0.;
            }
            if self.car_controls.steering <= 0.8 {
                self.car_controls.steering += STEERING_INTERVAL;
            }
        } else {
            self.car_controls.steering = 0.;
        }
        self.car_controls.clone()
    }

    async fn send_car_controls(&self, controls: CarControls) -> Result<()> {
        self.client.send_car_controls(&controls).await
    }
}

impl Controller {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            keyboard: Keyboard::default(),
            car_controls: CarControls::default(),
        }
    }
    fn handle_key_press(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::Up => {
                self.keyboard.up = true;
            }
            VirtualKeyCode::Down => {
                self.keyboard.down = true;
            }
            VirtualKeyCode::Left => {
                self.keyboard.left = true;
            }
            VirtualKeyCode::Right => {
                self.keyboard.right = true;
            }
            VirtualKeyCode::Escape => {
                self.keyboard.escape = true;
            }
            not_handled_key => {
                println!("key not handled {:?}", not_handled_key);
            }
        }
    }
    fn handle_key_release(&mut self, key: VirtualKeyCode) {
        match key {
            VirtualKeyCode::Up => {
                self.keyboard.up = false;
            }
            VirtualKeyCode::Down => {
                self.keyboard.down = false;
            }
            VirtualKeyCode::Left => {
                self.keyboard.left = false;
            }
            VirtualKeyCode::Right => {
                self.keyboard.right = false;
            }
            not_handled_key => {
                println!("key not handled {:?}", not_handled_key);
            }
        }
    }

    pub fn run(mut self) {
        EventLoop::new().run(move |event, _, control_flow| {
            if let glutin::event::Event::DeviceEvent { event, .. } = event {
                // handle keyboard and mouse
                if let glutin::event::DeviceEvent::Key(input) = event {
                    if input.state == glutin::event::ElementState::Pressed {
                        if let Some(key) = input.virtual_keycode {
                            self.handle_key_press(key);
                        }
                    }
                    if input.state == glutin::event::ElementState::Released {
                        if let Some(key) = input.virtual_keycode {
                            self.handle_key_release(key);
                        }
                    }
                }
            }
            let car_controls = self.get_car_controls();
            task::block_on(self.send_car_controls(car_controls))
                .expect("couldn't send car controls");
            *control_flow = self.get_next_control_flow();
        });
    }

    fn get_next_control_flow(&self) -> ControlFlow {
        if self.keyboard.escape {
            ControlFlow::Exit
        } else {
            ControlFlow::WaitUntil(Instant::now() + REFRESH_INTERVAL)
        }
    }
}
#[derive(Default)]
struct Keyboard {
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    escape: bool,
}
