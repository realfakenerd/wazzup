use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use crate::db::repository::{ChatRecord, MessageRecord};
use crate::ipc::protocol::WacliCommand;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientRequest {
    FetchChats,
    FetchChatMessages { chat_jid: String, limit: u32 },
    SyncChatHistory { chat_jid: String, backfill_count: u32 },
    SendCommand(WacliCommand),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientResponse {
    Chats(Vec<ChatRecord>),
    Messages { chat_jid: String, messages: Vec<MessageRecord> },
    CommandAck,
    Error(String),
}

pub type ResponseSender = oneshot::Sender<ClientResponse>;

pub struct ClientMessage {
    pub request: ClientRequest,
    pub respond_to: ResponseSender,
}

pub trait WhatsAppClient: Send + Sync {
    fn fetch_chats(&self) -> BoxFuture<'_, Result<Vec<ChatRecord>, String>>;
    fn fetch_messages<'a>(
        &'a self,
        chat_jid: &'a str,
        limit: u32,
    ) -> BoxFuture<'a, Result<Vec<MessageRecord>, String>>;
    fn sync_chat_history<'a>(
        &'a self,
        chat_jid: &'a str,
        backfill_count: u32,
    ) -> BoxFuture<'a, Result<Vec<MessageRecord>, String>>;
    fn send_command(
        &self,
        cmd: WacliCommand,
    ) -> BoxFuture<'_, Result<(), String>>;
}

/// In-memory Mock client for fast testing and offline preview
#[derive(Clone, Default)]
pub struct MockWhatsAppClient {
    chats: Arc<Mutex<Vec<ChatRecord>>>,
    messages: Arc<Mutex<HashMap<String, Vec<MessageRecord>>>>,
}

impl MockWhatsAppClient {
    pub fn new() -> Self {
        Self {
            chats: Arc::new(Mutex::new(Vec::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_fixture(chats: Vec<ChatRecord>, messages: HashMap<String, Vec<MessageRecord>>) -> Self {
        Self {
            chats: Arc::new(Mutex::new(chats)),
            messages: Arc::new(Mutex::new(messages)),
        }
    }

    pub fn add_chat(&self, chat: ChatRecord) {
        self.chats.lock().unwrap().push(chat);
    }

    pub fn add_message(&self, msg: MessageRecord) {
        self.messages
            .lock()
            .unwrap()
            .entry(msg.chat_jid.clone())
            .or_default()
            .push(msg);
    }
}

impl WhatsAppClient for MockWhatsAppClient {
    fn fetch_chats(&self) -> BoxFuture<'_, Result<Vec<ChatRecord>, String>> {
        Box::pin(async move {
            Ok(self.chats.lock().unwrap().clone())
        })
    }

    fn fetch_messages<'a>(
        &'a self,
        chat_jid: &'a str,
        limit: u32,
    ) -> BoxFuture<'a, Result<Vec<MessageRecord>, String>> {
        Box::pin(async move {
            let msgs = self.messages.lock().unwrap();
            let list = msgs.get(chat_jid).cloned().unwrap_or_default();
            let truncated = list.into_iter().take(limit as usize).collect();
            Ok(truncated)
        })
    }

    fn sync_chat_history<'a>(
        &'a self,
        chat_jid: &'a str,
        _backfill_count: u32,
    ) -> BoxFuture<'a, Result<Vec<MessageRecord>, String>> {
        self.fetch_messages(chat_jid, 1000)
    }

    fn send_command(
        &self,
        _cmd: WacliCommand,
    ) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            Ok(())
        })
    }
}

/// Actor handle communicating over non-blocking mpsc channel
#[derive(Clone)]
pub struct ChannelWhatsAppClient {
    sender: mpsc::Sender<ClientMessage>,
}

impl ChannelWhatsAppClient {
    pub fn new(sender: mpsc::Sender<ClientMessage>) -> Self {
        Self { sender }
    }
}

impl WhatsAppClient for ChannelWhatsAppClient {
    fn fetch_chats(&self) -> BoxFuture<'_, Result<Vec<ChatRecord>, String>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            self.sender
                .send(ClientMessage {
                    request: ClientRequest::FetchChats,
                    respond_to: tx,
                })
                .await
                .map_err(|e| format!("Actor channel send error: {}", e))?;

