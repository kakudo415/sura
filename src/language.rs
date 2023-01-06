pub mod message;

use nix::unistd;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::unix::prelude::FromRawFd;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

pub async fn start(
    path: String,
    event_dispatcher: mpsc::Sender<crate::app::Event>,
    mut request_queue: mpsc::Receiver<(message::ClientMessage, oneshot::Sender<message::Response>)>,
) {
    let request_pipe = unistd::pipe().unwrap();
    let response_pipe = unistd::pipe().unwrap();

    match unsafe { unistd::fork().unwrap() } {
        unistd::ForkResult::Child => {
            unistd::close(request_pipe.1).unwrap();
            unistd::close(response_pipe.0).unwrap();
            unistd::dup2(request_pipe.0, 0).unwrap();
            unistd::dup2(response_pipe.1, 1).unwrap();
            let path = CString::new(path).unwrap();
            unistd::execv(&path, &[&path]).unwrap();
            panic!("UNEXPECTED SERVER TERMINATION");
        }

        unistd::ForkResult::Parent { child: _ } => {
            unistd::close(request_pipe.0).unwrap();
            unistd::close(response_pipe.1).unwrap();
            let mut request_buffer = unsafe { fs::File::from_raw_fd(request_pipe.1) };
            let mut response_buffer = unsafe { fs::File::from_raw_fd(response_pipe.0) };

            let mut sent_requests = HashMap::<i32, oneshot::Sender<message::Response>>::new();

            loop {
                tokio::select! {
                    response = read_response(&mut response_buffer) => {
                        let response: message::ServerMessage = serde_json::from_str(&response).unwrap();
                        match response {
                            message::ServerMessage::Response(response) => {
                                sent_requests.remove(&response.id).unwrap().send(response).unwrap(); // not sent.get() because of ownership
                            },
                            message::ServerMessage::Notification(notification) => {
                                event_dispatcher.send(crate::app::Event::LanguageServerNotification(notification)).await.unwrap()
                            }
                        }
                    }
                    Some(request) = request_queue.recv() => {
                        let tx = request.1;
                        match request.0 {
                            message::ClientMessage::Request(request) => {
                                let id = request.id;
                                send_request(&mut request_buffer, &message::ClientMessage::Request(request)).await;
                                sent_requests.insert(id, tx);
                            }
                            message::ClientMessage::Notification(notification) => {
                                send_request(&mut request_buffer, &message::ClientMessage::Notification(notification)).await;
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn read_response(response_buffer: &mut fs::File) -> String {
    let content_length = read_response_header(response_buffer).await;
    let mut buffer = vec![0; content_length];
    response_buffer.read_exact(&mut buffer).await.unwrap();
    std::str::from_utf8(&buffer).unwrap().to_string()
}

async fn read_response_header(response_buffer: &mut fs::File) -> usize {
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

async fn send_request(request_buffer: &mut fs::File, request: &message::ClientMessage) {
    let raw_message = serde_json::to_string(request).unwrap();
    let raw_message = raw_message.as_bytes();
    request_buffer
        .write_all(format!("Content-Length: {}\r\n\r\n", raw_message.len()).as_bytes())
        .await
        .unwrap();
    request_buffer.write_all(raw_message).await.unwrap();

    request_buffer.flush().await.unwrap();
}
