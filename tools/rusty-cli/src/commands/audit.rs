use std::{
    fs, io,
    path::{Path, PathBuf},
};

use proc_macro2::Span;
use syn::{
    Expr, ExprCall, Lit,
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::{commands::CommandResult, project::Project};

const SHELL_EXECUTION: &str = "RUSTY-AUDIT-001";
const DYNAMIC_SQL: &str = "RUSTY-AUDIT-002";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARN",
            Self::Info => "INFO",
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Finding {
    severity: Severity,
    rule_id: &'static str,
    path: PathBuf,
    line: usize,
    column: usize,
    message: &'static str,
    detail: String,
}

pub(crate) fn run() -> CommandResult {
    let project = Project::discover()?;
    let src = project.root().join("src");

    let mut rust_files = Vec::new();
    collect_rust_files(&src, &mut rust_files)?;

    rust_files.sort();

    let mut findings = Vec::new();

    for path in &rust_files {
        let source = fs::read_to_string(path)?;

        let relative = path.strip_prefix(project.root()).unwrap_or(path);

        findings.extend(scan_source(relative, &source).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("could not parse {}: {error}", relative.display()),
            )
        })?);
    }

    findings.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.rule_id.cmp(right.rule_id))
    });

    println!("Rusty audit");
    println!();
    println!("scanned : {} Rust files", rust_files.len());

    if findings.is_empty() {
        println!("result  : clean");
        return Ok(());
    }

    println!("findings: {}", findings.len());
    println!();

    for finding in &findings {
        println!(
            "{} [{}] {}:{}:{}",
            finding.severity.label(),
            finding.rule_id,
            finding.path.display(),
            finding.line,
            finding.column,
        );
        println!("  {}", finding.message);
        println!("  {}", finding.detail);
        println!();
    }

    let error_count = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .count();

    let warning_count = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Warning)
        .count();

    let info_count = findings
        .iter()
        .filter(|finding| finding.severity == Severity::Info)
        .count();

    println!(
        "summary : {error_count} error(s), \
         {warning_count} warning(s), \
         {info_count} info"
    );

    if error_count > 0 {
        return Err(io::Error::other(format!("audit failed with {error_count} error(s)")).into());
    }

    Ok(())
}

fn collect_rust_files(directory: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_rust_files(&path, files)?;
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }

    Ok(())
}

fn scan_source(path: &Path, source: &str) -> syn::Result<Vec<Finding>> {
    let file = syn::parse_file(source)?;

    let mut visitor = AuditVisitor {
        path,
        findings: Vec::new(),
    };

    visitor.visit_file(&file);

    Ok(visitor.findings)
}

struct AuditVisitor<'a> {
    path: &'a Path,
    findings: Vec<Finding>,
}

impl AuditVisitor<'_> {
    fn report(
        &mut self,
        severity: Severity,
        rule_id: &'static str,
        span: Span,
        message: &'static str,
        detail: String,
    ) {
        let start = span.start();

        self.findings.push(Finding {
            severity,
            rule_id,
            path: self.path.to_path_buf(),
            line: start.line,
            column: start.column + 1,
            message,
            detail,
        });
    }
}

impl<'ast> Visit<'ast> for AuditVisitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Some(shell) = shell_interpreter(node) {
            self.report(
                Severity::Error,
                SHELL_EXECUTION,
                node.span(),
                "shell interpreter execution detected",
                format!(
                    "Command::new({shell:?}) violates the application \
                     policy against shell interpretation"
                ),
            );
        }

        if let Some(function) = dynamic_sql_function(node) {
            self.report(
                Severity::Error,
                DYNAMIC_SQL,
                node.span(),
                "dynamic SQL construction detected",
                format!(
                    "{function} receives format!(...) output; \
                     use static SQL with bound parameters instead"
                ),
            );
        }

        visit::visit_expr_call(self, node);
    }
}

fn shell_interpreter(call: &ExprCall) -> Option<String> {
    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let segments: Vec<_> = function.path.segments.iter().collect();

    if segments.len() < 2 {
        return None;
    }

    let command = segments[segments.len() - 2].ident.to_string();
    let method = segments[segments.len() - 1].ident.to_string();

    if command != "Command" || method != "new" {
        return None;
    }

    let argument = call.args.first()?;

    let Expr::Lit(literal) = peel_expr(argument) else {
        return None;
    };

    let Lit::Str(value) = &literal.lit else {
        return None;
    };

    let executable = value.value();

    if is_shell_interpreter(&executable) {
        Some(executable)
    } else {
        None
    }
}

fn is_shell_interpreter(executable: &str) -> bool {
    let executable = executable.to_ascii_lowercase();

    let basename = executable.rsplit(['/', '\\']).next().unwrap_or(&executable);

    matches!(
        basename,
        "sh" | "bash"
            | "dash"
            | "zsh"
            | "cmd"
            | "cmd.exe"
            | "powershell"
            | "powershell.exe"
            | "pwsh"
            | "pwsh.exe"
    )
}

fn dynamic_sql_function(call: &ExprCall) -> Option<String> {
    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let mut segments = function.path.segments.iter();

    let first = segments.next()?;

    if first.ident != "sqlx" {
        return None;
    }

    let last = function.path.segments.last()?;

    let function_name = last.ident.to_string();

    let recognized = matches!(
        function_name.as_str(),
        "query"
            | "query_as"
            | "query_scalar"
            | "query_with"
            | "query_as_with"
            | "query_scalar_with"
            | "raw_sql"
    );

    if !recognized {
        return None;
    }

    let first_argument = call.args.first()?;

    if is_format_macro(first_argument) {
        Some(format!("sqlx::{function_name}"))
    } else {
        None
    }
}

fn is_format_macro(expression: &Expr) -> bool {
    let expression = peel_expr(expression);

    let Expr::Macro(expression) = expression else {
        return false;
    };

    expression
        .mac
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "format")
}

fn peel_expr(mut expression: &Expr) -> &Expr {
    loop {
        expression = match expression {
            Expr::Reference(reference) => &reference.expr,
            Expr::Paren(paren) => &paren.expr,
            Expr::Group(group) => &group.expr,
            _ => return expression,
        };
    }
}

#[cfg(test)]
mod tests;
