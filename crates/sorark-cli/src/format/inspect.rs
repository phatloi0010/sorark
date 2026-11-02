//! Formatter for `sorark inspect` WebAssembly SEP-0048 contract specification reports.

use super::markdown::render_diagnostics_markdown;
use super::text::render_diagnostics_text;
use sorark_ir::{EnumSpec, FunctionSpec, StructSpec, TupleStructSpec, UnionSpec};
use sorark_wasm::WasmInspectionReport;
use std::fmt::Write as _;

/// Renders a human-readable summary of a [`WasmInspectionReport`].
#[must_use]
pub(super) fn render_wasm_inspection_text(report: &WasmInspectionReport) -> String {
    let mut out = String::new();
    append_header(&mut out, report);
    append_structs(&mut out, &report.module.structs);
    append_tuple_structs(&mut out, &report.module.tuple_structs);
    append_unions(&mut out, &report.module.unions);
    append_enums(&mut out, &report.module.enums);
    append_functions(&mut out, &report.module.functions);

    if !report.diagnostics.is_empty() {
        out.push('\n');
        out.push_str(&render_diagnostics_text(&report.diagnostics));
    }
    out
}

/// Renders a [`WasmInspectionReport`] as clean GitHub-Flavored Markdown tables.
#[must_use]
pub(super) fn render_wasm_inspection_markdown(report: &WasmInspectionReport) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# SORARK WASM Contract Inspection: `{}`\n",
        report.module.name
    );

    append_markdown_metrics(&mut out, report);
    append_markdown_structs(&mut out, &report.module.structs);
    append_markdown_tuple_structs(&mut out, &report.module.tuple_structs);
    append_markdown_unions(&mut out, &report.module.unions);
    append_markdown_enums(&mut out, &report.module.enums);
    append_markdown_functions(&mut out, &report.module.functions);

    if !report.diagnostics.is_empty() {
        out.push('\n');
        out.push_str(&render_diagnostics_markdown(&report.diagnostics));
    }

    out
}

/// Appends the WASM binary size, protocol version, and SEP-0048 custom section summary header.
fn append_header(out: &mut String, report: &WasmInspectionReport) {
    let _ = writeln!(
        out,
        "=== SORARK WASM Contract Inspection: {} ===",
        report.module.name
    );
    let _ = writeln!(
        out,
        "Binary Size:          {} bytes ({:.2} KiB / 64.00 KiB limit)",
        report.metrics.byte_size,
        report.metrics.byte_size as f64 / 1024.0
    );
    let _ = writeln!(
        out,
        "Soroban Protocol:     {}",
        report
            .metrics
            .protocol_version
            .map_or_else(|| "missing".to_string(), |p| format!("Protocol {p}"))
    );
    let _ = writeln!(
        out,
        "SEP-0048 Sections:    contractspecv0={}, contractenvmetav0={}, contractmetav0={}",
        report.metrics.has_contractspecv0,
        report.metrics.has_contractenvmetav0,
        report.metrics.has_contractmetav0
    );
}

/// Appends decoded `#[contracttype]` named structs (`ScMap`).
fn append_structs(out: &mut String, structs: &[StructSpec]) {
    if structs.is_empty() {
        return;
    }
    out.push_str("\n#[contracttype] Named Structs (ScMap):\n");
    for s in structs {
        let fields: Vec<String> = s
            .fields
            .iter()
            .map(|f| format!("{}: {}", f.name, f.ty))
            .collect();
        let _ = writeln!(out, "  - struct {} {{ {} }}", s.name, fields.join(", "));
    }
}

/// Appends decoded `#[contracttype]` positional tuple structs (`ScVec`).
fn append_tuple_structs(out: &mut String, tuple_structs: &[TupleStructSpec]) {
    if tuple_structs.is_empty() {
        return;
    }
    out.push_str("\n#[contracttype] Tuple Structs (ScVec):\n");
    for ts in tuple_structs {
        let fields: Vec<String> = ts.fields.iter().map(ToString::to_string).collect();
        let _ = writeln!(out, "  - struct {}({})", ts.name, fields.join(", "));
    }
}

