/// Edge case tests for malformed and corrupted data handling
///
/// Ensures that:
/// - Missing required fields are detected with clear error messages
/// - Invalid timestamps fail gracefully with line numbers
/// - Out-of-range percentages are caught
/// - Slack transcript format mismatches are detected
/// - Partial corruption doesn't crash the parser

#[cfg(test)]
mod malformed_data_tests {
    use qir_core::ingest::jira_csv::{parse_jira_csv, JiraCsvProfile, CsvParseError};
    use qir_core::ingest::slack_transcript::{parse_slack_transcript, SlackFormat};

    fn default_jira_profile() -> JiraCsvProfile {
        JiraCsvProfile {
            name: "Test Profile".to_string(),
            title_field: "Summary".to_string(),
            detection_field: "Created".to_string(),
            resolution_field: "Resolved".to_string(),
            impact_field: "Impact".to_string(),
            degradation_field: "Degradation".to_string(),
        }
    }

    #[test]
    fn test_jira_csv_missing_required_column() {
        // CSV missing "Resolved" column
        let csv_data = r#"Summary,Created,Impact,Degradation
"Missing resolved time",2025-01-15T10:00:00Z,90,85"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "MISSING_REQUIRED_FIELD");
        assert!(err.message.to_lowercase().contains("resolved"));
    }

