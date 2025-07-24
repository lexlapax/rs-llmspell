// ABOUTME: Example demonstrating the ResponseBuilder pattern for creating standardized responses
// ABOUTME: Shows success, error, validation, and complex responses with metadata

use llmspell_utils::response::{
    error_response, success_response, validation_response, ErrorDetails, ResponseBuilder,
    ValidationError,
};
use serde_json::json;

fn main() {
    println!("ResponseBuilder Examples");
    println!("=======================\n");

    // Example 1: Simple success response
    println!("1. Simple Success Response:");
    let response = success_response("create_file", "File created successfully");
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());

    // Example 2: Simple error response
    println!("2. Simple Error Response:");
    let response = error_response("read_file", "File not found");
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());

    // Example 3: Response with result data
    println!("3. Response with Result Data:");
    let response = ResponseBuilder::success("list_files")
        .with_message("Found 3 files")
        .with_result(json!({
            "files": ["file1.txt", "file2.txt", "file3.txt"],
            "total": 3
        }))
        .build();
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());

    // Example 4: Response with metadata
    println!("4. Response with Metadata:");
    let response = ResponseBuilder::success("process_data")
        .with_message("Data processed successfully")
        .with_result(json!({"processed": 100}))
        .with_metadata("duration_ms", json!(250))
        .with_metadata("memory_usage_mb", json!(45.5))
        .build();
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());

    // Example 5: Error with detailed information
    println!("5. Detailed Error Response:");
    let error = ErrorDetails::new("Invalid JSON in request body")
        .with_code("PARSE_ERROR")
        .with_details(json!({
            "line": 5,
            "column": 23,
            "expected": "string",
            "found": "number"
        }));

    let response = ResponseBuilder::success("parse_json")
        .with_error_details(error)
        .build();
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());

    // Example 6: Validation response
    println!("6. Validation Response:");
    let errors = vec![
        ValidationError::new("Field is required")
            .with_field("username")
            .with_code("REQUIRED"),
        ValidationError::new("Invalid email format")
            .with_field("email")
            .with_code("FORMAT"),
        ValidationError::new("Password too short")
            .with_field("password")
            .with_code("MIN_LENGTH"),
    ];

    let response = validation_response(false, &Some(errors));
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());

    // Example 7: File operation response
    println!("7. File Operation Response:");
    let response = ResponseBuilder::success("write_file")
        .with_message("File written successfully")
        .with_file_info("/path/to/file.txt", Some(1024))
        .with_duration_ms(15)
        .build();
    println!("{}\n", serde_json::to_string_pretty(&response).unwrap());

    // Example 8: Build for output (for tool integration)
    println!("8. Build for Output (Tool Integration):");
    let (text, response) = ResponseBuilder::success("calculate")
        .with_message("Calculation completed")
        .with_result(json!({"result": 42}))
        .build_for_output();
    println!("Text output: {}", text);
    println!(
        "JSON response: {}\n",
        serde_json::to_string_pretty(&response).unwrap()
    );

    // Example 9: Validation success
    println!("9. Validation Success:");
    let response = validation_response(true, &None);
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}
