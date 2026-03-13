//! MCP Transport layer - stdio implementation.

use crate::error::{Error, Result};
use async_trait::async_trait;
use std::io::{self, BufRead, Write};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, trace};

/// Transport trait for MCP communication.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Send a message to the client.
    async fn send(&mut self, message: &str) -> Result<()>;

    /// Receive a message from the client.
    async fn receive(&mut self) -> Result<Option<String>>;

    /// Close the transport.
    async fn close(&mut self) -> Result<()>;
}

/// Stdio transport for MCP communication.
pub struct StdioTransport<R: AsyncRead + Unpin + Send + Sync, W: AsyncWrite + Unpin + Send + Sync> {
    reader: BufReader<R>,
    writer: W,
    closed: bool,
}

impl StdioTransport<tokio::io::Stdin, tokio::io::Stdout> {
    /// Create a new stdio transport using standard input/output.
    pub fn new() -> Self {
        Self::with_streams(tokio::io::stdin(), tokio::io::stdout())
    }
}

impl Default for StdioTransport<tokio::io::Stdin, tokio::io::Stdout> {
    fn default() -> Self {
        Self::new()
    }
}

impl<R: AsyncRead + Unpin + Send + Sync, W: AsyncWrite + Unpin + Send + Sync> StdioTransport<R, W> {
    /// Create a new transport with custom read/write streams.
    pub fn with_streams(reader: R, writer: W) -> Self {
        StdioTransport {
            reader: BufReader::new(reader),
            writer,
            closed: false,
        }
    }
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send + Sync, W: AsyncWrite + Unpin + Send + Sync> Transport
    for StdioTransport<R, W>
{
    async fn send(&mut self, message: &str) -> Result<()> {
        if self.closed {
            return Err(Error::Protocol("Transport is closed".to_string()));
        }

        trace!("Sending message: {}", message);

        // Write the message followed by newline
        self.writer.write_all(message.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;

        debug!("Message sent successfully");
        Ok(())
    }

    async fn receive(&mut self) -> Result<Option<String>> {
        if self.closed {
            return Ok(None);
        }

        let mut line = String::new();
        let bytes_read = self.reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            // EOF reached
            debug!("EOF reached on input stream");
            self.closed = true;
            return Ok(None);
        }

        // Remove trailing newline
        let line = line.trim_end().to_string();

        // Skip empty lines
        if line.is_empty() {
            return self.receive().await;
        }

        trace!("Received message: {}", line);
        Ok(Some(line))
    }

    async fn close(&mut self) -> Result<()> {
        if !self.closed {
            debug!("Closing transport");
            self.closed = true;
        }
        Ok(())
    }
}

/// Synchronous stdio transport for simpler use cases.
pub struct SyncStdioTransport {
    closed: bool,
}

impl SyncStdioTransport {
    /// Create a new synchronous stdio transport.
    pub fn new() -> Self {
        SyncStdioTransport { closed: false }
    }

    /// Send a message synchronously.
    pub fn send(&mut self, message: &str) -> Result<()> {
        if self.closed {
            return Err(Error::Protocol("Transport is closed".to_string()));
        }

        trace!("Sending message (sync): {}", message);

        let stdout = io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(message.as_bytes())?;
        handle.write_all(b"\n")?;
        handle.flush()?;

        debug!("Message sent successfully (sync)");
        Ok(())
    }

    /// Receive a message synchronously.
    pub fn receive(&mut self) -> Result<Option<String>> {
        if self.closed {
            return Ok(None);
        }

        let stdin = io::stdin();
        let mut line = String::new();
        let bytes_read = stdin.lock().read_line(&mut line)?;

        if bytes_read == 0 {
            debug!("EOF reached on input stream (sync)");
            self.closed = true;
            return Ok(None);
        }

        let line = line.trim_end().to_string();

        if line.is_empty() {
            return self.receive();
        }

        trace!("Received message (sync): {}", line);
        Ok(Some(line))
    }

    /// Close the transport.
    pub fn close(&mut self) -> Result<()> {
        if !self.closed {
            debug!("Closing transport (sync)");
            self.closed = true;
        }
        Ok(())
    }
}

impl Default for SyncStdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // Async StdioTransport Tests

