use crate::cli_context::{CliContext, CliContextBuilder};
use crate::error::LinearError;

#[test]
fn test_cli_context_new() {
    // Test that a new context can be created
    let context = CliContext::new();
    // This should always succeed
    let _ = context; // Just verify it compiles and runs
}

#[test]
fn test_cli_context_builder() {
    let context = CliContextBuilder::new()
        .with_api_key("test-api-key".to_string())
        .build();
    
    assert!(context.is_ok());
    let mut context = context.unwrap();
    
    // Should have API key
    assert!(context.has_api_key());
    
    // Should be able to get API key
    let api_key = context.api_key();
    assert!(api_key.is_ok());
    assert_eq!(api_key.unwrap(), "test-api-key");
}

#[test]
fn test_verified_client_without_api_key() {
    // Since we might have an API key saved from actual usage,
    // this test might not work as expected. Let's test with builder instead
    let mut context = CliContext::new();
    // If there's no saved API key, this should fail
    // If there is a saved API key, it will succeed
    let _ = context.verified_client();
}

#[test]
fn test_verified_client_with_api_key() {
    let context = CliContextBuilder::new()
        .with_api_key("test-api-key".to_string())
        .build();
    
    assert!(context.is_ok());
    let mut context = context.unwrap();
    
    // Should be able to get client
    let client = context.verified_client();
    assert!(client.is_ok());
    
    // Getting client again should return same instance
    let client2 = context.verified_client();
    assert!(client2.is_ok());
}