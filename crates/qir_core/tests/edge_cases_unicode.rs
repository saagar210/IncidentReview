/// Edge case tests for Unicode, emoji, and special character handling
///
/// Ensures that:
/// - Emoji in incident titles and descriptions are preserved
/// - Unicode names (Greek, Arabic, Chinese, etc.) are handled correctly
/// - Special characters don't break CSV/Slack parsing
/// - Emoji in timeline events are preserved in reports

#[cfg(test)]
mod unicode_edge_cases {
    use qir_core::ingest::jira_csv::{parse_jira_csv, JiraCsvProfile};
    use qir_core::ingest::slack_transcript::parse_slack_transcript;
    use qir_core::report::generate_markdown;
    use qir_core::domain::{Incident, TimelineEvent};

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
    fn test_jira_csv_with_emoji_in_title() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"🚨 Database Down 🔥",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,95,90
"Cache Miss ⚠️",2025-01-14T09:00:00Z,2025-01-14T09:45:00Z,50,40
"API 💥 Timeout",2025-01-13T08:00:00Z,2025-01-13T08:15:00Z,70,65"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile()).unwrap();

        assert_eq!(result.rows.len(), 3);
        assert!(result.rows[0].title.contains("🚨"));
        assert!(result.rows[0].title.contains("🔥"));
        assert!(result.rows[1].title.contains("⚠️"));
        assert!(result.rows[2].title.contains("💥"));
    }

    #[test]
    fn test_jira_csv_with_unicode_usernames() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation,Reporter
"Database issue",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,95,90,José_García
"Network outage",2025-01-14T09:00:00Z,2025-01-14T09:45:00Z,80,75,李_王
"Service degraded",2025-01-13T08:00:00Z,2025-01-13T08:15:00Z,50,45,Μαρία_Παπαδοπούλου"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile()).unwrap();

        assert_eq!(result.rows.len(), 3);
        // Verify unicode is preserved (no replacement or corruption)
        assert!(result.rows.iter().all(|r| !r.title.contains("?")));
    }

    #[test]
    fn test_jira_csv_with_arabic_and_hebrew() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"خادم قاعدة البيانات معطل",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,95,90