            match rx.await {
                Ok(ClientResponse::Chats(chats)) => Ok(chats),
                Ok(ClientResponse::Error(err)) => Err(err),
                _ => Err("Invalid response type for FetchChats".to_string()),
            }
        })
    }

    fn fetch_messages<'a>(
        &'a self,
        chat_jid: &'a str,
        limit: u32,
    ) -> BoxFuture<'a, Result<Vec<MessageRecord>, String>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            self.sender
                .send(ClientMessage {
                    request: ClientRequest::FetchChatMessages {
                        chat_jid: chat_jid.to_string(),
                        limit,
                    },
                    respond_to: tx,
                })
                .await
                .map_err(|e| format!("Actor channel send error: {}", e))?;

            match rx.await {
                Ok(ClientResponse::Messages { messages, .. }) => Ok(messages),
                Ok(ClientResponse::Error(err)) => Err(err),
                _ => Err("Invalid response type for FetchChatMessages".to_string()),
            }
        })
    }

    fn sync_chat_history<'a>(
        &'a self,
        chat_jid: &'a str,
        backfill_count: u32,
    ) -> BoxFuture<'a, Result<Vec<MessageRecord>, String>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            self.sender
                .send(ClientMessage {
                    request: ClientRequest::SyncChatHistory {
                        chat_jid: chat_jid.to_string(),
                        backfill_count,
                    },
                    respond_to: tx,
                })
                .await
                .map_err(|e| format!("Actor channel send error: {}", e))?;

            match rx.await {
                Ok(ClientResponse::Messages { messages, .. }) => Ok(messages),
                Ok(ClientResponse::Error(err)) => Err(err),
                _ => Err("Invalid response type for SyncChatHistory".to_string()),
            }
        })
    }

    fn send_command(
        &self,
        cmd: WacliCommand,
    ) -> BoxFuture<'_, Result<(), String>> {
        Box::pin(async move {
            let (tx, rx) = oneshot::channel();
            self.sender
                .send(ClientMessage {
                    request: ClientRequest::SendCommand(cmd),
                    respond_to: tx,
                })
                .await
                .map_err(|e| format!("Actor channel send error: {}", e))?;

            match rx.await {
                Ok(ClientResponse::CommandAck) => Ok(()),
                Ok(ClientResponse::Error(err)) => Err(err),
                _ => Err("Invalid response type for SendCommand".to_string()),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::MessageStatus;

    #[tokio::test]
    async fn test_mock_client_lifecycle() {
        let client = MockWhatsAppClient::new();
        assert!(client.fetch_chats().await.unwrap().is_empty());

        let chat = ChatRecord {
            jid: "123@s.whatsapp.net".to_string(),
            name: Some("Alice".to_string()),
            unread_count: 2,
            last_message_timestamp: Some(100),
            pinned: true,
        };
        client.add_chat(chat.clone());

        let chats = client.fetch_chats().await.unwrap();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].name.as_deref(), Some("Alice"));

        let msg = MessageRecord {
            id: "msg_1".to_string(),
            chat_jid: "123@s.whatsapp.net".to_string(),
            sender_jid: "123@s.whatsapp.net".to_string(),
            timestamp: 100,
            from_me: false,
            body: "Hi Alice".to_string(),
            status: MessageStatus::Read,
            media: None,
        };
        client.add_message(msg);

        let messages = client.fetch_messages("123@s.whatsapp.net", 10).await.unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].body, "Hi Alice");

        // sync_chat_history succeeds
        let sync_msgs = client.sync_chat_history("123@s.whatsapp.net", 50).await.unwrap();
        assert_eq!(sync_msgs.len(), 1);

        // Send command succeeds
        assert!(client.send_command(WacliCommand::SyncChats).await.is_ok());
    }

    #[tokio::test]
    async fn test_channel_client_dispatch() {
        let (tx, mut rx) = mpsc::channel::<ClientMessage>(16);
        let client = ChannelWhatsAppClient::new(tx);

        // Spawn mock actor loop
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg.request {
                    ClientRequest::FetchChats => {
                        let _ = msg.respond_to.send(ClientResponse::Chats(vec![ChatRecord {
                            jid: "test@s.whatsapp.net".to_string(),
                            name: Some("Test".to_string()),
                            unread_count: 0,
                            last_message_timestamp: None,
                            pinned: false,
                        }]));
                    }
                    ClientRequest::FetchChatMessages { chat_jid, .. } => {
                        let _ = msg.respond_to.send(ClientResponse::Messages {
                            chat_jid,
                            messages: vec![],
                        });
                    }
                    ClientRequest::SyncChatHistory { chat_jid, .. } => {
                        let _ = msg.respond_to.send(ClientResponse::Messages {
                            chat_jid,
                            messages: vec![],
                        });
                    }
                    ClientRequest::SendCommand(_) => {
                        let _ = msg.respond_to.send(ClientResponse::CommandAck);
                    }
                }
            }
        });

        let chats = client.fetch_chats().await.unwrap();
        assert_eq!(chats.len(), 1);
        assert_eq!(chats[0].jid, "test@s.whatsapp.net");

        let messages = client.fetch_messages("test@s.whatsapp.net", 10).await.unwrap();
        assert!(messages.is_empty());

        let sync_msgs = client.sync_chat_history("test@s.whatsapp.net", 50).await.unwrap();
        assert!(sync_msgs.is_empty());

        let cmd_res = client.send_command(WacliCommand::GetStatus).await;
        assert!(cmd_res.is_ok());
    }
}
