use anyhow::Result;
use nix::unistd::{close, dup2, execv, fork, pipe, ForkResult};
use serde_json::json;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::prelude::*;
use std::sync::{Arc, Mutex};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::{mpsc, oneshot};

use crate::message::*;

pub struct Client {
    request_writer: BufWriter<File>,
    unreturned: Arc<Mutex<HashMap<i32, oneshot::Sender<Response>>>>,
}

pub async fn initialize(
    path: String,
    event_sender: mpsc::UnboundedSender<Event>,
) -> Result<Client> {
    let mut client = Client::new(path, event_sender).await?;
    let _response = client
        .request(Request::new("initialize", json!({"capabilities": {}})))
        .await?;
    // PARSE RESPONSE (SUCH AS TO GET TRIGGER CHARACTERS)
    client
        .notify(Notification::new("initialized", json!({})))
        .await?;
    Ok(client)
}

impl Client {
    pub async fn new(path: String, event_sender: mpsc::UnboundedSender<Event>) -> Result<Client> {
        let response_pipe = pipe()?;
        let request_pipe = pipe()?;

        match unsafe { fork()? } {
            ForkResult::Child => {
                close(request_pipe.1)?;
                close(response_pipe.0)?;
                dup2(request_pipe.0, 0)?;
                dup2(response_pipe.1, 1)?;

                let path = CString::new(path).expect("CSTRING NEW FAILED");
                execv(&path, &[&path])?;

                panic!("UNEXPECTED SERVER TERMINATION")
            }

            ForkResult::Parent { child: _ } => {
                close(request_pipe.0)?;
                close(response_pipe.1)?;

                let request_channel = unsafe { File::from_raw_fd(request_pipe.1) };
                let response_channel = unsafe { File::from_raw_fd(response_pipe.0) };
                let unreturned = Arc::new(Mutex::new(HashMap::new()));

                tokio::spawn(listen(
                    BufReader::new(response_channel),
                    event_sender.clone(),
                    unreturned.clone(),
                ));

                Ok(Client {
                    request_writer: BufWriter::new(request_channel),
                    unreturned,
                })
            }
        }
    }

    pub async fn request(&mut self, content: Request) -> Result<Response> {
        let (response_sender, response_receiver) = oneshot::channel();

        match self.unreturned.lock() {
            Ok(mut unreturned) => {
                unreturned.insert(content.id, response_sender);
            }
            Err(err) => {
                panic!("UNRETURNED REQUEST POOL LOCK FAILED\n{}", err);
            }
        }

        match serde_json::to_vec(&content) {
            Ok(msg) => {
                self.send(msg).await?;
            }
            Err(err) => {
                panic!("NOTIFICATION SERIALIZE FAILED\n{}", err);
            }
        }

        match response_receiver.await {
            Ok(response) => Ok(response),
            Err(err) => {
                panic!("RESPONSE LISTENER HAS ALREADY CLOSED\n{}", err);
            }
        }
    }

    pub async fn notify(&mut self, content: Notification) -> Result<()> {
        match serde_json::to_vec(&content) {
            Ok(msg) => {
                self.send(msg).await?;
            }
            Err(err) => panic!("NOTIFICATION SERIALIZE FAILED\n{}", err),
        }
        Ok(())
    }

    async fn send(&mut self, content: Vec<u8>) -> Result<()> {
        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        self.request_writer.write(header.as_bytes()).await?;
        self.request_writer.write(&content).await?;
        self.request_writer.flush().await?;
        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<()> {
        self.request(Request::new("shutdown", serde_json::Value::Null))
            .await?;
        self.notify(Notification::new("exit", serde_json::Value::Null))
            .await?;
        Ok(())
    }

    pub async fn did_open(&mut self, language_id: &str, uri: &str, text: &str) -> Result<()> {
        self.notify(Notification::new(
            "textDocument/didOpen",
            json!({
                "textDocument": {
                    "uri": uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": text,
                }
            }),
        ))
        .await
    }

    pub async fn did_close(&mut self, uri: &str) -> Result<()> {
        self.notify(Notification::new(
            "textDocument/didClose",
            json!({
                "textDocument": {
                    "uri": uri,
                }
            }),
        ))
        .await
    }
}

async fn listen(
    mut response_reader: BufReader<File>,
    event_sender: mpsc::UnboundedSender<Event>,
    unreturned: Arc<Mutex<HashMap<i32, oneshot::Sender<Response>>>>,
) -> Result<()> {
    loop {
        let msg = read_response(&mut response_reader).await?;
        match serde_json::from_slice::<ServerMessage>(&msg)? {
            ServerMessage::Response(response) => match unreturned.lock() {
                Ok(mut unreturned) => {
                    if let Some(sender) = unreturned.remove(&response.id) {
                        if let Err(_) = sender.send(response) {
                            panic!("RESPONSE RECEIVER HAS ALREADY CLOSED");
                        }
                    } else {
                        panic!("NO RESPONSE SENDER IN UNRETURNED REQUEST POOL");
                    }
                }
                Err(_) => panic!("UNRETURNED REQUEST POOL LOCK FAILED"),
            },
            ServerMessage::Notification(notification) => {
                event_sender.send(Event::LanguageNotification(notification))?;
            }
        }
    }
}

async fn read_response(response_reader: &mut BufReader<File>) -> Result<Vec<u8>> {
    let content_length = read_response_header(response_reader).await?;
    let mut buf = vec![0; content_length];
    response_reader.read_exact(&mut buf).await?;
    Ok(buf)
}

async fn read_response_header(response_reader: &mut BufReader<File>) -> Result<usize> {
    // TODO: handle Content-Type
    let mut content_length = 0;
    let mut buf = [0; 16];
    response_reader.read_exact(&mut buf).await?; // "Content-Length: "
    response_reader.read(&mut buf[0..1]).await?;
    while b'0' <= buf[0] && buf[0] <= b'9' {
        content_length = content_length * 10 + (buf[0] - 48) as usize;
        response_reader.read(&mut buf[0..1]).await?;
    }
    response_reader.read_exact(&mut buf[0..3]).await?; // \n\r\n
    Ok(content_length)
}
