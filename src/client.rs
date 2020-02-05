use crate::message::{Message, Notification, Request, Response};
use async_std::io;
use async_std::io::prelude::*;
use async_std::net::{TcpStream, ToSocketAddrs};
use async_std::sync::{channel, Receiver, Sender};
use async_std::sync::{Arc, Mutex};
use async_std::task;
use std::collections::HashMap;
use std::io::{Cursor, Error};
use std::time::Duration;

pub struct Client {
    request_sender: Sender<Request>,
    notification_sender: Sender<Notification>,
    pub notification_receiver: Receiver<Notification>,
    pub request_receiver: Receiver<Request>,
    response_channels: Arc<Mutex<HashMap<usize, Sender<Response>>>>,
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
            let mut buf = vec![0u8; 1024];
            loop {
                // Send request
                if !request_receiver.is_empty() {
                    let request = request_receiver
                        .recv()
                        .await
                        .expect("non empty channel receiver didn't yield any message");
                    let message = &Message::Request(request)
                        .pack()
                        .expect("Couldn't serialize message");
                    stream
                        .write_all(message)
                        .await
                        .expect("couldn't send message");
                }

                // Send notification
                if !notification_receiver.is_empty() {
                    let notification = notification_receiver
                        .recv()
                        .await
                        .expect("non empty channel receiver didn't yield any message");
                    let message = &Message::Notification(notification)
                        .pack()
                        .expect("Couldn't serialize message");
                    stream
                        .write_all(message)
                        .await
                        .expect("couldn't send message");
                }
                // Receive data
                let _ = io::timeout(Duration::from_millis(1), async {
                    while let Ok(n) = stream.read(&mut buf).await {
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
                            Err(_) => {
                                return Ok(());
                            }
                        };
                        let (_, remaining) = current_message.split_at(frame.position() as usize);
                        current_message = remaining.to_vec();
                    }
                    Ok(())
                })
                .await;
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

    pub fn request_sync(&self, request: Request) -> Result<Option<Response>, Error> {
        task::block_on(self.request(request))
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
