use std::{
    collections::{HashMap, HashSet},
    fs, io,
    path::{Path, PathBuf},
};

use proc_macro2::Span;
use syn::{
    Attribute, Expr, ExprCall, ExprMethodCall, FnArg, ItemFn, Lit, Local, Pat, PathArguments, Type,
    spanned::Spanned,
    visit::{self, Visit},
};

use crate::{commands::CommandResult, project::Project};

const SHELL_EXECUTION: &str = "RUSTY-AUDIT-001";
const DYNAMIC_SQL: &str = "RUSTY-AUDIT-002";
const PUBLIC_INPUT_AUDITED: &str = "RUSTY-AUDIT-010";
const PUBLIC_INPUT_UNRECOGNIZED: &str = "RUSTY-AUDIT-011";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProtectionBoundary {
    ValidatedDomainInput,
    UuidParse,
}

impl ProtectionBoundary {
    fn label(self) -> &'static str {
        match self {
            Self::ValidatedDomainInput => "validated domain input",
            Self::UuidParse => "UUID parser",
        }
    }
}

struct ProcedureInput {
    name: String,
    span: Span,
    boundary: Option<ProtectionBoundary>,
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

    fn audit_procedure(&mut self, function: &ItemFn) {
        let Some(inputs) = analyze_procedure(function) else {
            return;
        };

        if inputs.is_empty() {
            return;
        }

        let summary = inputs
            .iter()
            .map(|input| {
                let status = input
                    .boundary
                    .map(ProtectionBoundary::label)
                    .unwrap_or("unrecognized");

                format!("{}: String -> {status}", input.name)
            })
            .collect::<Vec<_>>()
            .join(", ");

        self.report(
            Severity::Info,
            PUBLIC_INPUT_AUDITED,
            function.sig.ident.span(),
            "public procedure input boundaries audited",
            format!("procedure `{}`: {summary}", function.sig.ident),
        );

        for input in inputs {
            if input.boundary.is_some() {
                continue;
            }

            self.report(
                Severity::Warning,
                PUBLIC_INPUT_UNRECOGNIZED,
                input.span,
                "public String input has no recognized protection boundary",
                format!(
                    "procedure `{}` input `{}` is externally supplied; \
                     Rusty did not recognize a validated domain input or \
                     approved parser boundary",
                    function.sig.ident, input.name,
                ),
            );
        }
    }
}

impl<'ast> Visit<'ast> for AuditVisitor<'_> {
    fn visit_item_fn(&mut self, node: &'ast ItemFn) {
        self.audit_procedure(node);
        visit::visit_item_fn(self, node);
    }

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

fn analyze_procedure(function: &ItemFn) -> Option<Vec<ProcedureInput>> {
    if !has_procedure_attribute(&function.attrs) {
        return None;
    }

    let mut inputs = procedure_string_inputs(function);

    if inputs.is_empty() {
        return Some(inputs);
    }

    let external_inputs = inputs.iter().map(|input| input.name.clone()).collect();

    let mut flow = ProcedureFlowVisitor {
        external_inputs,
        from_untrusted_bindings: HashMap::new(),
        validated_bindings: HashSet::new(),
        uuid_parsed_inputs: HashSet::new(),
    };

    flow.visit_block(&function.block);

    for input in &mut inputs {
        input.boundary = flow.boundary_for(&input.name);
    }

    Some(inputs)
}

fn has_procedure_attribute(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "procedure")
    })
}

fn procedure_string_inputs(function: &ItemFn) -> Vec<ProcedureInput> {
    function
        .sig
        .inputs
        .iter()
        .filter_map(|argument| {
            let FnArg::Typed(argument) = argument else {
                return None;
            };

            if !is_string_type(&argument.ty) {
                return None;
            }

            let Pat::Ident(pattern) = argument.pat.as_ref() else {
                return None;
            };

            Some(ProcedureInput {
                name: pattern.ident.to_string(),
                span: argument.span(),
                boundary: None,
            })
        })
        .collect()
}

