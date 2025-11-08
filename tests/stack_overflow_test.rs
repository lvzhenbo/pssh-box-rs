//! Test to verify that find_boxes_stream doesn't cause stack overflow with large inputs

use std::io::Cursor;
use pssh_box::find_boxes_stream;
use base64::prelude::{Engine as _, BASE64_STANDARD};

#[test]
fn test_no_stack_overflow_with_large_input() {
    // Create a large buffer to test that we don't get stack overflow
    // The old code would allocate 1MB on stack in each iteration
    let mut data = Vec::new();
    
    // Add some valid PSSH boxes (from the finding tests)
    let pssh_bytes = BASE64_STANDARD.decode("AAAAQHBzc2gAAAAA7e+LqXnWSs6jyCfc1R0h7QAAACAiGFlPVVRVQkU6NTM5ZjEyZjRhM2IzMTczYkjj3JWbBg==").unwrap();
    data.extend_from_slice(&pssh_bytes);
    
    // Add 5MB of zeros to force multiple iterations through the read buffer
    data.extend_from_slice(&vec![0u8; 5 * 1024 * 1024]);
    
    let stream = Cursor::new(data);
    
    // This should complete without stack overflow
    let boxes: Vec<_> = find_boxes_stream(stream)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    println!("Found {} boxes without stack overflow", boxes.len());
    assert!(boxes.len() > 0, "Should find at least one PSSH box");
}

#[test]
fn test_multiple_iterations_no_stack_overflow() {
    // Test that we can process multiple MB of data without stack overflow
    let mut data = Vec::new();
    
    // Create 10MB of data with no PSSH boxes to force many iterations
    data.extend_from_slice(&vec![0xFFu8; 10 * 1024 * 1024]);
    
    let stream = Cursor::new(data);
    
    // This should complete without finding any boxes and without stack overflow
    let boxes: Vec<_> = find_boxes_stream(stream)
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    
    assert_eq!(boxes.len(), 0, "Should not find any PSSH boxes in random data");
}
