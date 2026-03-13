//! Unit tests for mcp/transport.rs.

use k8s_mcp::mcp::transport::{StdioTransport, SyncStdioTransport, Transport};
use std::io::Cursor;

// ============================================================================
// Async StdioTransport Tests
// ============================================================================

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

// ============================================================================
// SyncStdioTransport Tests
// ============================================================================

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

// ============================================================================
// Transport Trait Tests
// ============================================================================

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

    let result = <StdioTransport<Cursor<Vec<u8>>, Vec<u8>> as Transport>::receive(&mut transport)
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

// ============================================================================
// Edge Cases
// ============================================================================

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
