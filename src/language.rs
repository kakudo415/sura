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

pub async fn initialize(path: String, event_sender: mpsc::UnboundedSender<Event>) -> Client {
    let mut client = Client::new(path, event_sender).await;
    let response = client
        .request(Request::new("initialize", json!({"capabilities": {}})))
        .await;
    client
        .notify(Notification::new("initialized", json!({})))
        .await;
    client
}

impl Client {
    pub async fn new(path: String, event_sender: mpsc::UnboundedSender<Event>) -> Client {
        let response_pipe = pipe().unwrap();
        let request_pipe = pipe().unwrap();
        let path = CString::new(path).expect("CString::new failed");

        match unsafe { fork().unwrap() } {
            ForkResult::Child => {
                close(request_pipe.1).unwrap();
                close(response_pipe.0).unwrap();
                dup2(request_pipe.0, 0).unwrap();
                dup2(response_pipe.1, 1).unwrap();
                execv(&path, &[&path]).unwrap();
                panic!("UNEXPECTED SERVER TERMINATION")
            }

            ForkResult::Parent { child: _ } => {
                close(request_pipe.0).unwrap();
                close(response_pipe.1).unwrap();
                let request_channel = unsafe { File::from_raw_fd(request_pipe.1) };
                let response_channel = unsafe { File::from_raw_fd(response_pipe.0) };
                let unreturned = Arc::new(Mutex::new(HashMap::new()));

                tokio::spawn(listen(
                    BufReader::new(response_channel),
                    event_sender.clone(),
                    unreturned.clone(),
                ));

                Client {
                    request_writer: BufWriter::new(request_channel),
                    unreturned,
                }
            }
        }
    }

    pub async fn request(&mut self, content: Request) -> Response {
        let (response_sender, response_receiver) = oneshot::channel();

        self.unreturned
            .lock()
            .unwrap()
            .insert(content.id, response_sender);

        let content = serde_json::to_vec(&content).unwrap();
        self.send(content).await;

        response_receiver.await.unwrap()
    }

    pub async fn notify(&mut self, content: Notification) {
        let content = serde_json::to_vec(&content).unwrap();
        self.send(content).await;
    }

    async fn send(&mut self, content: Vec<u8>) {
        self.request_writer
            .write(format!("Content-Length: {}\r\n\r\n", content.len()).as_bytes())
            .await
            .unwrap();
        self.request_writer.write(&content).await.unwrap();
        self.request_writer.flush().await.unwrap();
    }
}

async fn listen(
    mut response_reader: BufReader<File>,
    event_sender: mpsc::UnboundedSender<Event>,
    unreturned: Arc<Mutex<HashMap<i32, oneshot::Sender<Response>>>>,
) {
    loop {
        let content = read_response(&mut response_reader).await;
        let content: ServerMessage = serde_json::from_str(&content).unwrap();
        match content {
            ServerMessage::Response(response) => {
                let response_id = response.id;
                let sender: oneshot::Sender<Response> =
                    unreturned.lock().unwrap().remove(&response_id).unwrap();
                sender.send(response).unwrap();
            }
            ServerMessage::Notification(notification) => {
                event_sender
                    .send(Event::LanguageNotification(notification))
                    .unwrap();
            }
        }
    }
}

async fn read_response(response_buffer: &mut BufReader<File>) -> String {
    let content_length = read_response_header(response_buffer).await;
    let mut buffer = vec![0; content_length];
    response_buffer.read_exact(&mut buffer).await.unwrap();
    std::str::from_utf8(&buffer).unwrap().to_string()
}

async fn read_response_header(response_buffer: &mut BufReader<File>) -> usize {
    // TODO: handle Content-Type
    let mut content_length = 0;
    let mut buffer = [0; 16]; // "Content-Length: "
    response_buffer.read_exact(&mut buffer).await.unwrap();
    loop {
        response_buffer.read(&mut buffer[0..1]).await.unwrap();
        if buffer[0] as char == '\r' {
            response_buffer.read_exact(&mut buffer[0..3]).await.unwrap(); // "\n\r\n"
            break;
        }
        let digit = buffer[0] - 48;
        content_length = content_length * 10 + digit as usize;
    }
    content_length
}
