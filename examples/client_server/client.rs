use super::common::Message;

use message_io::events::{self, Event};
use message_io::network_manager::{NetworkManager, Endpoint};

use std::time::{Duration};

enum Signal {
    Close,
    WriteToServer
}

pub fn run() {
    let (mut event_queue, input_event_handle) = events::new_event_system::<Message, Signal, Endpoint>();
    let mut network = NetworkManager::new(input_event_handle);

    let mut server = None;
    loop {
        if let Some(event) = event_queue.pop_event() {
            match event {
                Event::Start => {
                    let addr = "127.0.0.1:3000".parse().unwrap();
                    server = network.create_tcp_connection(addr);
                    match server {
                        Some(server) => {
                            println!("Server connected");
                            event_queue.push_timed_signal(Signal::WriteToServer, Duration::from_secs(1));
                        }
                        None => println!("Can not connect to the server")
                    }
                }
                Event::Message(message, _) => match message {
                    Message::Info(text) => println!("Server says: {}", text),
                    Message::NotifyDisconnection(duration) => println!("Server will be disconnected in {} secs", duration.as_secs()),
                    Message::Bye => println!("Server is closing"),
                },
                Event::RemovedEndpoint(_) => {
                    println!("Server is disconnected");
                    break;
                }
                Event::Signal(Signal::Close) => {
                    if let Some(server) = server {
                        network.send(server, Message::Bye);
                        network.remove_tcp_connection(server);
                    }
                    break;
                }
                Event::Signal(Signal::WriteToServer) => {
                    if let Some(server) = server {
                        println!("Sending info to the client");
                        network.send(server, Message::Info(String::from("Client info")));
                        event_queue.push_timed_signal(Signal::WriteToServer, Duration::from_secs(1));
                    }
                },
                Event::Idle => println!("I am waiting to the server..."),
                _ => unreachable!()
            }
        }
    }
}
