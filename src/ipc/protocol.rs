use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected,
    QrReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Image,
    Audio,
    Video,
    Document,
    Sticker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaPayload {
    pub media_type: MediaType,
    pub mimetype: String,
    pub file_path: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusEvent {
    pub state: ConnectionState,
    pub phone: Option<String>,
    pub push_name: Option<String>,
    pub qr_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageEvent {
    pub id: String,
    pub chat_jid: String,
    pub sender_jid: String,
    pub timestamp: i64,
    pub from_me: bool,
    pub body: String,
    pub status: MessageStatus,
    pub media: Option<MediaPayload>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatEvent {
    pub jid: String,
    pub name: Option<String>,
    pub unread_count: u32,
    pub last_message_timestamp: Option<i64>,
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WacliEvent {
    Status(StatusEvent),
    Message(MessageEvent),
    Chat(ChatEvent),
    Error(ErrorEvent),
}

impl WacliEvent {
    pub fn parse_line(line: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(line.trim())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum WacliCommand {
    SendText {
        to: String,
        text: String,
    },
    FetchHistory {
        chat_jid: String,
        limit: u32,
    },
    SyncChats,
    GetStatus,
}

impl WacliCommand {
    pub fn to_line(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string(self)?;
        json.push('\n');
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_status_event_connected() {
        let json = r#"{"type":"status","state":"connected","phone":"+15550199","push_name":"Luke","qr_code":null}"#;
        let event = WacliEvent::parse_line(json).expect("should parse status event");
        match event {
            WacliEvent::Status(s) => {
                assert_eq!(s.state, ConnectionState::Connected);
                assert_eq!(s.phone.as_deref(), Some("+15550199"));
                assert_eq!(s.push_name.as_deref(), Some("Luke"));
                assert_eq!(s.qr_code, None);
            }
            _ => panic!("unexpected event variant"),
        }
    }

    #[test]
    fn test_parse_status_event_qr_ready() {
        let json = r#"{"type":"status","state":"qr_ready","phone":null,"push_name":null,"qr_code":"2@abc..."}"#;
        let event = WacliEvent::parse_line(json).expect("should parse qr event");
        match event {
            WacliEvent::Status(s) => {
                assert_eq!(s.state, ConnectionState::QrReady);
                assert_eq!(s.qr_code.as_deref(), Some("2@abc..."));
            }
            _ => panic!("unexpected event variant"),
        }
    }

    #[test]
    fn test_parse_message_event_text() {
        let json = r#"{
            "type": "message",
            "id": "MSG_1001",
            "chat_jid": "user@s.whatsapp.net",
            "sender_jid": "user@s.whatsapp.net",
            "timestamp": 1776260000,
            "from_me": false,
            "body": "Hello from Wazzup!",
            "status": "delivered",
            "media": null
        }"#;
        let event = WacliEvent::parse_line(json).expect("should parse text message");
        match event {
            WacliEvent::Message(m) => {
                assert_eq!(m.id, "MSG_1001");
                assert_eq!(m.body, "Hello from Wazzup!");
                assert_eq!(m.status, MessageStatus::Delivered);
                assert!(!m.from_me);
                assert!(m.media.is_none());
            }
            _ => panic!("unexpected event variant"),
        }
    }

    #[test]
    fn test_parse_message_event_with_media() {
        let json = r#"{
            "type": "message",
            "id": "MSG_1002",
            "chat_jid": "group@g.us",
            "sender_jid": "friend@s.whatsapp.net",
            "timestamp": 1776260100,
            "from_me": true,
            "body": "Voice note",
            "status": "read",
            "media": {
                "media_type": "audio",
                "mimetype": "audio/ogg; codecs=opus",
                "file_path": "/home/luke/.cache/wazzup/audio_1002.ogg",
                "file_name": "audio_1002.ogg",
                "file_size": 24500
            }
        }"#;
        let event = WacliEvent::parse_line(json).expect("should parse audio message");
        match event {
            WacliEvent::Message(m) => {
                assert_eq!(m.id, "MSG_1002");
                assert_eq!(m.status, MessageStatus::Read);
                assert!(m.from_me);
                let media = m.media.expect("media must be present");
                assert_eq!(media.media_type, MediaType::Audio);
                assert_eq!(media.file_size, Some(24500));
            }
            _ => panic!("unexpected event variant"),
        }
    }

    #[test]
    fn test_parse_chat_event() {
        let json = r#"{
            "type": "chat",
            "jid": "family@g.us",
            "name": "Family Group",
            "unread_count": 5,
            "last_message_timestamp": 1776260200,
            "pinned": true
        }"#;
        let event = WacliEvent::parse_line(json).expect("should parse chat event");
        match event {
            WacliEvent::Chat(c) => {
                assert_eq!(c.jid, "family@g.us");
                assert_eq!(c.name.as_deref(), Some("Family Group"));
                assert_eq!(c.unread_count, 5);
                assert!(c.pinned);
            }
            _ => panic!("unexpected event variant"),
        }
    }

    #[test]
    fn test_command_serialization() {
        let cmd = WacliCommand::SendText {
            to: "1234@s.whatsapp.net".to_string(),
            text: "Hi there!".to_string(),
        };
        let line = cmd.to_line().expect("should serialize command");
        assert!(line.ends_with('\n'));
        assert!(line.contains("\"action\":\"send_text\""));
        assert!(line.contains("\"to\":\"1234@s.whatsapp.net\""));
        assert!(line.contains("\"text\":\"Hi there!\""));
    }
}
