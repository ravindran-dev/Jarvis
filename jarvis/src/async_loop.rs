use anyhow::Result;
use crossterm::event::Event;
use log::debug;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Async event message types
#[derive(Debug, Clone)]
pub enum AsyncMessage {
    UIEvent(Event),
    SystemUpdate,
    TaskComplete(String),
    Error(String),
}

/// Non-blocking async event loop
pub struct AsyncEventLoop {
    sender: mpsc::UnboundedSender<AsyncMessage>,
    receiver: Option<mpsc::UnboundedReceiver<AsyncMessage>>,
    task_handle: Option<JoinHandle<()>>,
}

impl AsyncEventLoop {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            sender: sender.clone(),
            receiver: Some(receiver),
            task_handle: None,
        }
    }

    pub fn start(&mut self) -> mpsc::UnboundedSender<AsyncMessage> {
        let _sender = self.sender.clone();

        let task = tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                debug!("Async event loop tick");
            }
        });

        self.task_handle = Some(task);
        self.sender.clone()
    }

    pub fn take_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<AsyncMessage>> {
        self.receiver.take()
    }

    pub fn send(&self, msg: AsyncMessage) -> Result<()> {
        self.sender.send(msg)?;
        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

impl Default for AsyncEventLoop {
    fn default() -> Self {
        Self::new()
    }
}

/// Background task runner for non-blocking operations
pub struct BackgroundTaskRunner {
    runtime: Arc<tokio::runtime::Runtime>,
}

impl BackgroundTaskRunner {
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()?;
        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }

    pub fn spawn_task<F>(&self, task: F) -> JoinHandle<()>
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        self.runtime.spawn(task)
    }

    pub fn spawn_blocking<F>(&self, f: F) -> JoinHandle<()>
    where
        F: FnOnce() + Send + 'static,
    {
        self.runtime.spawn(async move {
            let _ = tokio::task::spawn_blocking(f).await;
        })
    }
}

impl Default for BackgroundTaskRunner {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_creation() {
        let loop_obj = AsyncEventLoop::new();
        assert!(loop_obj.receiver.is_some());
    }

    #[test]
    fn test_background_runner_creation() {
        let runner = BackgroundTaskRunner::new();
        assert!(runner.is_ok());
    }

    #[tokio::test]
    async fn test_send_message() {
        let loop_obj = AsyncEventLoop::new();
        let msg = AsyncMessage::TaskComplete("test".to_string());
        assert!(loop_obj.send(msg).is_ok());
    }
}