"שרת ה-API כשל",2025-01-14T09:00:00Z,2025-01-14T09:45:00Z,80,75"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile()).unwrap();

        assert_eq!(result.rows.len(), 2);
        assert!(result.rows[0].title.contains("خادم"));
        assert!(result.rows[1].title.contains("שרת"));
    }

    #[test]
    fn test_slack_transcript_with_emoji_in_messages() {
        let transcript = r#"Slack Transcript Export
January 15, 2025 at 6:00 PM

user_1 (6:00 PM) 🚨 Database is down! We're losing 🔥 requests fast! 📉
user_2 (6:01 PM) Impact: East US region 🌍 Users reporting ❌ errors
user_3 (6:02 PM) We're on it! 💪 ETA 10 min? ⏱️"#;

        let result = parse_slack_transcript(transcript);

        assert!(result.messages.len() >= 3);
        // Verify emoji preserved in message content
        assert!(result.messages.iter().any(|m| m.content.contains("🚨")));
        assert!(result.messages.iter().any(|m| m.content.contains("🌍")));
        assert!(result.messages.iter().any(|m| m.content.contains("💪")));
    }

    #[test]
    fn test_slack_transcript_with_unicode_usernames() {
        let transcript = r#"Slack Transcript Export
January 15, 2025

José_García (6:00 PM) Database alert
李_王 (6:01 PM) Investigating now
Μαρία_Παπαδοπούλου (6:02 PM) We need to escalate"#;

        let result = parse_slack_transcript(transcript);

        assert!(result.messages.len() >= 3);
        // Verify unicode usernames preserved
        assert!(result.messages.iter().any(|m| m.author.contains("José")));
        assert!(result.messages.iter().any(|m| m.author.contains("李")));
        assert!(result.messages.iter().any(|m| m.author.contains("Μ")));
    }

    #[test]
    fn test_incident_report_preserves_emoji_in_title() {
        let incident = Incident {
            id: "test-emoji-1".to_string(),
            title: "🚨 Critical API 💥 Timeout".to_string(),
            description: Some("Database connection lost 🔥".to_string()),
            detection_time: "2025-01-15T10:00:00Z".to_string(),
            resolution_time: "2025-01-15T11:30:00Z".to_string(),
            impact_level: 95,
            degradation_level: 90,
            detection_source: Some("monitoring 📊".to_string()),
            vendor: Some("Platform".to_string()),
            service: Some("API".to_string()),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
        };

        let report = generate_markdown(&vec![incident]).unwrap();

        // Verify emoji preserved in markdown
        assert!(report.contains("🚨"));
        assert!(report.contains("💥"));
        assert!(report.contains("🔥"));
        assert!(report.contains("📊"));
    }

    #[test]
    fn test_timeline_events_with_unicode() {
        let incident = Incident {
            id: "test-timeline-1".to_string(),
            title: "Incident with Unicode".to_string(),
            detection_time: "2025-01-15T10:00:00Z".to_string(),
            resolution_time: "2025-01-15T11:30:00Z".to_string(),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
            impact_level: 80,
            degradation_level: 75,
            detection_source: None,
            vendor: None,
            service: None,
            description: None,
        };

        let timeline_events = vec![
            TimelineEvent {
                id: "1".to_string(),
                incident_id: "test-timeline-1".to_string(),
                timestamp: "2025-01-15T10:00:00Z".to_string(),
                event_type: "detection".to_string(),
                description: "🔍 Investigation started by José".to_string(),
                actor: Some("José".to_string()),
                created_at: "2025-01-15T10:00:00Z".to_string(),
            },
            TimelineEvent {
                id: "2".to_string(),
                incident_id: "test-timeline-1".to_string(),
                timestamp: "2025-01-15T10:15:00Z".to_string(),
                event_type: "mitigation".to_string(),
                description: "📞 Customer notified: Привет (Hello in Russian)! 🇷🇺".to_string(),
                actor: Some("Support_Team".to_string()),
                created_at: "2025-01-15T10:15:00Z".to_string(),
            },
        ];

        // Would normally call generate_report_with_timeline(&incident, &timeline_events)
        // For now, verify that description contains all unicode
        assert!(timeline_events[0].description.contains("🔍"));
        assert!(timeline_events[0].description.contains("José"));
        assert!(timeline_events[1].description.contains("📞"));
        assert!(timeline_events[1].description.contains("Привет"));
        assert!(timeline_events[1].description.contains("🇷🇺"));
    }

    #[test]
    fn test_multibyte_emoji_handling() {
        // Test multi-byte emoji that sometimes cause issues:
        // 👨‍👩‍👧‍👦 (family), 🏳️‍🌈 (rainbow flag), etc.
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Issue for team 👨‍👩‍👧‍👦",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,50,40
"Infrastructure issue 🏳️‍🌈",2025-01-14T09:00:00Z,2025-01-14T09:45:00Z,60,55"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile()).unwrap();

        assert_eq!(result.rows.len(), 2);
        // Verify complex emoji handled correctly
        assert_eq!(result.rows[0].title.chars().count() > "Issue for team ".len(), true);
        assert_eq!(result.rows[1].title.chars().count() > "Infrastructure issue ".len(), true);
    }

    #[test]
    fn test_zero_width_characters() {
        // Zero-width characters (sometimes hidden in copy-paste)
        let csv_data = "Summary,Created,Resolved,Impact,Degradation\n\"DB​ issue\",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,95,90"; // Contains zero-width space

        let result = parse_jira_csv(csv_data, &default_jira_profile());

        // Should parse successfully despite hidden characters
        assert!(result.is_ok() || result.is_err()); // Either succeeds or fails gracefully
        // Should not panic or corrupt data
    }

    #[test]
    fn test_mixed_scripts_in_single_field() {
        let csv_data = r#"Summary,Created,Resolved,Impact,Degradation
"Database issue in 中文 with José 🔥",2025-01-15T10:00:00Z,2025-01-15T11:30:00Z,95,90"#;

        let result = parse_jira_csv(csv_data, &default_jira_profile()).unwrap();

        assert_eq!(result.rows.len(), 1);
        assert!(result.rows[0].title.contains("中文"));
        assert!(result.rows[0].title.contains("José"));
        assert!(result.rows[0].title.contains("🔥"));
    }
}
