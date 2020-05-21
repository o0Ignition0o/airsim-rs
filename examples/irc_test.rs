extern crate irc;
use airsim::{airsim::Client as AirsimClient, car::Car};
use async_std::task;
use irc::client::prelude::*;
use lazy_static::lazy_static;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;
use std::{sync::Mutex, time::Duration};

const BOT_NAME: &str = "o0ignition0o_bot";
const TWITCH_CHANNEL: &str = "#o0ignition0o";

#[inline]
pub fn get_car() -> &'static Mutex<Car> {
    lazy_static! {
        static ref CAR: Mutex<Car> = { Mutex::new(start_car().expect("couldn't start the car")) };
    }
    &*CAR
}

fn get_password() -> Result<String> {
    let mut twitch_secret_file = File::open("twitch/ignition_bot_secret")?;
    let mut password = "oauth:".to_string();
    twitch_secret_file.read_to_string(&mut password)?;
    Ok(password)
}

fn hug_someone(hug_command: String, sender: &str) -> Message {
    let command_words: Vec<&str> = hug_command.split_whitespace().collect();
    // hug
    if command_words.len() == 1 {
        Message::new(
            Some(format!("{}.tmi.twitch.tv", BOT_NAME).as_ref()),
            "PRIVMSG",
            vec![TWITCH_CHANNEL],
            Some(format!("I am giving everyone a hug").as_ref()),
        )
        .expect("couldn't create message")
    } else if command_words[1] == "me" {
        Message::new(
            Some(format!("{}.tmi.twitch.tv", BOT_NAME).as_ref()),
            "PRIVMSG",
            vec![TWITCH_CHANNEL],
            Some(format!("I am giving {} a hug", sender).as_ref()),
        )
        .expect("couldn't create message")
    } else if command_words.len() == 2 {
        Message::new(
            Some(format!("{}.tmi.twitch.tv", BOT_NAME).as_ref()),
            "PRIVMSG",
            vec![TWITCH_CHANNEL],
            Some(format!("I am giving {} a hug", command_words[1]).as_ref()),
        )
        .expect("couldn't create message")
    } else {
        let mut response_sentence = format!("I am giving {} ", command_words[1]).to_string();
        for word in command_words.iter().skip(2) {
            // hug foo
            response_sentence.push_str(format!("and {} ", word).as_ref());
        }
        response_sentence.push_str("a hug.");
        Message::new(
            Some(format!("{}.tmi.twitch.tv", BOT_NAME).as_ref()),
            "PRIVMSG",
            vec![TWITCH_CHANNEL],
            Some(response_sentence.as_ref()),
        )
        .expect("couldn't create message")
    }
}

async fn async_steer_left() -> std::io::Result<()> {
    if let Ok(mut car) = get_car().lock() {
        car.go_left().await
    } else {
        println!("Couldn't get into the car");
        Ok(())
    }
}

fn steer_left() -> std::io::Result<()> {
    task::block_on(async_steer_left())
}

async fn async_steer_right() -> std::io::Result<()> {
    if let Ok(mut car) = get_car().lock() {
        car.go_right().await
    } else {
        println!("Couldn't get into the car");
        Ok(())
    }
}

fn steer_right() -> std::io::Result<()> {
    task::block_on(async_steer_right())
}

fn send_steer_command(command: String) {
    let command_words: Vec<&str> = command.split_whitespace().collect();
    if command_words.len() != 2 {
        println!("wrong command {}", command);
        return;
    }

    match command_words[1] {
        "left" => {
            println!("steering left");
            steer_left().expect("couldn't steer left");
        }
        "right" => {
            println!("steering right");
            steer_right().expect("couldn't steer right");
        }
        _ => {
            println!("invalid command {}", command_words[1]);
        }
    }
}

async fn async_throttle_up() -> std::io::Result<()> {
    if let Ok(mut car) = get_car().lock() {
        car.go_forward().await
    } else {
        println!("Couldn't get into the car");
        Ok(())
    }
}

fn throttle_up() -> std::io::Result<()> {
    task::block_on(async_throttle_up())
}

async fn async_throttle_down() -> std::io::Result<()> {
    if let Ok(mut car) = get_car().lock() {
        car.stop().await
    } else {
        println!("Couldn't get into the car");
        Ok(())
    }
}

