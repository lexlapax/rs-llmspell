//! ABOUTME: Integration tests for CsvAnalyzerTool with real-world CSV scenarios
//! ABOUTME: Tests complex workflows, large file handling, and error cases

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::CsvAnalyzerTool;
use serde_json::json;

#[tokio::test]
async fn test_complex_csv_analysis_workflow() {
    let tool = CsvAnalyzerTool::default();

    // Complex CSV with mixed types and encoding issues
    let csv_content = r#"employee_id,name,salary,department,hire_date,is_active
101,"Smith, John",75000.50,Engineering,2021-03-15,true
102,"García José",82000,Sales,2020-01-10,true
103,"O'Brien, Mary",,Marketing,2022-06-01,false
104,"李明",65000,Engineering,2019-11-20,true
105,"Müller, Hans",70000.75,HR,2021-08-30,true
"#;

    // First, analyze the CSV
    let analyze_input = AgentInput::text("analyze employee data").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "analyze",
            "input": csv_content
        }),
    );

    let analysis = tool
        .execute(analyze_input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(analysis.text.contains("row_count"));
    assert!(analysis.text.contains("column_count"));
    assert!(analysis.text.contains("encoding"));

    // Verify the analysis results
    let extra = &analysis.metadata.extra;
    let result = extra.get("analysis_result").unwrap();
    assert_eq!(result["row_count"], 5);
    assert_eq!(result["column_count"], 6);
    assert!(result["columns"].is_array());

    // Filter for active employees
    let filter_input = AgentInput::text("filter active employees").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "filter",
            "input": csv_content,
            "options": {
                "filter": "is_active == \"true\""
            }
        }),
    );

    let filtered = tool
        .execute(filter_input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(filtered.text.contains("101"));
    assert!(filtered.text.contains("102"));
    assert!(filtered.text.contains("104"));
    assert!(filtered.text.contains("105"));
    assert!(!filtered.text.contains("103")); // Mary is not active

    // Convert to JSON format
    let convert_input = AgentInput::text("convert to json").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "convert",
            "input": csv_content,
            "options": {
                "format": "json"
            }
        }),
    );

    let json_output = tool
        .execute(convert_input, ExecutionContext::default())
        .await
        .unwrap();

    // Verify JSON is valid
    let parsed: serde_json::Value = serde_json::from_str(&json_output.text).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed.as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn test_malformed_csv_handling() {
    let tool = CsvAnalyzerTool::default();

    // CSV with various issues
    let malformed_csv = r#"id,name,value
1,"Unclosed quote,100
2,Normal,200
3,"Contains
newline",300
4,Missing value at end,
5,"Escaped ""quotes""",500
"#;

    let input = AgentInput::text("analyze malformed csv").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "analyze",
            "input": malformed_csv
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Should handle malformed data gracefully
    assert!(result.text.contains("row_count"));

    let extra = &result.metadata.extra;
    let analysis = extra.get("analysis_result").unwrap();
    // May have parse errors noted
    if analysis.get("parse_errors").is_some() {
        let errors = analysis["parse_errors"].as_array().unwrap();
        assert!(!errors.is_empty());
    }
}

#[tokio::test]
async fn test_large_csv_streaming() {
    let tool = CsvAnalyzerTool::default();

    // Generate a large CSV (but still within default limits)
    let mut csv_content = String::from("id,value,category\n");
    for i in 1..=1000 {
        csv_content.push_str(&format!(
            "{},value_{},{}\n",
            i,
            i,
            if i % 2 == 0 { "A" } else { "B" }
        ));
    }

    let input = AgentInput::text("sample large csv").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "sample",
            "input": csv_content,
            "options": {
                "size": 10,
                "method": "random"
            }
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Count lines in result (header + 10 samples)
    let line_count = result.text.lines().count();
    assert_eq!(line_count, 11); // 1 header + 10 samples
}

#[tokio::test]
async fn test_csv_validation() {
    let tool = CsvAnalyzerTool::default();

    let csv_content = r#"id,email,age
1,john@example.com,25
2,invalid-email,30
3,mary@test.org,150
4,bob@demo.net,-5
"#;

    let input = AgentInput::text("validate csv data").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "validate",
            "input": csv_content,
            "options": {
                "rules": {
                    "email": "email",
                    "age": "range:0-120"
                }
            }
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(result.text.contains("valid"));

    let extra = &result.metadata.extra;
    if let Some(validation) = extra.get("validation_result") {
        assert!(validation["errors"].is_array());
        let errors = validation["errors"].as_array().unwrap();
        assert!(!errors.is_empty()); // Should have validation errors
    }
}

#[tokio::test]
async fn test_csv_transform_operation() {
    let tool = CsvAnalyzerTool::default();

    let csv_content = r#"product,price,quantity
Apple,1.50,100
Banana,0.75,200
Orange,2.00,150
"#;

    let input = AgentInput::text("transform csv").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "transform",
            "input": csv_content,
            "options": {
                "add_columns": {
                    "total": "price * quantity"
                },
                "rename_columns": {
                    "product": "item_name",
                    "price": "unit_price"
                }
            }
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    // Check transformed headers
    assert!(result.text.contains("item_name"));
    assert!(result.text.contains("unit_price"));
    assert!(result.text.contains("total"));
    assert!(!result.text.contains("product,")); // Old name should be gone
}

#[tokio::test]
async fn test_encoding_detection() {
    let tool = CsvAnalyzerTool::default();

    // UTF-8 with BOM and various characters
    let csv_with_special_chars =
        "\u{FEFF}id,name,description\n1,Café,Coffee shop\n2,Naïve,Simple\n3,Москва,Moscow\n";

    let input = AgentInput::text("analyze encoded csv").with_parameter(
        "parameters".to_string(),
        json!({
            "operation": "analyze",
            "input": csv_with_special_chars
        }),
    );

    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    assert!(result.text.contains("encoding"));

    let extra = &result.metadata.extra;
    let analysis = extra.get("analysis_result").unwrap();
    // Should detect UTF-8 encoding
    assert!(analysis["encoding"].as_str().unwrap().contains("UTF"));
}
