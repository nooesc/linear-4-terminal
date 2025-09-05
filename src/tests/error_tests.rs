use crate::error::{LinearError, ErrorContext};
use crate::linear_error;

#[test]
fn test_error_context_on_result() {
    let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "file not found"
    ));
    
    let linear_result = result.context("Failed to read config file");
    assert!(linear_result.is_err());
    
    match linear_result {
        Err(LinearError::Unknown(msg)) => {
            assert!(msg.contains("Failed to read config file"));
            assert!(msg.contains("file not found"));
        }
        _ => panic!("Expected LinearError::Unknown"),
    }
}

#[test]
fn test_error_context_on_option() {
    let option: Option<String> = None;
    let result = option.context("API key not found");
    
    assert!(result.is_err());
    match result {
        Err(LinearError::Unknown(msg)) => {
            assert_eq!(msg, "API key not found");
        }
        _ => panic!("Expected LinearError::Unknown"),
    }
}

#[test]
fn test_error_context_with_closure() {
    let result: Result<i32, std::io::Error> = Err(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "access denied"
    ));
    
    let linear_result = result.with_context(|| {
        format!("Failed to access file at path: {}", "/tmp/test.txt")
    });
    
    assert!(linear_result.is_err());
    match linear_result {
        Err(LinearError::Unknown(msg)) => {
            assert!(msg.contains("Failed to access file at path: /tmp/test.txt"));
            assert!(msg.contains("access denied"));
        }
        _ => panic!("Expected LinearError::Unknown"),
    }
}

#[test]
fn test_linear_error_macro() {
    let error = linear_error!(ApiError, "Request failed");
    match error {
        LinearError::ApiError(msg) => assert_eq!(msg, "Request failed"),
        _ => panic!("Expected LinearError::ApiError"),
    }
    
    let error = linear_error!(InvalidInput, "Invalid filter: {}", "status:invalid");
    match error {
        LinearError::InvalidInput(msg) => assert_eq!(msg, "Invalid filter: status:invalid"),
        _ => panic!("Expected LinearError::InvalidInput"),
    }
}