fn throttle_down() -> std::io::Result<()> {
    task::block_on(async_throttle_down())
}

fn send_throttle_command(command: String) {
    let command_words: Vec<&str> = command.split_whitespace().collect();
    if command_words.len() != 2 {
        println!("wrong command {}", command);
        return;
    }

    match command_words[1] {
        "up" => {
            println!("throttling up");
            throttle_up().expect("couldn't throttle up");
        }
        "down" => {
            println!("throttling down");
        }
        _ => {
            println!("invalid command {}", command_words[1]);
        }
    }
}

async fn reset_car() -> std::io::Result<()> {
    if let Ok(mut car) = get_car().lock() {
        car.reset().await
    } else {
        println!("Couldn't get into the car");
        Ok(())
    }
}

fn send_reset_command() {
    task::block_on(reset_car()).expect("couldn't reset the car")
}

fn reply_message(contents: &str) -> Message {
    Message::new(
        Some(format!("{}.tmi.twitch.tv", BOT_NAME).as_ref()),
        "PRIVMSG",
        vec![TWITCH_CHANNEL],
        Some(contents),
    )
    .expect("couldn't create message")
}

fn help_message() -> Message {
    reply_message(format!("Hi I'm {}, type 'help car' to learn how to control the car, and type 'help hug' to know how to get hugs", BOT_NAME).as_ref())
}

fn help_car_message() -> Message {
    reply_message("type 'w' or 'a' or 's' or 'd' (zqsd if using a french keyboard) to steer the car. If the car is stuck, type 'reset'.")
}

fn help_hug_message() -> Message {
    reply_message("type 'hug' to ask me to hug everyone. type 'hug me' to get a free hug. type 'hug someone someoneelse' to ask me to hug someone and someoneelse")
}

async fn run_car() -> std::io::Result<Car> {
    let address = "127.0.0.1:41451";
    let client = AirsimClient::connect(address).await?;
    client.reset().await?;
    println!("server version is: {}", client.get_server_version().await?);
    task::sleep(Duration::from_secs(5)).await;
    let car = Car::new(client);
    Ok(car)
}

fn start_car() -> std::io::Result<Car> {
    task::block_on(run_car())
}

fn give_a_beer(receiver: &str) -> Message {
    reply_message(format!("{} here's a beer for you!", receiver).as_ref())
}

fn main() -> Result<()> {
    // start the car
    let _ = get_car();
    let password = get_password()?;

    let config = Config {
        nickname: Some("o0ignition0o_bot".to_string()),
        password: Some(password),
        server: Some("irc.chat.twitch.tv".to_string()),
        channels: Some(vec![TWITCH_CHANNEL.to_owned()]),
        ..Config::default()
    };

    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();

    reactor.register_client_with_handler(client, |client, message| {
        println!("{:?}", message);
        println!("sender is {:?}", message.source_nickname());
        println!("recipient is {:?}", message.response_target());

        let m2 = message.clone();

        let sender = m2.source_nickname().unwrap_or("anonymous");
        match message.command {
            Command::PRIVMSG(target, payload) => {
                if target == TWITCH_CHANNEL && sender != BOT_NAME {
                    if payload == "beer" {
                        client
                            .send(give_a_beer(sender))
                            .expect("couldn't send reply");
                    }
                    if payload == "help car" {
                        client
                            .send(help_car_message())
                            .expect("couldn't send reply");
                    }
                    if payload == "help hug" {
                        client
                            .send(help_hug_message())
                            .expect("couldn't send reply");
                    } else if payload == "help" {
                        client.send(help_message()).expect("couldn't send reply");
                    } else if payload.starts_with("hug") {
                        let reply = hug_someone(payload, sender);
                        client.send(reply).expect("couldn't send reply");
                    } else if payload == "w" || payload == "z" {
                        throttle_up().expect("couldn't throttle up");
                    } else if payload == "s" {
                        throttle_down().expect("couldn't throttle down");
                    } else if payload == "a" || payload == "q" {
                        steer_left().expect("couldn't steer left");
                    } else if payload == "d" {
                        steer_right().expect("couldn't steer right");
                    } else if payload == "reset" {
                        send_reset_command();
                    }
                }
            }
            _ => {}
        }
        Ok(())
    });

    reactor.run().unwrap();
    Ok(())
}