fn is_string_type(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    if type_path.qself.is_some() {
        return false;
    }

    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };

    segment.ident == "String" && matches!(segment.arguments, PathArguments::None)
}

struct ProcedureFlowVisitor {
    external_inputs: HashSet<String>,
    from_untrusted_bindings: HashMap<String, HashSet<String>>,
    validated_bindings: HashSet<String>,
    uuid_parsed_inputs: HashSet<String>,
}

impl ProcedureFlowVisitor {
    fn boundary_for(&self, input: &str) -> Option<ProtectionBoundary> {
        let validated = self
            .from_untrusted_bindings
            .iter()
            .any(|(binding, sources)| {
                self.validated_bindings.contains(binding) && sources.contains(input)
            });

        if validated {
            return Some(ProtectionBoundary::ValidatedDomainInput);
        }

        if self.uuid_parsed_inputs.contains(input) {
            return Some(ProtectionBoundary::UuidParse);
        }

        None
    }
}

impl<'ast> Visit<'ast> for ProcedureFlowVisitor {
    fn visit_local(&mut self, node: &'ast Local) {
        let Some(binding) = local_binding_name(&node.pat) else {
            visit::visit_local(self, node);
            return;
        };

        if let Some(init) = &node.init {
            if let Some(sources) = from_untrusted_sources(&init.expr, &self.external_inputs) {
                self.from_untrusted_bindings
                    .insert(binding.clone(), sources.into_iter().collect());
            }

            if self.external_inputs.contains(&binding)
                && expression_contains_uuid_parse(&init.expr, &binding)
            {
                self.uuid_parsed_inputs.insert(binding);
            }
        }

        visit::visit_local(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if node.method == "validate" {
            if let Some(receiver) = expression_identifier(&node.receiver) {
                self.validated_bindings.insert(receiver);
            }
        }

        visit::visit_expr_method_call(self, node);
    }
}

fn local_binding_name(pattern: &Pat) -> Option<String> {
    let Pat::Ident(pattern) = pattern else {
        return None;
    };

    Some(pattern.ident.to_string())
}

fn from_untrusted_sources(
    expression: &Expr,
    external_inputs: &HashSet<String>,
) -> Option<Vec<String>> {
    let Expr::Call(call) = peel_expr(expression) else {
        return None;
    };

    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let last = function.path.segments.last()?;

    if last.ident != "from_untrusted" {
        return None;
    }

    Some(
        call.args
            .iter()
            .filter_map(expression_identifier)
            .filter(|name| external_inputs.contains(name))
            .collect(),
    )
}

fn expression_identifier(expression: &Expr) -> Option<String> {
    let Expr::Path(path) = peel_expr(expression) else {
        return None;
    };

    if path.qself.is_some() || path.path.segments.len() != 1 {
        return None;
    }

    Some(path.path.segments.first()?.ident.to_string())
}

fn expression_contains_uuid_parse(expression: &Expr, target: &str) -> bool {
    let mut visitor = UuidParseVisitor {
        target,
        found: false,
    };

    visitor.visit_expr(expression);

    visitor.found
}

struct UuidParseVisitor<'a> {
    target: &'a str,
    found: bool,
}

impl<'ast> Visit<'ast> for UuidParseVisitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if uuid_parse_source(node)
            .as_deref()
            .is_some_and(|source| source == self.target)
        {
            self.found = true;
            return;
        }

        visit::visit_expr_call(self, node);
    }
}

fn uuid_parse_source(call: &ExprCall) -> Option<String> {
    let Expr::Path(function) = call.func.as_ref() else {
        return None;
    };

    let segments: Vec<_> = function.path.segments.iter().collect();

    if segments.len() < 2 {
        return None;
    }

    let owner = &segments[segments.len() - 2].ident;
    let method = &segments[segments.len() - 1].ident;

    if owner != "Uuid" || method != "parse_str" {
        return None;
    }

    expression_identifier(call.args.first()?)
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
