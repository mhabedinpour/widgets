use std::collections::HashMap;
use std::path::{Path, PathBuf};
use syn::{Attribute, Fields, FnArg, Item, TraitItem, Type};

use crate::codegen::model::{BindingDef, EventFieldDef, EventVariantDef, EventsDef, FieldDef, ServiceDef};
use crate::codegen::type_map::{expand, expand_return, Expansion};

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

            let service_name = t.ident.to_string().to_lowercase();

            let src_dir = path
                .parent()
                .expect("trait file has no parent directory")
                .to_path_buf();

            let structs = collect_structs(&src_dir);

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
                    let return_expansion = infer_return_expansion(&method.sig.output, &executor_method);
                    let data_type = extract_data_param_type(&method.sig);

                    let fields = match &data_type {
                        Some(dt) => find_struct_fields(&structs, dt),
                        None => Vec::new(),
                    };

                    bindings.push(BindingDef {
                        executor_method,
                        builder_name,
                        data_type,
                        fields,
                        return_expansion,
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

/// Parse every `.rs` file in `src_dir` once and index its structs by name.
/// On duplicate names, the first occurrence in sorted file order wins.
fn collect_structs(src_dir: &Path) -> HashMap<String, syn::ItemStruct> {
    let mut structs = HashMap::new();
    for path in sorted_rs_files(src_dir) {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let file = syn::parse_file(&source)
            .unwrap_or_else(|e| panic!("cannot parse {}: {e}", path.display()));

        for item in file.items {
            if let Item::Struct(s) = item {
                structs.entry(s.ident.to_string()).or_insert(s);
            }
        }
    }
    structs
}

/// Look up `struct_name` in the pre-parsed struct index and return its field definitions.
fn find_struct_fields(structs: &HashMap<String, syn::ItemStruct>, struct_name: &str) -> Vec<FieldDef> {
    let s = structs
        .get(struct_name)
        .unwrap_or_else(|| panic!("No struct `{struct_name}` found in scanned files"));
    let Fields::Named(named) = &s.fields else {
        panic!("Struct `{struct_name}` must have named fields");
    };

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
    for field in &named.named {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let type_name = type_name_from_syn(&field.ty, struct_name, &field_name);
        let field_doc = collect_doc(&field.attrs);
        let default = find_directive(&field_doc, "@default")
            .map(|line| extract_after_directive(&line, "@default"));
        let setter_name = find_directive(&field_doc, "@setter")
            .map(|line| extract_after_directive(&line, "@setter"));
        let required = required_set.contains(&field_name.as_str());
        let expansion = expand(&field_name, &type_name, struct_name, required, default.as_deref());
        fields.push(FieldDef { name: field_name, expansion, setter_name });
    }
    fields
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
        _ => panic!(
            "unexpected parameter type in @wasm method (only path types and references are supported)"
        ),
    }
}

/// Infer the return expansion from a method signature. Returns `None` for void.
fn infer_return_expansion(output: &syn::ReturnType, method_name: &str) -> Option<Expansion> {
    match output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Tuple(t) if t.elems.is_empty() => None,
            Type::Path(p) => {
                let seg = p
                    .path
                    .segments
                    .last()
                    .expect("return type path has no segments")
                    .ident
                    .to_string();
                Some(expand_return(&seg, method_name))
            }
            _ => panic!(
                "Unsupported return type shape on `{method_name}` (only plain path types are supported)"
            ),
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

    paths.sort();
    paths
}

// ── helpers ──────────────────────────────────────────────────────────────────

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

fn find_directive(doc: &str, directive: &str) -> Option<String> {
    doc.lines()
        .find(|line| line.contains(directive))
        .map(|s| s.to_string())
}

fn parse_kv(line: &str, key: &str) -> Option<String> {
    let needle = format!("{key}=\"");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn extract_after_directive(line: &str, directive: &str) -> String {
    let start = line.find(directive).unwrap() + directive.len();
    let rest = line[start..].trim();
    let end = rest.find('@').unwrap_or(rest.len());
    rest[..end].trim().to_string()
}

/// Extract the Rust type name (e.g. "u32", "Color", "Vec<String>") from a
/// `syn::Type`, including a single generic argument when present.
fn type_name_from_syn(ty: &Type, context: &str, field: &str) -> String {
    match ty {
        Type::Reference(r) => type_name_from_syn(&r.elem, context, field),
        Type::Path(p) => {
            let seg = p.path.segments.last().expect("type path has no segments");
            let ident = seg.ident.to_string();
            if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
                if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
                    return format!("{ident}<{}>", type_name_from_syn(t, context, field));
                }
            }
            ident
        }
        Type::Tuple(t) => {
            let elems: Vec<String> = t
                .elems
                .iter()
                .map(|e| type_name_from_syn(e, context, field))
                .collect();
            format!("({})", elems.join(", "))
        }
        _ => panic!(
            "Unsupported type shape on {context}::{field} (only path types, references, and tuples are supported)"
        ),
    }
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

                let context = format!("WidgetEvent::{variant_name}");
                let fields = named
                    .named
                    .iter()
                    .map(|f| {
                        let field_name = f.ident.as_ref().unwrap().to_string();
                        let type_name = type_name_from_syn(&f.ty, &context, &field_name);
                        let expansion = expand(&field_name, &type_name, &context, true, None);
                        EventFieldDef { name: field_name, expansion }
                    })
                    .collect();

                variants.push(EventVariantDef {
                    name: variant_name,
                    index: idx,
                    fields,
                });
            }

            return EventsDef { variants };
        }
    }

    panic!(
        "WidgetEvent enum not found in {}",
        widget_mod_path.display()
    );
}
