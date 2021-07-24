use crate::{
    errors::{NetworkError, NetworkResult},
    msgpack::Client as MsgPackClient,
};
use async_std::{channel::RecvError, net::ToSocketAddrs};
use rmp_rpc::message::{Notification, Request, Response};
use rmpv::Value;
use std::sync::atomic::{AtomicU32, Ordering};

pub struct Client {
    client: MsgPackClient,
    last_request_id: AtomicU32,
}

impl Client {
    pub async fn connect(addrs: impl ToSocketAddrs) -> NetworkResult<Self> {
        let car = Self {
            last_request_id: AtomicU32::new(0),
            client: MsgPackClient::connect(addrs).await?,
        };
        car.ping().await?;
        car.enable_api_control().await?;
        Ok(car)
    }

    pub async fn ping(&self) -> NetworkResult<Response> {
        self.client
            .request(Request {
                id: self.new_request_id(),
                method: "ping".to_string(),
                params: Vec::new(),
            })
            .await
            .map_err(Into::into)
    }

    pub async fn reset(&self) -> NetworkResult<()> {
        self.client
            .notify(Notification {
                method: "reset".to_string(),
                params: Vec::new(),
            })
            .await
            .map_err::<NetworkError, _>(Into::into)?;
        Ok(())
    }

    pub async fn get_client_version(&self) -> NetworkResult<String> {
        Ok("1".to_string())
    }

    pub async fn get_server_version(&self) -> NetworkResult<i64> {
        self.client
            .request(Request {
                id: self.new_request_id(),
                method: "getServerVersion".to_string(),
                params: Vec::new(),
            })
            .await
            .map(|res| {
                res.result
                    .unwrap_or_else(|_| rmpv::Value::Integer(0.into()))
                    .as_i64()
                    .unwrap_or(0)
            })
            .map_err(Into::into)
    }

    pub async fn send_car_controls(&self, controls: &CarControls) -> NetworkResult<()> {
        self.client
            .request(Request {
                id: self.new_request_id(),
                method: "setCarControls".to_string(),
                params: controls.serialize(),
            })
            .await?;
        Ok(())
    }

    async fn enable_api_control(&self) -> Result<bool, RecvError> {
        self.client
            .request(Request {
                id: self.new_request_id(),
                method: "enableApiControl".to_string(),
                params: vec![
                    rmp_rpc::Value::Boolean(true),
                    rmp_rpc::Value::String("".into()),
                ],
            })
            .await?;
        self.client
            .request(Request {
                id: self.new_request_id(),
                method: "enableApiControl".to_string(),
                params: vec![
                    rmp_rpc::Value::Boolean(true),
                    rmp_rpc::Value::String("".into()),
                ],
            })
            .await
            .map_err(Into::into)
            .map(|response| {
                response.result.is_ok() && response.result.unwrap().as_bool() == Some(true)
            })
    }

    fn new_request_id(&self) -> u32 {
        self.last_request_id
            .compare_and_swap(u32::max_value(), 0, Ordering::AcqRel);
        self.last_request_id.fetch_add(1, Ordering::AcqRel)
    }
}
#[derive(Default, Debug, Clone)]
pub struct CarControls {
    pub throttle: f64,
    pub steering: f64,
    pub brake: f64,
    pub handbrake: bool,
    pub is_manual_gear: bool,
    pub manual_gear: i8,
    pub gear_immediate: bool,
}

impl CarControls {
    #[must_use]
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