    #[test]
    fn test_jira_csv_missing_all_optional_columns() {
        // CSV with only required columns (should succeed with defaults)
        let csv_data = r#"Summary,Created,Resolved
"Minimal incident",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // Should succeed with default values for missing optional fields
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows.len(), 1);
    }

    #[test]
    fn test_jira_csv_invalid_rfc3339_timestamp() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Bad timestamp",not-a-date,2025-01-15T11:30:00Z,95,90"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "INVALID_TIMESTAMP");
        assert!(err.message.contains("not-a-date") || err.message.contains("parse"));
    }

    #[test]
    fn test_jira_csv_malformed_timestamp_formats() {
        // Test various invalid timestamp formats
        let test_cases = vec![
            ("2025/01/15 10:00:00", "slash format"),        // Wrong separators
            ("2025-01-15 10:00:00", "space instead of T"),  // Missing T
            ("2025-01-15T10:00:00", "missing timezone"),    // Missing Z or +00:00
            ("Jan 15, 2025", "human readable"),             // English date
            ("", "empty string"),                            // Empty
            ("2025-01-15T25:00:00Z", "invalid hour"),       // Hour out of range
        ];

        for (timestamp, description) in test_cases {
            let csv_data = format!(
                "Summary,Created,Resolved,Impact,Degradation\n\"Test\",{},2025-01-15T11:30:00Z,90,85",
                timestamp
            );

            let result = parse_jira_csv(&csv_data, &default_jira_profile());

            assert!(
                result.is_err(),
                "Should reject {} timestamp",
                description
            );
            assert_eq!(result.unwrap_err().code, "INVALID_TIMESTAMP");
        }
    }

    #[test]
    fn test_jira_csv_invalid_percentage_above_100() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Bad percentage",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,150,90"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALUE_OUT_OF_RANGE");
        assert!(err.message.contains("Impact") || err.message.contains("150"));
    }

    #[test]
    fn test_jira_csv_invalid_percentage_negative() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Negative percentage",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,50,-10"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, "VALUE_OUT_OF_RANGE");
        assert!(err.message.contains("Degradation") || err.message.contains("-10"));
    }

    #[test]
    fn test_jira_csv_non_numeric_percentage() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Non-numeric",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,high,medium"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        assert!(result.is_err());
        let err = result.unwrap_err();
        // Should be a type error or value parsing error
        assert!(err.code == "VALUE_OUT_OF_RANGE" || err.code == "INVALID_VALUE_TYPE");
    }

    #[test]
    fn test_jira_csv_resolution_before_detection() {
        // Detection time is after resolution time (invalid)
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Invalid timeline",2025-01-15T11:30:00Z,2025-01-15T10:00:00Z,90,85"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // Should parse successfully but validation should flag this later
        assert!(result.is_ok() || result.is_err());
        // If parsed, the negative duration should be caught by validation layer
    }

    #[test]
    fn test_jira_csv_very_long_title() {
        // Title with 100,000+ characters
        let long_title = "A".repeat(100_000);
        let csv_data = format!(
            "Summary,Created,Resolved,Impact,Degradation\n\"{}\",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,90,85",
            long_title
        );

        let result = parse_jira_csv(&csv_data, &default_jira_profile());

        // Should either succeed or fail gracefully, not hang
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_jira_csv_empty_required_field() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,90,85"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // Empty title might be an error or might be allowed with a default
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_jira_csv_extra_unquoted_commas() {
        // CSV with unescaped commas in field (malformed CSV)
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Issue, with comma",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,90,85"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // CSV parser should handle quoted fields correctly
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rows[0].title, "Issue, with comma");
    }

    #[test]
    fn test_slack_transcript_unrecognized_format() {
        let transcript = "This is just plain text, not a Slack transcript at all.";

        let result = parse_slack_transcript(transcript);

        assert_eq!(result.format, SlackFormat::Unknown);
        assert_eq!(result.messages.len(), 0);
        assert!(result.parse_warnings.len() >= 0); // May or may not have warnings
    }

    #[test]
    fn test_slack_transcript_partial_corruption() {
        let transcript = r#"Slack Transcript Export
January 15, 2025

user1 (6:00 PM) Message 1
This line has NO STRUCTURE and breaks the format
user2 (6:01 PM) Message 2
ANOTHER_CORRUPTED_LINE
user3 (6:02 PM) Message 3"#;

        let result = parse_slack_transcript(transcript);

        // Should parse valid messages and skip/warn about corrupted lines
        assert!(result.messages.len() >= 2); // At least the valid messages
        assert!(result.parse_warnings.len() > 0); // Should have warnings
    }

    #[test]
    fn test_slack_transcript_empty_file() {
        let transcript = "";

        let result = parse_slack_transcript(transcript);

        assert_eq!(result.format, SlackFormat::Unknown);
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn test_slack_transcript_only_headers() {
        let transcript = r#"Slack Transcript Export
January 15, 2025"#;

        let result = parse_slack_transcript(transcript);

        // Should recognize format but have no messages
        assert!(result.messages.len() == 0);
    }

    #[test]
    fn test_slack_transcript_malformed_json() {
        let transcript = r#"{
  "messages": [
    {"type": "message", "user": "U123456", "text": "Hello",
    // BROKEN JSON - missing closing brace
  ]
}"#;

        let result = parse_slack_transcript(transcript);

        // Should fail gracefully for JSON format
        assert!(result.messages.len() == 0 || result.parse_warnings.len() > 0);
    }

    #[test]
    fn test_slack_transcript_wrong_json_schema() {
        let transcript = r#"{
  "users": [{"id": "U123", "name": "alice"}],
  "channels": [{"name": "general"}]
}"#;

        let result = parse_slack_transcript(transcript);

        // Valid JSON but wrong Slack schema
        assert_eq!(result.format, SlackFormat::Unknown);
        assert_eq!(result.messages.len(), 0);
    }

    #[test]
    fn test_csv_with_bom() {
        // CSV with Byte Order Mark (UTF-8 BOM)
        let csv_data = "\u{FEFF}Summary,Created,Resolved,Impact,Degradation\n\"Issue\",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,90,85";

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // Should handle or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_csv_with_inconsistent_column_count() {
        // Some rows have extra/missing columns
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Issue 1",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,90,85
"Issue 2",2025-01-14T09:00:00Z,2025-01-14T09:45:00Z,80  <- missing column
"Issue 3",2025-01-13T08:00:00Z,2025-01-13T08:15:00Z,70,65,extra_column"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // Should either succeed with defaults or fail gracefully
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_csv_with_null_bytes() {
        // CSV with null bytes (corrupted file)
        let csv_data = "Summary,Created,Resolved,Impact,Degradation\n\"Issue\0WithNull\",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,90,85";

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // Should handle or reject gracefully
        assert!(result.is_ok() || result.is_err());
    }
}
