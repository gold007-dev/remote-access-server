#[macro_use]
extern crate rocket;
use saphyr::{LoadableYamlNode, Yaml, YamlEmitter};

use futures_util::{SinkExt, StreamExt as _};
use rocket_ws as ws;
use std::process::Command;
use tokio_pty_process::{AsyncPtyMaster, CommandExt};

#[derive(Debug)]
struct User {
    name: String,
    password: String,
}

#[get("/")]
fn index() {
    let settings =
        Yaml::load_from_str("{\"users\": [{\"name\": \"admin\",\"password\": \"password\"}]}")
            .unwrap();

    let users_yaml = &settings[0]["users"].as_vec().expect("users not a list");
    let users: Vec<User> = users_yaml
        .iter()
        .map(|user| {
            let name = user["name"].as_str().expect("name not set").to_string();
            let password = user["password"]
                .as_str()
                .expect("password not set")
                .to_string();
            User { name, password }
        })
        .collect();
    println!("{:?}", users);
}

#[get("/terminal/ws")]
fn terminal_socket(ws: ws::WebSocket) -> ws::Stream!['static] {
    ws::Stream! { ws=>
                let mut child = Command::new("/bin/bash")
            .arg("-i")
            .spawn_pty()
            .expect("Failed to spawn bash");

        let mut pty_master = child.pty_master().unwrap();
        let mut ws_stream = ws;

        // Spawn a task to read from PTY and send to WebSocket
        let mut pty_reader = tokio::io::read_split(&mut pty_master).0;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 4096];
            loop {
                match pty_reader.read(&mut buf).await {
                    Ok(n) if n > 0 => {
                        // PTY produced output → send as binary message
                        let _ = ws_sender.send(Message::Binary(buf[..n].into())).await;
                    }
                    Ok(_) => break, // EOF
                    Err(e) => {
                        eprintln!("PTY read error: {}", e);
                        break;
                    }
                }
            }
        });

        // Main loop: receive WebSocket messages → write to PTY
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Send keystrokes or input to the PTY
                    let _ = pty_master.write_all(text.as_bytes()).await;
                }
                Ok(Message::Binary(data)) => {
                    let _ = pty_master.write_all(&data).await;
                }
                Ok(Message::Close(_)) => break,
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        // After loop, kill the shell process
        let _ = child.kill();
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
