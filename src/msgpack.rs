use async_std::io::prelude::*;
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::sync::{channel, Receiver, Sender};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::future::FutureExt;
use futures::select;
use rmp_rpc::message::{Message, Notification, Request, Response};
use std::collections::HashMap;
use std::io::Cursor;

pub struct Client {
    request_sender: Sender<Request>,
    notification_sender: Sender<Notification>,
    pub notification_receiver: Receiver<Notification>,
    pub request_receiver: Receiver<Request>,
    response_channels: Arc<Mutex<HashMap<u32, Sender<Response>>>>,
}

enum ToDo {
    Send(Message),
    Receive(usize),
}

impl Client {
    pub async fn connect(addrs: impl ToSocketAddrs) -> std::io::Result<Self> {
        let mut stream = TcpStream::connect(addrs).await?;
        let response_channels = Arc::new(Mutex::new(HashMap::new()));

        let (request_sender, request_receiver) = channel::<Request>(1);
        let (inner_request_sender, inner_request_receiver) = channel::<Request>(1);
        let (notification_sender, notification_receiver) = channel::<Notification>(1);
        let (inner_notification_sender, inner_notification_receiver) = channel::<Notification>(1);
        let res_channels = Arc::clone(&response_channels);

        task::spawn(async move {
            let mut current_message: Vec<u8> = vec![];
            let mut buf = vec![0_u8; 1024];
            loop {
                let to_process = select! {
                    maybe_request = request_receiver.recv().fuse() => {
                        if let Some(request) = maybe_request {
                            Some(ToDo::Send(Message::Request(request)))
                        } else {
                            None
                        }
                    },
                    maybe_notification = notification_receiver.recv().fuse() => {
                        if let Some(notification) = maybe_notification {
                            Some(ToDo::Send(Message::Notification(notification)))
                        } else {
                            None
                        }
                    },
                    maybe_bytes_read = stream.read(&mut buf).fuse() => {
                        if let Ok(bytes_read) = maybe_bytes_read {
                            Some(ToDo::Receive(bytes_read))
                        } else {
                            None
                        }
                    }
                };
                match to_process {
                    Some(ToDo::Send(m)) => {
                        let message = m.pack().expect("Couldn't serialize message");
                        stream
                            .write_all(&message)
                            .await
                            .expect("couldn't send message");
                    }
                    Some(ToDo::Receive(n)) => {
                        current_message.extend(&buf[..n]);
                        let mut frame = Cursor::new(current_message.clone());
                        match Message::decode(&mut frame) {
                            Ok(Message::Notification(n)) => {
                                inner_notification_sender.send(n).await;
                            }
                            Ok(Message::Request(r)) => {
                                inner_request_sender.send(r).await;
                            }
                            Ok(Message::Response(r)) => {
                                let mut senders = res_channels.lock().await;
                                let sender: Sender<Response> = senders
                                    .remove(&r.id)
                                    .expect("Got response but no request awaiting it");
                                sender.send(r).await;
                            }
                            Err(e) => {
                                // TODO: let's figure something out!
                                panic!(e);
                            }
                        };
                        #[allow(clippy::cast_possible_truncation)]
                        {
                            let (_, remaining) =
                                current_message.split_at(frame.position() as usize);
                            current_message = remaining.to_vec();
                        }
                    }
                    None => {}
                }
            }
        });
        Ok(Self {
            request_sender,
            notification_sender,
            notification_receiver: inner_notification_receiver,
            request_receiver: inner_request_receiver,
            response_channels,
        })
    }

    pub async fn request(&self, request: Request) -> std::io::Result<Option<Response>> {
        let (response_sender, response_receiver) = channel(1);

        // TODO: check if there was something
        let _ = self
            .response_channels
            .lock()
            .await
            .insert(request.id, response_sender);

        self.request_sender.send(request).await;
        Ok(response_receiver.recv().await)
    }

    pub async fn notify(&self, notification: Notification) -> std::io::Result<()> {
        self.notification_sender.send(notification).await;
        Ok(())
    }
}