    #[tokio::test]
    async fn test_stdio_transport_send_receive() {
        let input = Cursor::new(b"test message\n".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        // Receive message
        let msg = transport.receive().await.unwrap();
        assert_eq!(msg, Some("test message".to_string()));

        // Send response
        transport.send("response").await.unwrap();
    }

    #[tokio::test]
    async fn test_stdio_transport_multiple_messages() {
        let input = Cursor::new(b"msg1\nmsg2\nmsg3\n".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        // Receive all messages
        let msg1 = transport.receive().await.unwrap();
        assert_eq!(msg1, Some("msg1".to_string()));

        let msg2 = transport.receive().await.unwrap();
        assert_eq!(msg2, Some("msg2".to_string()));

        let msg3 = transport.receive().await.unwrap();
        assert_eq!(msg3, Some("msg3".to_string()));

        // EOF after all messages
        let msg4 = transport.receive().await.unwrap();
        assert_eq!(msg4, None);
    }

    #[tokio::test]
    async fn test_stdio_transport_empty_lines() {
        let input = Cursor::new(b"\n\nreal message\n".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        // Empty lines should be skipped
        let msg = transport.receive().await.unwrap();
        assert_eq!(msg, Some("real message".to_string()));
    }

    #[tokio::test]
    async fn test_stdio_transport_eof() {
        let input = Cursor::new(b"".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        let msg = transport.receive().await.unwrap();
        assert_eq!(msg, None);
        assert!(transport.closed);
    }

    #[tokio::test]
    async fn test_stdio_transport_send_after_close() {
        let input = Cursor::new(b"".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        // Trigger EOF
        transport.receive().await.unwrap();

        // Send should fail after close
        let result = transport.send("test").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stdio_transport_receive_after_close() {
        let input = Cursor::new(b"".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        // Trigger EOF
        transport.receive().await.unwrap();

        // Receive should return None after close
        let msg = transport.receive().await.unwrap();
        assert_eq!(msg, None);
    }

    #[tokio::test]
    async fn test_stdio_transport_close() {
        let input = Cursor::new(b"test\n".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        transport.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_stdio_transport_close_idempotent() {
        let input = Cursor::new(b"".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        // Close multiple times should be fine
        transport.close().await.unwrap();
        transport.close().await.unwrap();
        transport.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_stdio_transport_json_message() {
        let json_msg = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
        let input = Cursor::new(format!("{}\n", json_msg).into_bytes());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        let msg = transport.receive().await.unwrap();
        assert_eq!(msg, Some(json_msg.to_string()));
    }

    // SyncStdioTransport Tests

    #[test]
    fn test_sync_stdio_transport_new() {
        let transport = SyncStdioTransport::new();
        let _ = transport;
    }

    #[test]
    fn test_sync_stdio_transport_default() {
        let transport = SyncStdioTransport::default();
        let _ = transport;
    }

    #[test]
    fn test_sync_stdio_transport_close() {
        let mut transport = SyncStdioTransport::new();
        transport.close().unwrap();
    }

    #[test]
    fn test_sync_stdio_transport_close_idempotent() {
        let mut transport = SyncStdioTransport::new();

        transport.close().unwrap();
        transport.close().unwrap();
        transport.close().unwrap();
    }

    #[test]
    fn test_sync_stdio_transport_send_after_close() {
        let mut transport = SyncStdioTransport::new();
        transport.close().unwrap();

        let result = transport.send("test");
        assert!(result.is_err());
    }

    #[test]
    fn test_sync_stdio_transport_receive_after_close() {
        let mut transport = SyncStdioTransport::new();
        transport.close().unwrap();

        let result = transport.receive().unwrap();
        assert_eq!(result, None);
    }

    // Transport Trait Tests

    #[tokio::test]
    async fn test_transport_trait_send() {
        let input = Cursor::new(b"".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        // Test that Transport trait is implemented
        let result = <StdioTransport<Cursor<Vec<u8>>, Vec<u8>> as Transport>::send(
            &mut transport,
            "test message",
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_transport_trait_receive() {
        let input = Cursor::new(b"message\n".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        let result =
            <StdioTransport<Cursor<Vec<u8>>, Vec<u8>> as Transport>::receive(&mut transport)
                .await
                .unwrap();
        assert_eq!(result, Some("message".to_string()));
    }

    #[tokio::test]
    async fn test_transport_trait_close() {
        let input = Cursor::new(b"".to_vec());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        <StdioTransport<Cursor<Vec<u8>>, Vec<u8>> as Transport>::close(&mut transport)
            .await
            .unwrap();
    }

    // Edge Cases

    #[tokio::test]
    async fn test_stdio_transport_unicode() {
        let unicode_msg = "Hello, 世界! 🌍";
        let input = Cursor::new(format!("{}\n", unicode_msg).into_bytes());
        let output: Vec<u8> = Vec::new();

        let mut transport: StdioTransport<Cursor<Vec<u8>>, Vec<u8>> =
            StdioTransport::with_streams(input, output);

        let msg = transport.receive().await.unwrap();
        assert_eq!(msg, Some(unicode_msg.to_string()));
    }
}
