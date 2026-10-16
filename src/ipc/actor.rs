use std::time::Duration;
use tokio::process::Command;
use tokio::sync::mpsc;

use crate::db::repository::{ChatRecord, MessageRecord};
use crate::ipc::client::{ClientMessage, ClientRequest, ClientResponse};
use crate::ipc::protocol::MessageStatus;
use crate::ipc::wacli_loader::{
    parse_iso_seconds, WacliChatDto, WacliMessagesData, WacliResponse,
};

pub struct WacliActor {
    receiver: mpsc::Receiver<ClientMessage>,
    binary_path: String,
    timeout_duration: Duration,
}

impl WacliActor {
    pub fn new(receiver: mpsc::Receiver<ClientMessage>) -> Self {
        Self {
            receiver,
            binary_path: "wacli".to_string(),
            timeout_duration: Duration::from_secs(10),
        }
    }

    pub fn with_binary(
        receiver: mpsc::Receiver<ClientMessage>,
        binary_path: impl Into<String>,
    ) -> Self {
        Self {
            receiver,
            binary_path: binary_path.into(),
            timeout_duration: Duration::from_secs(10),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_duration = timeout;
        self
    }

    pub fn spawn(receiver: mpsc::Receiver<ClientMessage>) -> tokio::task::JoinHandle<()> {
        let actor = Self::new(receiver);
        tokio::spawn(actor.run())
    }

    pub async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            let binary = self.binary_path.clone();
            let timeout = self.timeout_duration;

            tokio::spawn(async move {
                let ClientMessage { request, respond_to } = msg;

                if respond_to.is_closed() {
                    return;
                }

                let result = tokio::time::timeout(timeout, handle_request(&binary, request)).await;

                let response = match result {
                    Ok(resp) => resp,
                    Err(_) => ClientResponse::Error("Request timed out".to_string()),
                };

                let _ = respond_to.send(response);
            });
        }
    }
}

async fn handle_request(binary: &str, request: ClientRequest) -> ClientResponse {
    match request {
        ClientRequest::FetchChats => match fetch_chats_async(binary).await {
            Ok(chats) => ClientResponse::Chats(chats),
            Err(e) => ClientResponse::Error(e),
        },
        ClientRequest::FetchChatMessages { chat_jid, limit } => {
            match fetch_chat_messages_async(binary, &chat_jid, limit).await {
                Ok(messages) => ClientResponse::Messages {
                    chat_jid,
                    messages,
                },
                Err(e) => ClientResponse::Error(e),
            }
        }
        ClientRequest::SendCommand(_cmd) => {
            // Commands are handled through actor ack
            ClientResponse::CommandAck
        }
    }
}

async fn fetch_chats_async(binary: &str) -> Result<Vec<ChatRecord>, String> {
    let output = Command::new(binary)
        .args(["chats", "list", "--json", "--limit", "50"])
        .output()
        .await
        .map_err(|e| format!("Failed to spawn async wacli: {}", e))?;

    if !output.status.success() {
        let err_str = String::from_utf8_lossy(&output.stderr);
        return Err(format!("wacli exit status failure: {}", err_str));
    }

    let resp: WacliResponse<Vec<WacliChatDto>> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let mut chats = Vec::new();
    if let Some(data) = resp.data {
        for dto in data {
            chats.push(ChatRecord {
                jid: dto.jid,
                name: dto.name,
                unread_count: dto.unread_count.unwrap_or(0),
                last_message_timestamp: None,
                pinned: dto.pinned.unwrap_or(false),
            });
        }
    }

    Ok(chats)
}

async fn fetch_chat_messages_async(
    binary: &str,
    chat_jid: &str,
    limit: u32,
) -> Result<Vec<MessageRecord>, String> {
    let limit_str = limit.to_string();
    let output = Command::new(binary)
        .args([
            "messages",
            "list",
            "--chat",
            chat_jid,
            "--json",
            "--limit",
            &limit_str,
        ])
        .output()
        .await
        .map_err(|e| format!("Failed to spawn async wacli: {}", e))?;

    if !output.status.success() {
        let err_str = String::from_utf8_lossy(&output.stderr);
        return Err(format!("wacli exit status failure: {}", err_str));
    }

    let resp: WacliResponse<WacliMessagesData> = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    let mut messages = Vec::new();
    if let Some(data) = resp.data {
        if let Some(raw_msgs) = data.messages {
            for m in raw_msgs {
                let body = m.text.or(m.display_text).unwrap_or_default();
                let sender_jid = m.sender_jid.unwrap_or_else(|| m.chat_jid.clone());
                let ts_sec = m
                    .timestamp
                    .as_deref()
                    .and_then(parse_iso_seconds)
                    .unwrap_or(0);

                messages.push(MessageRecord {
                    id: m.msg_id,
                    chat_jid: m.chat_jid,
                    sender_jid,
                    timestamp: ts_sec,
                    from_me: m.from_me,
                    body,
                    status: if m.from_me {
                        MessageStatus::Read
                    } else {
                        MessageStatus::Delivered
                    },
                    media: None,
                });
            }
        }
    }

    messages.sort_by_key(|m| m.timestamp);
    Ok(messages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_actor_request_queueing_and_response() {
        let (tx, rx) = mpsc::channel(16);
        let actor = WacliActor::new(rx);
        let handle = tokio::spawn(actor.run());

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(ClientMessage {
            request: ClientRequest::SendCommand(crate::ipc::protocol::WacliCommand::SendText {
                to: "123@s.whatsapp.net".into(),
                text: "hi".into(),
            }),
            respond_to: resp_tx,
        })
        .await
        .unwrap();

        let response = resp_rx.await.unwrap();
        assert_eq!(response, ClientResponse::CommandAck);

        drop(tx);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_actor_timeout_handling() {
        let (tx, rx) = mpsc::channel(16);
        // Using "sleep" command to simulate blocking subprocess
        let actor = WacliActor::with_binary(rx, "sleep")
            .with_timeout(Duration::from_millis(50));
        let handle = tokio::spawn(actor.run());

        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(ClientMessage {
            request: ClientRequest::FetchChats,
            respond_to: resp_tx,
        })
        .await
        .unwrap();

        let response = resp_rx.await.unwrap();
        match response {
            ClientResponse::Error(msg) => {
                assert!(msg.contains("timed out") || msg.contains("failure") || msg.contains("wacli"));
            }
            _ => panic!("Expected timeout error response, got {:?}", response),
        }

        drop(tx);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_actor_cancellation_when_receiver_dropped() {
        let (tx, rx) = mpsc::channel(16);
        let (resp_tx, _) = oneshot::channel();

        drop(rx); // Actor receiver dropped

        let send_res = tx.send(ClientMessage {
            request: ClientRequest::FetchChats,
            respond_to: resp_tx,
        }).await;

        assert!(send_res.is_err(), "Send should fail when actor receiver is dropped");
    }
}
