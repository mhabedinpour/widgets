use std::path::Path;
use syn::{Attribute, Fields, Item, Type};

use crate::codegen::model::{BindingDef, FieldDef, FieldType, ServiceDef};

pub fn parse_service(name: &str, src_dir: &Path) -> ServiceDef {
    let mut bindings = Vec::new();

    let entries = std::fs::read_dir(src_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", src_dir.display()));

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "rs"))
        .collect();

    paths.sort(); // deterministic ordering

    for path in paths {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));

        let file = syn::parse_file(&source)
            .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()));

        for item in file.items {
            if let Item::Struct(s) = item {
                let struct_name = s.ident.to_string();
                if !struct_name.ends_with("Data") {
                    continue;
                }

                let doc = collect_doc(&s.attrs);
                let Some(wasm_line) = find_directive(&doc, "@wasm") else {
                    continue;
                };

                let wasm_module = parse_kv(&wasm_line, "module")
                    .unwrap_or_else(|| panic!("@wasm on {struct_name} missing module="));
                let wasm_fn = parse_kv(&wasm_line, "fn")
                    .unwrap_or_else(|| panic!("@wasm on {struct_name} missing fn="));
                let executor_method = parse_kv(&wasm_line, "executor")
                    .unwrap_or_else(|| panic!("@wasm on {struct_name} missing executor="));
                let required_raw = parse_kv(&wasm_line, "required").unwrap_or_default();
                let required_fields: Vec<&str> = required_raw
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();

                let Fields::Named(named) = s.fields else {
                    continue;
                };

                let mut fields = Vec::new();
                for field in named.named {
                    let field_name = field.ident.unwrap().to_string();
                    let ty = resolve_type(&field.ty, &struct_name, &field_name);
                    let field_doc = collect_doc(&field.attrs);

                    let default = find_directive(&field_doc, "@default").map(|line| {
                        // Everything after "@default" up to the next "@" or end
                        extract_after_directive(&line, "@default")
                    });

                    let setter_name = find_directive(&field_doc, "@setter")
                        .map(|line| extract_after_directive(&line, "@setter"));

                    let is_required = required_fields.contains(&field_name.as_str());

                    fields.push(FieldDef {
                        name: field_name,
                        ty,
                        required: is_required,
                        default,
                        setter_name,
                    });
                }

                bindings.push(BindingDef {
                    wasm_module,
                    wasm_fn,
                    executor_method,
                    data_type: struct_name,
                    fields,
                });
            }
        }
    }

    ServiceDef {
        name: name.to_string(),
        bindings,
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Collect all `#[doc = "..."]` attribute strings into one joined string.
fn collect_doc(attrs: &[Attribute]) -> String {
    attrs
        .iter()
        .filter_map(|a| {
            if !a.path().is_ident("doc") {
                return None;
            }
            if let syn::Meta::NameValue(nv) = &a.meta {
                if let syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(s),
                    ..
                }) = &nv.value
                {
                    return Some(s.value());
                }
            }
            None
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find the first line in `doc` that contains `directive` and return that line.
fn find_directive<'a>(doc: &'a str, directive: &str) -> Option<String> {
    doc.lines()
        .find(|line| line.contains(directive))
        .map(|s| s.to_string())
}

/// Parse `key="value"` from a directive line.
fn parse_kv(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

/// Extract the text immediately after a directive keyword, stopping at the next `@` or end of line.
fn extract_after_directive(line: &str, directive: &str) -> String {
    let start = line.find(directive).unwrap() + directive.len();
    let rest = line[start..].trim();
    // Stop at next directive
    let end = rest.find('@').unwrap_or(rest.len());
    rest[..end].trim().to_string()
}

/// Map a `syn::Type` to our `FieldType` enum.
fn resolve_type(ty: &Type, struct_name: &str, field_name: &str) -> FieldType {
    match ty {
        Type::Reference(r) => {
            // &str or &'a str
            resolve_type(&r.elem, struct_name, field_name)
        }
        Type::Path(p) => {
            let seg = p
                .path
                .segments
                .last()
                .expect("type path has no segments")
                .ident
                .to_string();
            match seg.as_str() {
                "u32" | "u8" | "u16" | "u64" => FieldType::U32,
                "bool" => FieldType::Bool,
                "Color" => FieldType::Color,
                "Point" => FieldType::Point,
                "Rect" => FieldType::Rect,
                "str" => FieldType::Str,
                other => panic!(
                    "Unknown field type `{other}` on {struct_name}::{field_name}. \
                     Add it to type_map.rs."
                ),
            }
        }
        other => panic!("Unsupported type shape on {struct_name}::{field_name}: {other:?}"),
    }
}
