use std::path::{Path, PathBuf};
use syn::{Attribute, Fields, FnArg, Item, TraitItem, Type};

use crate::codegen::model::{EventFieldDef, EventVariantDef, EventsDef};

use crate::codegen::model::{BindingDef, FieldDef, FieldType, ReturnType, ServiceDef};

/// Recursively scan `src_root` for Rust traits annotated with `@wasm` and
/// return a `ServiceDef` for each one found.
pub fn scan_services(src_root: &Path) -> Vec<ServiceDef> {
    let mut services = Vec::new();
    scan_dir(src_root, &mut services);
    services
}

fn scan_dir(dir: &Path, services: &mut Vec<ServiceDef>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if path.extension().map_or(false, |e| e == "rs") {
            if let Some(svc) = try_parse_service_from_file(&path) {
                services.push(svc);
            }
        }
    }

    for subdir in subdirs {
        scan_dir(&subdir, services);
    }
}

/// Parse a single `.rs` file; if it contains a trait annotated with `@wasm`,
/// build a `ServiceDef` driven by the trait's method annotations.
fn try_parse_service_from_file(path: &Path) -> Option<ServiceDef> {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let file =
        syn::parse_file(&source).unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()));

    for item in &file.items {
        if let Item::Trait(t) = item {
            let doc = collect_doc(&t.attrs);
            let Some(_wasm_line) = find_directive(&doc, "@wasm") else {
                continue;
            };

            // Service name comes from the trait name (lower-cased).
            let service_name = t.ident.to_string().to_lowercase();

            let src_dir = path
                .parent()
                .expect("trait file has no parent directory")
                .to_path_buf();

            let paths = sorted_rs_files(&src_dir);

            // Each trait method with @wasm becomes one binding.
            let mut bindings = Vec::new();
            for ti in &t.items {
                if let TraitItem::Fn(method) = ti {
                    let method_doc = collect_doc(&method.attrs);
                    let Some(method_wasm_line) = find_directive(&method_doc, "@wasm") else {
                        continue;
                    };

                    let executor_method = method.sig.ident.to_string();
                    let builder_name =
                        parse_kv(&method_wasm_line, "builder_name").unwrap_or_else(|| {
                            panic!("@wasm on `{executor_method}` missing builder_name=")
                        });
                    let return_type = infer_return_type(&method.sig.output, &executor_method);
                    let data_type = extract_data_param_type(&method.sig).unwrap_or_else(|| {
                        panic!("@wasm method `{executor_method}` has no data parameter")
                    });

                    // Field definitions and required flags come from the struct annotation.
                    let fields = find_struct_fields(&paths, &data_type);

                    bindings.push(BindingDef {
                        executor_method,
                        builder_name,
                        data_type,
                        fields,
                        return_type,
                    });
                }
            }

            return Some(ServiceDef {
                name: service_name,
                bindings,
            });
        }
    }

    None
}

/// Scan `paths` for a struct named `struct_name` and return its field definitions.
/// The `required` flag on each field is derived from the struct's own `@wasm required="..."` doc.
fn find_struct_fields(paths: &[PathBuf], struct_name: &str) -> Vec<FieldDef> {
    for path in paths {
        let source = std::fs::read_to_string(path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let file = syn::parse_file(&source)
            .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()));

        for item in file.items {
            if let Item::Struct(s) = item {
                if s.ident != struct_name {
                    continue;
                }
                let Fields::Named(named) = s.fields else {
                    continue;
                };

                // Read required fields from the struct's @wasm annotation.
                let struct_doc = collect_doc(&s.attrs);
                let required_raw = find_directive(&struct_doc, "@wasm")
                    .and_then(|line| parse_kv(&line, "required"))
                    .unwrap_or_default();
                let required_set: Vec<&str> = required_raw
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect();

                let mut fields = Vec::new();
                for field in named.named {
                    let field_name = field.ident.unwrap().to_string();
                    let ty = resolve_type(&field.ty, struct_name, &field_name);
                    let field_doc = collect_doc(&field.attrs);
                    let default = find_directive(&field_doc, "@default")
                        .map(|line| extract_after_directive(&line, "@default"));
                    let setter_name = find_directive(&field_doc, "@setter")
                        .map(|line| extract_after_directive(&line, "@setter"));
                    fields.push(FieldDef {
                        required: required_set.contains(&field_name.as_str()),
                        name: field_name,
                        ty,
                        default,
                        setter_name,
                    });
                }
                return fields;
            }
        }
    }
    panic!("No struct `{struct_name}` found in scanned files");
}

/// Extract the name of the first non-`self` parameter type from a method signature.
fn extract_data_param_type(sig: &syn::Signature) -> Option<String> {
    for input in &sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            return Some(path_type_name(&pat_type.ty));
        }
    }
    None
}

