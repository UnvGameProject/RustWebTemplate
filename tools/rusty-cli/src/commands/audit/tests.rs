use std::path::Path;

use super::{
    DYNAMIC_SQL, PUBLIC_INPUT_AUDITED, PUBLIC_INPUT_UNRECOGNIZED, SHELL_EXECUTION, Severity,
    scan_source,
};

fn scan(source: &str) -> Vec<super::Finding> {
    scan_source(Path::new("src/example.rs"), source).expect("test source should parse")
}

#[test]
fn accepts_static_parameterized_sql() {
    let findings = scan(
        r#"
        async fn example(pool: &sqlx::PgPool, id: i64) {
            let _ = sqlx::query(
                "SELECT * FROM contacts WHERE id = $1"
            )
            .bind(id)
            .fetch_optional(pool)
            .await;
        }
        "#,
    );

    assert!(findings.is_empty());
}

#[test]
fn accepts_sqlx_compile_time_query_macro() {
    let findings = scan(
        r#"
        async fn example(pool: &sqlx::PgPool, id: i64) {
            let _ = sqlx::query!(
                "SELECT id FROM contacts WHERE id = $1",
                id
            )
            .fetch_optional(pool)
            .await;
        }
        "#,
    );

    assert!(findings.is_empty());
}

#[test]
fn detects_format_based_sql_query() {
    let findings = scan(
        r#"
        fn example(id: i64) {
            let _ = sqlx::query(
                &format!("SELECT * FROM contacts WHERE id = {id}")
            );
        }
        "#,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
    assert_eq!(findings[0].rule_id, DYNAMIC_SQL);
}

#[test]
fn detects_format_based_sql_query_as() {
    let findings = scan(
        r#"
        fn example(id: i64) {
            let _ = sqlx::query_as::<_, Contact>(
                &format!("SELECT * FROM contacts WHERE id = {id}")
            );
        }
        "#,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, DYNAMIC_SQL);
}

#[test]
fn detects_unix_shell_interpreter() {
    let findings = scan(
        r#"
        fn example() {
            let _ = std::process::Command::new("/bin/bash");
        }
        "#,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
    assert_eq!(findings[0].rule_id, SHELL_EXECUTION);
}

#[test]
fn detects_windows_shell_interpreter() {
    let findings = scan(
        r#"
        use std::process::Command;

        fn example() {
            let _ = Command::new("powershell.exe");
        }
        "#,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, SHELL_EXECUTION);
}

#[test]
fn accepts_fixed_non_shell_executable() {
    let findings = scan(
        r#"
        use std::process::Command;

        fn example() {
            let _ = Command::new("git")
                .arg("status")
                .status();
        }
        "#,
    );

    assert!(findings.is_empty());
}

#[test]
fn reports_source_location() {
    let findings = scan(
        r#"
fn example() {
    let _ = std::process::Command::new("sh");
}
"#,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].line, 3);
    assert!(findings[0].column > 0);
}

#[test]
fn recognizes_validated_domain_input_in_procedure() {
    let findings = scan(
        r#"
        #[procedure]
        async fn create_contact(
            cx: &Cx,
            name: String,
            email: String,
        ) {
            let input =
                CreateContactInput::from_untrusted(name, email);

            let input = match input.validate() {
                Ok(input) => input,
                Err(_) => return,
            };

            persist(cx, &input).await;
        }
        "#,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Info);
    assert_eq!(findings[0].rule_id, PUBLIC_INPUT_AUDITED);
    assert!(
        findings[0]
            .detail
            .contains("name: String -> validated domain input")
    );
    assert!(
        findings[0]
            .detail
            .contains("email: String -> validated domain input")
    );
}

#[test]
fn recognizes_uuid_parse_boundary_in_procedure() {
    let findings = scan(
        r#"
        #[procedure]
        async fn delete_contact(
            cx: &Cx,
            id: String,
        ) {
            let id = match Uuid::parse_str(&id) {
                Ok(id) => id,
                Err(_) => return,
            };

            delete(cx, id).await;
        }
        "#,
    );

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Info);
    assert_eq!(findings[0].rule_id, PUBLIC_INPUT_AUDITED);
    assert!(findings[0].detail.contains("id: String -> UUID parser"));
}

#[test]
fn warns_for_unrecognized_public_string_input() {
    let findings = scan(
        r#"
        #[procedure]
        async fn unsafe_create(
            cx: &Cx,
            name: String,
        ) {
            persist(cx, name).await;
        }
        "#,
    );

    assert_eq!(findings.len(), 2);

    assert!(findings.iter().any(|finding| {
        finding.rule_id == PUBLIC_INPUT_AUDITED && finding.severity == Severity::Info
    }));

    assert!(findings.iter().any(|finding| {
        finding.rule_id == PUBLIC_INPUT_UNRECOGNIZED && finding.severity == Severity::Warning
    }));
}

#[test]
fn from_untrusted_without_validation_is_not_enough() {
    let findings = scan(
        r#"
        #[procedure]
        async fn unsafe_create(
            name: String,
        ) {
            let input =
                CreateContactInput::from_untrusted(name);

            persist(input).await;
        }
        "#,
    );

    assert!(
        findings
            .iter()
            .any(|finding| { finding.rule_id == PUBLIC_INPUT_UNRECOGNIZED })
    );
}

#[test]
fn non_procedure_string_is_not_treated_as_public_input() {
    let findings = scan(
        r#"
        async fn internal_helper(name: String) {
            persist(name).await;
        }
        "#,
    );

    assert!(findings.is_empty());
}
