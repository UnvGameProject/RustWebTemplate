use std::path::Path;

use super::{DYNAMIC_SQL, SHELL_EXECUTION, Severity, scan_source};

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