/// Appends decoded `#[contracttype]` tagged unions (`ScSpecUdtUnionV0`).
fn append_unions(out: &mut String, unions: &[UnionSpec]) {
    if unions.is_empty() {
        return;
    }
    out.push_str("\n#[contracttype] Unions / DataKeys (ScSpecUdtUnionV0):\n");
    for u in unions {
        let cases: Vec<String> = u
            .cases
            .iter()
            .map(|c| {
                if c.payload.is_empty() {
                    c.name.clone()
                } else {
                    let p: Vec<String> = c.payload.iter().map(ToString::to_string).collect();
                    format!("{}({})", c.name, p.join(", "))
                }
            })
            .collect();
        let _ = writeln!(out, "  - enum {} {{ {} }}", u.name, cases.join(", "));
    }
}

/// Appends decoded `#[contracttype]` integer enums (`ScSpecUdtEnumV0`).
fn append_enums(out: &mut String, enums: &[EnumSpec]) {
    if enums.is_empty() {
        return;
    }
    out.push_str("\n#[contracttype] Integer Enums (ScSpecUdtEnumV0):\n");
    for e in enums {
        let vars: Vec<String> = e
            .variants
            .iter()
            .map(|v| format!("{} = {}", v.name, v.discriminant))
            .collect();
        let _ = writeln!(out, "  - enum {} {{ {} }}", e.name, vars.join(", "));
    }
}

/// Appends decoded exported contract entrypoints (`ScSpecFunctionV0`).
fn append_functions(out: &mut String, functions: &[FunctionSpec]) {
    if functions.is_empty() {
        return;
    }
    out.push_str("\nExported Contract Entrypoints (ScSpecFunctionV0):\n");
    for f in functions {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| format!("{}: {}", p.name, p.ty))
            .collect();
        let _ = writeln!(
            out,
            "  - fn {}({}) -> {}",
            f.name,
            params.join(", "),
            f.return_type
        );
    }
}

/// Appends the WASM binary metrics table in GitHub-Flavored Markdown.
fn append_markdown_metrics(out: &mut String, report: &WasmInspectionReport) {
    out.push_str("## Binary Metrics\n\n");
    out.push_str("| Metric | Value | Details |\n");
    out.push_str("| :--- | :--- | :--- |\n");

    let size_kib = report.metrics.byte_size as f64 / 1024.0;
    let _ = writeln!(
        out,
        "| **Binary Size** | `{} bytes` | `{:.2} KiB` / `64.00 KiB` limit |",
        report.metrics.byte_size, size_kib
    );

    let protocol_label = report
        .metrics
        .protocol_version
        .map_or_else(|| "missing".to_string(), |p| format!("Protocol {p}"));
    let _ = writeln!(
        out,
        "| **Soroban Protocol** | `{}` | Target network runtime interface |",
        protocol_label
    );

    let spec_status = if report.metrics.has_contractspecv0 {
        "Present"
    } else {
        "Missing"
    };
    let _ = writeln!(
        out,
        "| **contractspecv0 Section** | `{}` | Embedded SEP-0048 XDR contract specification |",
        spec_status
    );

    let env_status = if report.metrics.has_contractenvmetav0 {
        "Present"
    } else {
        "Missing"
    };
    let _ = writeln!(
        out,
        "| **contractenvmetav0 Section** | `{}` | Soroban host environment metadata |",
        env_status
    );

    let meta_status = if report.metrics.has_contractmetav0 {
        "Present"
    } else {
        "Missing"
    };
    let _ = writeln!(
        out,
        "| **contractmetav0 Section** | `{}` | Contract author metadata |",
        meta_status
    );
    out.push('\n');
}

