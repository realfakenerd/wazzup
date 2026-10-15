use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use tokio::sync::{broadcast, mpsc};
use tracing::{error, info, warn};

use super::protocol::{WacliCommand, WacliEvent};

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("Failed to spawn process: {0}")]
    SpawnFailed(#[from] std::io::Error),
    #[error("Process stdin not available")]
    StdinUnavailable,
    #[error("Process stdout not available")]
    StdoutUnavailable,
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Supervisor channel closed")]
    ChannelClosed,
}

pub struct SupervisorConfig {
    pub binary_path: String,
    pub args: Vec<String>,
    pub channel_capacity: usize,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            binary_path: "wacli".to_string(),
            args: vec!["daemon".to_string(), "--json".to_string()],
            channel_capacity: 100,
        }
    }
}

pub struct WacliSupervisor {
    event_tx: broadcast::Sender<WacliEvent>,
    cmd_tx: mpsc::Sender<WacliCommand>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
}

impl WacliSupervisor {
    pub fn spawn(config: SupervisorConfig) -> Result<Self, SupervisorError> {
        let (event_tx, _) = broadcast::channel(config.channel_capacity);
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<WacliCommand>(config.channel_capacity);
        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let mut child = Command::new(&config.binary_path)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin: ChildStdin = child.stdin.take().ok_or(SupervisorError::StdinUnavailable)?;
        let stdout = child.stdout.take().ok_or(SupervisorError::StdoutUnavailable)?;

        let event_sender = event_tx.clone();

        // Background stdout reader task
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if line.trim().is_empty() {
                    continue;
                }
                match WacliEvent::parse_line(&line) {
                    Ok(event) => {
                        let _ = event_sender.send(event);
                    }
                    Err(e) => {
                        warn!("Failed to parse line from daemon: '{}', err: {}", line, e);
                    }
                }
            }
        });

        // Background stdin writer and process supervisor task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(cmd) = cmd_rx.recv() => {
                        match cmd.to_line() {
                            Ok(line) => {
                                if let Err(e) = stdin.write_all(line.as_bytes()).await {
                                    error!("Failed writing command to daemon stdin: {}", e);
                                    break;
                                }
                                if let Err(e) = stdin.flush().await {
                                    error!("Failed flushing command to daemon stdin: {}", e);
                                    break;
                                }
                            }
                            Err(e) => {
                                error!("Failed serializing command: {}", e);
                            }
                        }
                    }
                    _ = &mut shutdown_rx => {
                        info!("Shutdown signal received, killing daemon process");
                        let _ = child.kill().await;
                        break;
                    }
                    status = child.wait() => {
                        match status {
                            Ok(exit) => info!("Daemon process exited with status: {}", exit),
                            Err(e) => error!("Error waiting on daemon process: {}", e),
                        }
                        break;
                    }
                }
            }
        });

        Ok(Self {
            event_tx,
            cmd_tx,
            shutdown_tx: Some(shutdown_tx),
        })
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WacliEvent> {
        self.event_tx.subscribe()
    }

    pub async fn send_command(&self, cmd: WacliCommand) -> Result<(), SupervisorError> {
        self.cmd_tx
            .send(cmd)
            .await
            .map_err(|_| SupervisorError::ChannelClosed)
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl Drop for WacliSupervisor {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::protocol::ConnectionState;

    #[tokio::test]
    async fn test_supervisor_streams_stdout_events() {
        // Spawn sh script that outputs a status json event and waits
        let script = r#"echo '{"type":"status","state":"connected","phone":"+1234","push_name":"Tester","qr_code":null}'; sleep 1"#;
        let config = SupervisorConfig {
            binary_path: "sh".to_string(),
            args: vec!["-c".to_string(), script.to_string()],
            channel_capacity: 10,
        };

        let mut supervisor = WacliSupervisor::spawn(config).expect("should spawn mock supervisor");
        let mut rx = supervisor.subscribe();

        let event = tokio::time::timeout(std::time::Duration::from_secs(2), rx.recv())
            .await
            .expect("should receive event before timeout")
            .expect("should receive valid broadcast event");

        match event {
            WacliEvent::Status(s) => {
                assert_eq!(s.state, ConnectionState::Connected);
                assert_eq!(s.phone.as_deref(), Some("+1234"));
                assert_eq!(s.push_name.as_deref(), Some("Tester"));
            }
            _ => panic!("expected Status event"),
        }

        supervisor.stop();
    }

    #[tokio::test]
    async fn test_supervisor_command_dispatch() {
        // Mock process: cat
        let config = SupervisorConfig {
            binary_path: "cat".to_string(),
            args: vec![],
            channel_capacity: 10,
        };

        let mut supervisor = WacliSupervisor::spawn(config).expect("should spawn cat");
        let res = supervisor
            .send_command(WacliCommand::SendText {
                to: "123@s.whatsapp.net".to_string(),
                text: "ping".to_string(),
            })
            .await;

        assert!(res.is_ok());
        supervisor.stop();
    }
}