fn path_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .expect("type path has no segments")
            .ident
            .to_string(),
        Type::Reference(r) => path_type_name(&r.elem),
        other => panic!("unexpected parameter type in @wasm method: {other:?}"),
    }
}

/// Map a `syn::ReturnType` to our `ReturnType` enum, returning `None` for void.
fn infer_return_type(output: &syn::ReturnType, method_name: &str) -> Option<ReturnType> {
    match output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(t) if t.elems.is_empty() => None, // explicit `-> ()`
            Type::Path(p) => {
                let seg = p
                    .path
                    .segments
                    .last()
                    .expect("return type path has no segments")
                    .ident
                    .to_string();
                match seg.as_str() {
                    "u32" | "u8" | "u16" | "u64" | "usize" => Some(ReturnType::U32),
                    "bool" => Some(ReturnType::Bool),
                    "i32" | "i8" | "i16" | "i64" => Some(ReturnType::I32),
                    other => panic!(
                        "Unsupported return type `{other}` on trait method `{method_name}`. \
                         Supported: u32/usize/u8/u16/u64, bool, i32/i8/i16/i64."
                    ),
                }
            }
            other => {
                panic!("Unsupported return type shape on trait method `{method_name}`: {other:?}")
            }
        },
    }
}

fn sorted_rs_files(src_dir: &Path) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(src_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", src_dir.display()));

    let mut paths: Vec<_> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "rs"))
        .collect();

    paths.sort(); // deterministic ordering
    paths
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
fn find_directive(doc: &str, directive: &str) -> Option<String> {
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
    let end = rest.find('@').unwrap_or(rest.len());
    rest[..end].trim().to_string()
}

/// Parse `src/widget/mod.rs` and extract all variants of the `WidgetEvent` enum.
pub fn scan_events(widget_mod_path: &Path) -> EventsDef {
    let source = std::fs::read_to_string(widget_mod_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", widget_mod_path.display()));
    let file = syn::parse_file(&source)
        .unwrap_or_else(|e| panic!("cannot parse {}: {e}", widget_mod_path.display()));

    for item in &file.items {
        if let Item::Enum(e) = item {
            if e.ident != "WidgetEvent" {
                continue;
            }

            let mut variants = Vec::new();
            for (idx, variant) in e.variants.iter().enumerate() {
                let variant_name = variant.ident.to_string();
                let Fields::Named(named) = &variant.fields else {
                    panic!("WidgetEvent::{variant_name} must use named fields");
                };

                let fields = named
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let ty = resolve_event_field_type(&f.ty, &variant_name, &field_name);
                        EventFieldDef { name: field_name, ty }
                    })
                    .collect();

                variants.push(EventVariantDef { name: variant_name, index: idx, fields });
            }

            return EventsDef { variants };
        }
    }

    panic!("WidgetEvent enum not found in {}", widget_mod_path.display());
}

fn resolve_event_field_type(ty: &Type, variant: &str, field: &str) -> FieldType {
    match ty {
        Type::Reference(r) => resolve_event_field_type(&r.elem, variant, field),
        Type::Path(p) => {
            let seg = p.path.segments.last().unwrap().ident.to_string();
            match seg.as_str() {
                "u32" | "u8" | "u16" | "u64" => FieldType::U32,
                "i32" | "i8" | "i16" | "i64" => FieldType::U32,
                "TimerId" => FieldType::TimerId,
                "usize" => FieldType::Usize,
                "bool" => FieldType::Bool,
                "String" | "str" => FieldType::Str,
                other => panic!(
                    "Unknown field type `{other}` in WidgetEvent::{variant}::{field}. \
                     Add it to parser.rs resolve_event_field_type."
                ),
            }
        }
        other => panic!(
            "Unsupported type shape in WidgetEvent::{variant}::{field}: {other:?}"
        ),
    }
}

/// Map a `syn::Type` to our `FieldType` enum.
fn resolve_type(ty: &Type, struct_name: &str, field_name: &str) -> FieldType {
    match ty {
        Type::Reference(r) => resolve_type(&r.elem, struct_name, field_name),
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
                "TimerId" => FieldType::TimerId,
                "usize" => FieldType::Usize,
                "bool" => FieldType::Bool,
                "Color" => FieldType::Color,
                "Point" => FieldType::Point,
                "Rect" => FieldType::Rect,
                "str" => FieldType::Str,
                "Duration" => FieldType::Duration,
                other => panic!(
                    "Unknown field type `{other}` on {struct_name}::{field_name}. \
                     Add it to type_map.rs."
                ),
            }
        }
        other => panic!("Unsupported type shape on {struct_name}::{field_name}: {other:?}"),
    }
}