/// Appends named structs (`ScMap`) as a Markdown table.
fn append_markdown_structs(out: &mut String, structs: &[StructSpec]) {
    if structs.is_empty() {
        return;
    }
    out.push_str("## #[contracttype] Named Structs (ScMap)\n\n");
    out.push_str("| Struct | Fields |\n");
    out.push_str("| :--- | :--- |\n");
    for s in structs {
        let fields: Vec<String> = s
            .fields
            .iter()
            .map(|f| format!("`{}: {}`", f.name, f.ty))
            .collect();
        let _ = writeln!(out, "| `{}` | {} |", s.name, fields.join(", "));
    }
    out.push('\n');
}

/// Appends positional tuple structs (`ScVec`) as a Markdown table.
fn append_markdown_tuple_structs(out: &mut String, tuple_structs: &[TupleStructSpec]) {
    if tuple_structs.is_empty() {
        return;
    }
    out.push_str("## #[contracttype] Tuple Structs (ScVec)\n\n");
    out.push_str("| Tuple Struct | Elements |\n");
    out.push_str("| :--- | :--- |\n");
    for ts in tuple_structs {
        let fields: Vec<String> = ts.fields.iter().map(|f| format!("`{f}`")).collect();
        let _ = writeln!(out, "| `{}` | {} |", ts.name, fields.join(", "));
    }
    out.push('\n');
}

/// Appends tagged unions (`ScSpecUdtUnionV0`) as a Markdown table.
fn append_markdown_unions(out: &mut String, unions: &[UnionSpec]) {
    if unions.is_empty() {
        return;
    }
    out.push_str("## #[contracttype] Unions / DataKeys (ScSpecUdtUnionV0)\n\n");
    out.push_str("| Union / DataKey | Cases |\n");
    out.push_str("| :--- | :--- |\n");
    for u in unions {
        let cases: Vec<String> = u
            .cases
            .iter()
            .map(|c| {
                if c.payload.is_empty() {
                    format!("`{}`", c.name)
                } else {
                    let p: Vec<String> = c.payload.iter().map(ToString::to_string).collect();
                    format!("`{}({})`", c.name, p.join(", "))
                }
            })
            .collect();
        let _ = writeln!(out, "| `{}` | {} |", u.name, cases.join(", "));
    }
    out.push('\n');
}

/// Appends integer enums (`ScSpecUdtEnumV0`) as a Markdown table.
fn append_markdown_enums(out: &mut String, enums: &[EnumSpec]) {
    if enums.is_empty() {
        return;
    }
    out.push_str("## #[contracttype] Integer Enums (ScSpecUdtEnumV0)\n\n");
    out.push_str("| Enum | Variants & Discriminants |\n");
    out.push_str("| :--- | :--- |\n");
    for e in enums {
        let vars: Vec<String> = e
            .variants
            .iter()
            .map(|v| format!("`{} = {}`", v.name, v.discriminant))
            .collect();
        let _ = writeln!(out, "| `{}` | {} |", e.name, vars.join(", "));
    }
    out.push('\n');
}

/// Appends exported contract entrypoints (`ScSpecFunctionV0`) as a Markdown table.
fn append_markdown_functions(out: &mut String, functions: &[FunctionSpec]) {
    if functions.is_empty() {
        return;
    }
    out.push_str("## Exported Contract Entrypoints (ScSpecFunctionV0)\n\n");
    out.push_str("| Function | Parameters | Return Type |\n");
    out.push_str("| :--- | :--- | :--- |\n");
    for f in functions {
        let params: Vec<String> = f
            .params
            .iter()
            .map(|p| format!("`{}: {}`", p.name, p.ty))
            .collect();
        let params_str = if params.is_empty() {
            "*(none)*".to_string()
        } else {
            params.join(", ")
        };
        let _ = writeln!(
            out,
            "| `{}` | {} | `{}` |",
            f.name, params_str, f.return_type
        );
    }
    out.push('\n');
}
