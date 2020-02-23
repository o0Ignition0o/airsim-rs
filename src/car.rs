use crate::client::Client;
use crate::message::{Notification, Request, Response};
use async_std::net::ToSocketAddrs;
use rmpv::Value;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Car {
    client: Client,
    request_id: AtomicUsize,
    controls: CarControls,
}

impl Car {
    pub async fn connect(addrs: impl ToSocketAddrs) -> std::io::Result<Self> {
        let car = Self {
            request_id: AtomicUsize::new(0),
            client: Client::connect(addrs).await?,
            controls: CarControls::default(),
        };
        car.ping().await?;
        car.enable_api_control().await?;
        Ok(car)
    }

    pub async fn ping(&self) -> std::io::Result<Option<Response>> {
        self.client
            .request(Request {
                id: self.get_request_id(),
                method: "ping".to_string(),
                params: Vec::new(),
            })
            .await
    }

    pub async fn turn_right(&mut self) -> std::io::Result<()> {
        self.controls.is_manual_gear = false;
        self.controls.manual_gear = 0;
        self.controls.throttle = 1.;
        self.controls.steering = 1.;
        self.send_car_controls().await
    }

    pub async fn turn_left(&mut self) -> std::io::Result<()> {
        self.controls.is_manual_gear = false;
        self.controls.manual_gear = 0;
        self.controls.throttle = 1.;
        self.controls.steering = -1.;
        self.send_car_controls().await
    }

    pub async fn go_forward(&mut self) -> std::io::Result<()> {
        self.controls.is_manual_gear = false;
        self.controls.manual_gear = 0;
        self.controls.throttle = 1.;
        self.controls.steering = 0.;
        self.send_car_controls().await
    }

    pub async fn go_backward(&mut self) -> std::io::Result<()> {
        self.controls.is_manual_gear = true;
        self.controls.manual_gear = -1;
        self.controls.throttle = -0.5;
        self.controls.steering = 0.;
        self.send_car_controls().await
    }

    pub async fn stop(&mut self) -> std::io::Result<()> {
        self.controls.is_manual_gear = false;
        self.controls.manual_gear = 0;
        self.controls.throttle = 0.;
        self.controls.steering = 0.;
        self.send_car_controls().await
    }

    pub async fn reset(&self) -> std::io::Result<()> {
        self.client
            .notify(Notification {
                method: "reset".to_string(),
                params: Vec::new(),
            })
            .await?;
        Ok(())
    }

    pub async fn get_client_version(&self) -> std::io::Result<String> {
        Ok("1".to_string())
    }

    pub async fn get_server_version(&self) -> std::io::Result<i64> {
        if let Some(res) = self
            .client
            .request(Request {
                id: self.get_request_id(),
                method: "getServerVersion".to_string(),
                params: Vec::new(),
            })
            .await?
        {
            dbg!(&res);
            Ok(res
                .result
                .unwrap_or_else(|_| rmpv::Value::Integer(0.into()))
                .as_i64()
                .unwrap_or(0))
        } else {
            //TODO: Error handling
            Ok(0)
        }
    }

    async fn send_car_controls(&self) -> std::io::Result<()> {
        self.client
            .request(Request {
                id: self.get_request_id(),
                method: "setCarControls".to_string(),
                params: self.controls.serialize(),
            })
            .await?;
        Ok(())
    }

    async fn enable_api_control(&self) -> std::io::Result<bool> {
        self.client
            .request(Request {
                id: self.get_request_id(),
                method: "enableApiControl".to_string(),
                params: vec![
                    rmp_rpc::Value::Boolean(true),
                    rmp_rpc::Value::String("".into()),
                ],
            })
            .await?;
        if let Some(response) = self
            .client
            .request(Request {
                id: self.get_request_id(),
                method: "enableApiControl".to_string(),
                params: vec![
                    rmp_rpc::Value::Boolean(true),
                    rmp_rpc::Value::String("".into()),
                ],
            })
            .await?
        {
            Ok(response.result.is_ok() && response.result.unwrap().as_bool() == Some(true))
        } else {
            Ok(false)
        }
    }

    fn get_request_id(&self) -> usize {
        self.request_id
            .compare_and_swap(usize::max_value(), 0, Ordering::AcqRel);
        self.request_id.fetch_add(1, Ordering::AcqRel)
    }
}
#[derive(Default, Debug)]
struct CarControls {
    throttle: f64,
    steering: f64,
    brake: f64,
    handbrake: bool,
    is_manual_gear: bool,
    manual_gear: i8,
    gear_immediate: bool,
}

impl CarControls {
    pub fn serialize(&self) -> Vec<Value> {
        vec![
            Value::Map(vec![
                (Value::String("throttle".into()), Value::F64(self.throttle)),
                (Value::String("steering".into()), Value::F64(self.steering)),
                (Value::String("brake".into()), Value::F64(self.brake)),
                (
                    Value::String("handbrake".into()),
                    Value::Boolean(self.handbrake),
                ),
                (
                    Value::String("is_manual_gear".into()),
                    Value::Boolean(self.is_manual_gear),
                ),
                (
                    Value::String("manual_gear".into()),
                    Value::Integer(self.manual_gear.into()),
                ),
                (
                    Value::String("gear_immediate".into()),
                    Value::Boolean(self.gear_immediate),
                ),
            ]),
            // TODO: FIGURE OUT WHY ?!
            Value::String("".into()),
        ]
    }
}
