use async_std::io::prelude::*;
use async_std::net::{TcpListener, TcpStream, ToSocketAddrs};
use async_std::prelude::*;
use rmp_rpc::message::{Message, Notification, Request, Response};
use std::io::Cursor;

pub async fn listen(addrs: impl ToSocketAddrs) -> std::io::Result<()> {
    let listener = TcpListener::bind(addrs).await?;
    while let Some(stream) = listener.incoming().next().await {
        println!("New connection !");
        handle_connection(stream?).await?;
    }
    Ok(())
}

async fn handle_notification(_n: &Notification) {}

fn handle_request(r: &Request) -> Response {
    match r.method.as_ref() {
        "dostuff" => Response {
            id: r.id,
            result: Ok(rmpv::Value::Array(vec![
                rmpv::Value::String("bar".into()),
                rmpv::Value::Integer(1234.into()),
            ])),
        },
        "reset" => Response {
            id: r.id,
            result: Ok(rmpv::Value::String("ok".into())),
        },
        "ping" => Response {
            id: r.id,
            result: Ok(rmpv::Value::String("pong".into())),
        },
        "getServerVersion" => Response {
            id: r.id,
            result: Ok(rmpv::Value::Integer(1.into())),
        },
        "getMinRequiredClientVersion" => Response {
            id: r.id,
            result: Ok(rmpv::Value::Integer(1.into())),
        },
        "enableApiControl" => Response {
            id: r.id,
            result: Ok(rmpv::Value::Boolean(true)),
        },
        "setCarControls" => Response {
            id: r.id,
            result: Ok(rmpv::Value::Integer(1.into())),
        },
        "getCarState" => Response {
            id: r.id,
            result: Ok(rmpv::Value::Integer(1.into())),
        },
        _ => Response {
            id: r.id,
            result: Err(rmpv::Value::String("method not implemented".into())),
        },
    }
}
async fn handle_response(_r: &Response) {}
async fn handle_message(m: &Message) -> Option<Response> {
    match m {
        Message::Notification(n) => {
            handle_notification(&n).await;
            None
        }
        Message::Request(r) => Some(handle_request(&r)),
        Message::Response(r) => {
            handle_response(&r).await;
            None
        }
    }
}
async fn handle_frame(mut frame: &mut Cursor<Vec<u8>>) -> Option<Response> {
    match Message::decode(&mut frame) {
        Ok(message) => handle_message(&message).await,
        _ => None,
    }
}
async fn handle_connection(stream: TcpStream) -> std::io::Result<()> {
    let mut current_message: Vec<u8> = vec![];
    let (reader, writer) = &mut (&stream, &stream);
    let mut buf = vec![0u8; 1024];
    while let Ok(n) = reader.read(&mut buf).await {
        current_message.extend(&buf[..n]);
        let mut frame = Cursor::new(current_message.clone());
        match handle_frame(&mut frame).await {
            Some(response) => {
                writer
                    .write_all(&Message::Response(response).pack()?)
                    .await?;
                let (_, remaining) = current_message.split_at(frame.position() as usize);
                current_message = remaining.to_vec();
            }
            None => {
                break;
            }
        }
    }
    Ok(())
}
