mod auth;
#[macro_use]
extern crate rocket;
use crate::auth::{Auth, AuthError, User, generate_token};
use crate::rocket::tokio::io::AsyncReadExt;
use futures_util::SinkExt;
use rocket::http::{Method, Status};
use rocket::serde::json::Json;
use rocket_ws as ws;

use pty_process::open;
use rocket::futures::StreamExt;
use tokio::io::AsyncWriteExt;
use tokio::select;

#[post("/auth/login", data = "<user>")]
fn login(user: User) -> String {
    let token = generate_token(user.id);
    return token;
}

#[get("/auth/me")]
fn me(_auth: Auth) -> Json<Auth> {
    return Json(_auth);
}

#[get("/terminal/ws")]
async fn terminal_socket(
    _auth: Auth,
    ws: ws::WebSocket,
) -> Result<ws::Channel<'static>, AuthError> {
    Ok(ws.channel(move |mut stream| {
        Box::pin(async move {
            // ---- PTY setup ----
            let (mut pty, pts) = open().expect("failed to open pty");
            pty.resize(pty_process::Size::new(24, 80))
                .expect("failed to resize");
            let cmd = pty_process::Command::new("/usr/bin/bash");
            let mut child = cmd.spawn(pts).expect("failed to spawn");


            let (mut pty_reader, mut pty_writer) = pty.split();
            let mut pty_buf = vec![0u8; 4096];

            // ---- Main event loop ----
            loop {
                select! {
                    // PTY output -> send back to browser
                    n = pty_reader.read(&mut pty_buf) => match n {
                        Ok(0) => break,
                        Ok(n) => {
                            let data = pty_buf[..n].to_vec();
                            if stream.send(ws::Message::Binary(data)).await.is_err() {
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!("PTY read error: {}", e);
                            break;
                        }
                    },
                    // Browser input -> write to PTY
                    msg = stream.next() => match msg {
                                        Some(Ok(ws::Message::Text(text))) => {
                                            if pty_writer.write_all(text.as_bytes()).await.is_err() {
                                                break;
                                            }
                                            let _ = pty_writer.flush().await;
                                        }
                                        Some(Ok(ws::Message::Binary(data))) => {
                                            if pty_writer.write_all(&data).await.is_err() {
                                                break;
                                            }
                                            let _ = pty_writer.flush().await;
                                        }
                                        Some(Ok(ws::Message::Close(_))) => break,
                                        Some(Ok(_)) => {
                                            // Ignore Ping, Pong, Frame – the library handles them automatically
                                        }
                                        Some(Err(e)) => {
                                            eprintln!("WebSocket error: {}", e);
                                            break;
                                        }
                                        None => break,
                                    },
                }
            }

            // Clean up
            let _ = child.kill();
            Ok(())
        })
    }))
}

// cors
// Source - https://stackoverflow.com/a/64904947
// Posted by Ibraheem Ahmed, modified by community. See post 'Timeline' for change history
// Retrieved 2026-05-21, License - CC BY-SA 4.0

use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Header;
use rocket::{Request, Response};

pub struct CORS;

#[rocket::async_trait]
impl Fairing for CORS {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new("Access-Control-Allow-Methods", "*"));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
        if request.method() == Method::Options {
            response.set_status(Status::NoContent);
            let _ = response.body_mut().take();
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/api", routes![terminal_socket, login, me])
        .attach(CORS)
}
