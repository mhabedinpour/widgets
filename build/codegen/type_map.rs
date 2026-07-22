use super::render;

/// One scalar value crossing the WASM boundary.
///
/// `ty` is the AssemblyScript-side scalar type: "u32", "u64", "i32", "i64",
/// or "usize". A "usize" field is a pointer into guest memory; generators
/// handle pointers with the generic buffer-copy protocol (the receiver
/// allocates a buffer sized by the matching `*_len` field, which must
/// precede the pointer in the ABI list).
pub struct AbiField {
    pub name: String,
    pub ty: &'static str,
}

/// One direction of a conversion between a typed value and its ABI scalars.
pub struct Conversion {
    /// Statements emitted before the expressions are used ("" when not needed).
    /// May reference `%v%` (typed value) or the ABI field names, depending on
    /// direction — same rules as `exprs`.
    pub preamble: String,
    /// Typed → ABI: one expression per ABI field; `%v%` is the typed value.
    /// Callers may substitute a place expression like `*field`, so converters
    /// must write `(%v%)` when a `.` follows — and only then, to keep the
    /// generated code free of unnecessary-parentheses warnings.
    /// ABI → typed: a single expression constructing the typed value from the
    /// ABI field names, which must be in scope as variables.
    pub exprs: Vec<String>,
}

/// Everything the generators need to know about one field or return type.
///
/// The ABI list says *what* crosses the boundary; the four converters say
/// *how* each side turns its native value into ABI scalars and back. The
/// generators never look at the original type name — whether the value flows
/// through a builder call, an event, or a return value, they only combine
/// the ABI fields with the right converter for the direction.
pub struct Expansion {
    // ── TypeAssembly representation ──────────────────────────────────────────
    /// The TS-side type ("u32", "Color", "string", …).
    pub ta_type: String,
    /// `None` = required (constructor parameter, no setter).
    /// `Some(expr)` = optional (field default + setter).
    pub ta_default: Option<String>,

    // ── ABI: the scalars this type occupies on the wire, in call order ──────
    pub abi: Vec<AbiField>,

    // ── The four converters ──────────────────────────────────────────────────
    /// Host serializes: Rust value → ABI scalars.
    pub rust_to_abi: Conversion,
    /// Host deserializes: ABI scalars → Rust value.
    pub abi_to_rust: Conversion,
    /// TA serializes: TS value → ABI scalars.
    pub ts_to_abi: Conversion,
    /// TA deserializes: ABI scalars → TS value.
    pub abi_to_ts: Conversion,
}

fn abi(name: String, ty: &'static str) -> AbiField {
    AbiField { name, ty }
}

fn conv(preamble: String, exprs: Vec<String>) -> Conversion {
    Conversion { preamble, exprs }
}

fn expr(e: String) -> Conversion {
    conv(String::new(), vec![e])
}

fn int_expansion(name: &str, ty: &'static str) -> Expansion {
    Expansion {
        ta_type: ty.to_string(),
        ta_default: Some("0".to_string()),
        abi: vec![abi(name.to_string(), ty)],
        rust_to_abi: expr(format!("%v% as {ty}")),
        abi_to_rust: expr(format!("{name} as _")),
        ts_to_abi: expr("%v%".to_string()),
        abi_to_ts: expr(name.to_string()),
    }
}

/// Expand a field into all per-generator data.
///
/// `type_name` is the Rust identifier as it appears in source ("u32", "Color", …).
/// `context`   is used in the panic message (e.g. the struct or enum variant name).
/// `required` and `default` affect the TA builder side only.
///
/// Adding a new type: add one match arm and nowhere else. The arm's
/// `ta_default` is the type's built-in default; `required` / `@default`
/// handling is applied uniformly afterwards.
pub fn expand(
    name: &str,
    type_name: &str,
    context: &str,
    required: bool,
    default: Option<&str>,
) -> Expansion {
    let mut e = match type_name {
        "u32" | "u8" | "u16" | "usize" => int_expansion(name, "u32"),
        "u64" => int_expansion(name, "u64"),
        "i32" | "i8" | "i16" => int_expansion(name, "i32"),
        "i64" => int_expansion(name, "i64"),

        "bool" => Expansion {
            ta_type: "bool".to_string(),
            ta_default: Some("false".to_string()),
            abi: vec![abi(name.to_string(), "u32")],
            rust_to_abi: expr("if %v% { 1u32 } else { 0u32 }".to_string()),
            abi_to_rust: expr(format!("{name} != 0")),
            ts_to_abi: expr("%v% ? 1 : 0".to_string()),
            abi_to_ts: expr(format!("{name} != 0")),
        },

        "Color" => Expansion {
            ta_type: "Color".to_string(),
            ta_default: Some("Color.WHITE".to_string()),
            abi: vec![
                abi(format!("{name}_r"), "u32"),
                abi(format!("{name}_g"), "u32"),
                abi(format!("{name}_b"), "u32"),
            ],
            rust_to_abi: conv(
                String::new(),
                vec![
                    "match %v% { crate::drawer::types::Color::Rgb(r, _, _) => r as u32, _ => 0 }"
                        .to_string(),
                    "match %v% { crate::drawer::types::Color::Rgb(_, g, _) => g as u32, _ => 0 }"
                        .to_string(),
                    "match %v% { crate::drawer::types::Color::Rgb(_, _, b) => b as u32, _ => 0 }"
                        .to_string(),
                ],
            ),
            abi_to_rust: expr(format!(
                "crate::drawer::types::Color::Rgb({name}_r as u8, {name}_g as u8, {name}_b as u8)"
            )),
            ts_to_abi: conv(
                String::new(),
                vec![
                    "%v%.r".to_string(),
                    "%v%.g".to_string(),
                    "%v%.b".to_string(),
                ],
            ),
            abi_to_ts: expr(format!(
                "new Color(<u8>{name}_r, <u8>{name}_g, <u8>{name}_b)"
            )),
        },

        "Point" => Expansion {
            ta_type: "Point".to_string(),
            ta_default: Some("new Point(0, 0)".to_string()),
            abi: vec![
                abi(format!("{name}_x"), "u32"),
                abi(format!("{name}_y"), "u32"),
            ],
            rust_to_abi: conv(
                String::new(),
                vec!["(%v%).x".to_string(), "(%v%).y".to_string()],
            ),
            abi_to_rust: expr(format!(
                "crate::drawer::types::Point {{ x: {name}_x, y: {name}_y }}"
            )),
            ts_to_abi: conv(
                String::new(),
                vec!["%v%.x".to_string(), "%v%.y".to_string()],
            ),
            abi_to_ts: expr(format!("new Point({name}_x, {name}_y)")),
        },

        "Rect" => Expansion {
            ta_type: "Rect".to_string(),
            ta_default: Some("new Rect(0, 0, 0, 0)".to_string()),
            abi: vec![
                abi(format!("{name}_x"), "u32"),
                abi(format!("{name}_y"), "u32"),
                abi(format!("{name}_width"), "u32"),
                abi(format!("{name}_height"), "u32"),
            ],
            rust_to_abi: conv(
                String::new(),
                vec![
                    "(%v%).origin.x".to_string(),
                    "(%v%).origin.y".to_string(),
                    "(%v%).size.width".to_string(),
                    "(%v%).size.height".to_string(),
                ],
            ),
            abi_to_rust: expr(format!(
                "crate::drawer::types::Rect {{ origin: crate::drawer::types::Point {{ x: {name}_x, y: {name}_y }}, size: crate::drawer::types::Size {{ width: {name}_width, height: {name}_height }} }}"
            )),
            ts_to_abi: conv(
                String::new(),
                vec![
                    "%v%.x".to_string(),
                    "%v%.y".to_string(),
                    "%v%.width".to_string(),
                    "%v%.height".to_string(),
                ],
            ),
            abi_to_ts: expr(format!(
                "new Rect({name}_x, {name}_y, {name}_width, {name}_height)"
            )),
        },

        "str" | "String" => Expansion {
            ta_type: "string".to_string(),
            ta_default: Some("\"\"".to_string()),
            abi: vec![
                abi(format!("{name}_len"), "u32"),
                abi(format!("{name}_ptr"), "usize"),
            ],
            rust_to_abi: conv(
                String::new(),
                vec![
                    "(%v%).len() as u32".to_string(),
                    "(%v%).as_bytes().to_vec()".to_string(),
                ],
            ),
            abi_to_rust: conv(
                read_guest_bytes(name)
                    + &render(
                        r#"                    let %n%_utf8 = match core::str::from_utf8(%n%_bytes) {
                        Ok(s) => s,
                        Err(_) => return Err(wasmi::Error::new("field `%n%` is not valid UTF-8")),
                    };
                    let mut %n%_buf = alloc::string::String::new();
                    %n%_buf.push_str(%n%_utf8);
"#,
                        &[("n", name)],
                    ),
                vec![format!("{name}_buf")],
            ),
            ts_to_abi: conv(
                render(
                    "    const __%n%_buf = String.UTF8.encode(%v%);\n",
                    &[("n", name)],
                ),
                vec![
                    format!("__{name}_buf.byteLength"),
                    format!("changetype<usize>(__{name}_buf)"),
                ],
            ),
            abi_to_ts: expr(format!("String.UTF8.decodeUnsafe({name}_ptr, {name}_len)")),
        },

        "Duration" => Expansion {
            ta_type: "Duration".to_string(),
            ta_default: Some("new Duration(0)".to_string()),
            abi: vec![abi(name.to_string(), "u64")],
            rust_to_abi: expr("(%v%).as_ticks()".to_string()),
            abi_to_rust: expr(format!("embassy_time::Duration::from_ticks({name})")),
            ts_to_abi: expr("%v%.ticks".to_string()),
            abi_to_ts: expr(format!("new Duration({name})")),
        },

        "TimerId" => Expansion {
            ta_type: "u32".to_string(),
            ta_default: Some("0".to_string()),
            abi: vec![abi(name.to_string(), "u32")],
            rust_to_abi: expr("(%v%).0".to_string()),
            abi_to_rust: expr(format!("crate::time::TimerId({name})")),
            ts_to_abi: expr("%v%".to_string()),
            abi_to_ts: expr(name.to_string()),
        },

        "RequestId" => Expansion {
            ta_type: "u32".to_string(),
            ta_default: Some("0".to_string()),
            abi: vec![abi(name.to_string(), "u32")],
            rust_to_abi: expr("(%v%).0".to_string()),
            abi_to_rust: expr(format!("crate::http::RequestId({name})")),
            ts_to_abi: expr("%v%".to_string()),
            abi_to_ts: expr(name.to_string()),
        },

        "StrokeAlignment" => Expansion {
            ta_type: "StrokeAlignment".to_string(),
            ta_default: Some("StrokeAlignment.Center".to_string()),
            abi: vec![abi(name.to_string(), "u32")],
            rust_to_abi: expr("%v% as u32".to_string()),
            abi_to_rust: expr(format!(
                "crate::drawer::types::StrokeAlignment::from_int({name})"
            )),
            ts_to_abi: expr("%v% as u32".to_string()),
            abi_to_ts: expr(format!("{name} as StrokeAlignment")),
        },

        "Font" => Expansion {
            ta_type: "Font".to_string(),
            ta_default: Some("Font.Font6x10".to_string()),
            abi: vec![abi(name.to_string(), "u32")],
            rust_to_abi: expr("%v% as u32".to_string()),
            abi_to_rust: expr(format!("crate::drawer::types::Font::from_int({name})")),
            ts_to_abi: expr("%v% as u32".to_string()),
            abi_to_ts: expr(format!("{name} as Font")),
        },

        "TextAlignment" => Expansion {
            ta_type: "TextAlignment".to_string(),
            ta_default: Some("TextAlignment.Left".to_string()),
            abi: vec![abi(name.to_string(), "u32")],
            rust_to_abi: expr("%v% as u32".to_string()),
            abi_to_rust: expr(format!(
                "crate::drawer::types::TextAlignment::from_int({name})"
            )),
            ts_to_abi: expr("%v% as u32".to_string()),
            abi_to_ts: expr(format!("{name} as TextAlignment")),
        },

        "Baseline" => Expansion {
            ta_type: "Baseline".to_string(),
            ta_default: Some("Baseline.Alphabetic".to_string()),
            abi: vec![abi(name.to_string(), "u32")],
            rust_to_abi: expr("%v% as u32".to_string()),
            abi_to_rust: expr(format!("crate::drawer::types::Baseline::from_int({name})")),
            ts_to_abi: expr("%v% as u32".to_string()),
            abi_to_ts: expr(format!("{name} as Baseline")),
        },

        t if t.starts_with("Vec<") && t.ends_with('>') => {
            array_expansion(name, &t[4..t.len() - 1], context)
        }

        unknown => {
            panic!("Unknown type `{unknown}` on {context}::{name}. Add a match arm to type_map.rs.")
        }
    };

    // Uniform required/default handling: an `@default` annotation overrides
    // the type's built-in default; required fields have no default at all.
    e.ta_default = match (required, default) {
        (true, _) => None,
        (false, Some(d)) => Some(rust_default_to_ts(d)),
        (false, None) => e.ta_default,
    };
    e
}

/// Expand a return type. A return value is just a value flowing Host → TA,
/// so this is `expand` constrained to types that fit in a single WASM
/// return scalar. The ABI field is named `ret`.
pub fn expand_return(type_name: &str, method_name: &str) -> Expansion {
    let e = expand("ret", type_name, method_name, true, None);
    assert!(
        e.abi.len() == 1 && e.abi[0].ty != "usize",
        "Return type `{type_name}` on `{method_name}` does not fit in a single WASM return scalar."
    );
    e
}

// ── Arrays ────────────────────────────────────────────────────────────────────
//
// Every array crosses the boundary as the same `{n}_len` (byte length) +
// `{n}_ptr` pair as String — the pointer protocol is unchanged. Only the
// converters differ per element type: integer elements are the raw
// little-endian element bytes; string elements use a packed encoding of
// `u32 byte-length | utf8 bytes` per element inside the one buffer.

fn array_expansion(name: &str, elem: &str, context: &str) -> Expansion {
    match elem {
        "str" | "String" => string_array_expansion(name),
        "(String, String)" | "(str, str)" => string_pair_array_expansion(name),
        _ => match int_elem(elem) {
            Some((ts_elem, size, shift)) => int_array_expansion(name, elem, ts_elem, size, shift),
            None => panic!(
                "Unsupported array element type `{elem}` on {context}::{name}. \
                 Supported: integer scalars, String, and (String, String)."
            ),
        },
    }
}

/// (TS element type, byte size, shift) for an integer array element.
fn int_elem(elem: &str) -> Option<(&'static str, usize, u32)> {
    match elem {
        "u8" => Some(("u8", 1, 0)),
        "i8" => Some(("i8", 1, 0)),
        "u16" => Some(("u16", 2, 1)),
        "i16" => Some(("i16", 2, 1)),
        "u32" | "usize" => Some(("u32", 4, 2)),
        "i32" => Some(("i32", 4, 2)),
        "u64" => Some(("u64", 8, 3)),
        "i64" => Some(("i64", 8, 3)),
        _ => None,
    }
}

fn int_array_expansion(
    name: &str,
    elem: &str,
    ts_elem: &'static str,
    size: usize,
    shift: u32,
) -> Expansion {
    Expansion {
        ta_type: format!("{ts_elem}[]"),
        ta_default: Some("[]".to_string()),
        abi: vec![
            abi(format!("{name}_len"), "u32"),
            abi(format!("{name}_ptr"), "usize"),
        ],
        rust_to_abi: conv(
            String::new(),
            vec![
                format!("((%v%).len() * {size}) as u32"),
                "(%v%).iter().flat_map(|__x| __x.to_le_bytes()).collect::<alloc::vec::Vec<u8>>()"
                    .to_string(),
            ],
        ),
        abi_to_rust: conv(
            read_guest_bytes(name)
                + &render(
                    r#"                    let %n%_vec: alloc::vec::Vec<%elem%> = %n%_bytes
                        .chunks_exact(%size%)
                        .map(|__c| %elem%::from_le_bytes(__c.try_into().unwrap()))
                        .collect();
"#,
                    &[("n", name), ("elem", elem), ("size", &size.to_string())],
                ),
            vec![format!("{name}_vec")],
        ),
        ts_to_abi: conv(
            String::new(),
            vec![
                format!("(%v%).length << {shift}"),
                "(%v%).dataStart".to_string(),
            ],
        ),
        abi_to_ts: conv(
            render(
                r#"    const %n%_arr = new Array<%ts_elem%>(<i32>(%n%_len >>> %shift%));
    for (let %n%_i = 0; %n%_i < %n%_arr.length; %n%_i++) {
      %n%_arr[%n%_i] = load<%ts_elem%>(%n%_ptr + (<usize>%n%_i << %shift%));
    }
"#,
                &[
                    ("n", name),
                    ("ts_elem", ts_elem),
                    ("shift", &shift.to_string()),
                ],
            ),
            vec![format!("{name}_arr")],
        ),
    }
}

fn string_array_expansion(name: &str) -> Expansion {
    Expansion {
        ta_type: "string[]".to_string(),
        ta_default: Some("[]".to_string()),
        abi: vec![
            abi(format!("{name}_len"), "u32"),
            abi(format!("{name}_ptr"), "usize"),
        ],
        rust_to_abi: conv(String::new(), vec![
            "(%v%).iter().map(|__s| 4 + __s.len()).sum::<usize>() as u32".to_string(),
            "{ let mut __b: alloc::vec::Vec<u8> = alloc::vec::Vec::new(); for __s in (%v%).iter() { __b.extend_from_slice(&(__s.len() as u32).to_le_bytes()); __b.extend_from_slice(__s.as_bytes()); } __b }".to_string(),
        ]),
        abi_to_rust: conv(
            read_guest_bytes(name) + &rust_parse_packed_strings(name, &format!("{name}_vec")),
            vec![format!("{name}_vec")],
        ),
        ts_to_abi: conv(
            render(
                r#"    let __%n%_total = 0;
    const __%n%_enc = new Array<ArrayBuffer>();
    for (let __%n%_i = 0; __%n%_i < (%v%).length; __%n%_i++) {
      const __%n%_b = String.UTF8.encode((%v%)[__%n%_i]);
      __%n%_enc.push(__%n%_b);
      __%n%_total += 4 + __%n%_b.byteLength;
    }
"#,
                &[("n", name)],
            ) + &ts_pack_strings(name),
            vec![
                format!("__{name}_buf.byteLength"),
                format!("changetype<usize>(__{name}_buf)"),
            ],
        ),
        abi_to_ts: conv(
            render(
                r#"    const %n%_arr = new Array<string>();
    let %n%_off: usize = 0;
    while (%n%_off + 4 <= <usize>%n%_len) {
      const %n%_slen = load<u32>(%n%_ptr + %n%_off);
      %n%_off += 4;
      %n%_arr.push(String.UTF8.decodeUnsafe(%n%_ptr + %n%_off, %n%_slen));
      %n%_off += %n%_slen;
    }
"#,
                &[("n", name)],
            ),
            vec![format!("{name}_arr")],
        ),
    }
}

/// `Vec<(String, String)>` ⇄ `string[][]` — e.g. HTTP headers. Uses the same
/// packed string encoding as `Vec<String>` with the pairs flattened in order
/// (k1, v1, k2, v2, …) and regrouped on decode.
fn string_pair_array_expansion(name: &str) -> Expansion {
    Expansion {
        ta_type: "string[][]".to_string(),
        ta_default: Some("[]".to_string()),
        abi: vec![
            abi(format!("{name}_len"), "u32"),
            abi(format!("{name}_ptr"), "usize"),
        ],
        rust_to_abi: conv(String::new(), vec![
            "(%v%).iter().map(|__p| 8 + __p.0.len() + __p.1.len()).sum::<usize>() as u32".to_string(),
            "{ let mut __b: alloc::vec::Vec<u8> = alloc::vec::Vec::new(); for __p in (%v%).iter() { __b.extend_from_slice(&(__p.0.len() as u32).to_le_bytes()); __b.extend_from_slice(__p.0.as_bytes()); __b.extend_from_slice(&(__p.1.len() as u32).to_le_bytes()); __b.extend_from_slice(__p.1.as_bytes()); } __b }".to_string(),
        ]),
        abi_to_rust: conv(
            read_guest_bytes(name)
                + &rust_parse_packed_strings(name, &format!("{name}_flat"))
                + &render(
                    r#"                    let mut %n%_vec: alloc::vec::Vec<(alloc::string::String, alloc::string::String)> = alloc::vec::Vec::new();
                    let mut %n%_flat_it = %n%_flat.into_iter();
                    while let (Some(%n%_k), Some(%n%_v)) = (%n%_flat_it.next(), %n%_flat_it.next()) {
                        %n%_vec.push((%n%_k, %n%_v));
                    }
"#,
                    &[("n", name)],
                ),
            vec![format!("{name}_vec")],
        ),
        ts_to_abi: conv(
            render(
                r#"    let __%n%_total = 0;
    const __%n%_enc = new Array<ArrayBuffer>();
    for (let __%n%_i = 0; __%n%_i < (%v%).length; __%n%_i++) {
      const __%n%_pair = (%v%)[__%n%_i];
      for (let __%n%_j = 0; __%n%_j < __%n%_pair.length; __%n%_j++) {
        const __%n%_b = String.UTF8.encode(__%n%_pair[__%n%_j]);
        __%n%_enc.push(__%n%_b);
        __%n%_total += 4 + __%n%_b.byteLength;
      }
    }
"#,
                &[("n", name)],
            ) + &ts_pack_strings(name),
            vec![
                format!("__{name}_buf.byteLength"),
                format!("changetype<usize>(__{name}_buf)"),
            ],
        ),
        abi_to_ts: conv(
            render(
                r#"    const %n%_arr = new Array<string[]>();
    let %n%_off: usize = 0;
    while (%n%_off + 4 <= <usize>%n%_len) {
      const %n%_klen = load<u32>(%n%_ptr + %n%_off);
      %n%_off += 4;
      const %n%_k = String.UTF8.decodeUnsafe(%n%_ptr + %n%_off, %n%_klen);
      %n%_off += %n%_klen;
      if (%n%_off + 4 > <usize>%n%_len) break;
      const %n%_vlen = load<u32>(%n%_ptr + %n%_off);
      %n%_off += 4;
      const %n%_v = String.UTF8.decodeUnsafe(%n%_ptr + %n%_off, %n%_vlen);
      %n%_off += %n%_vlen;
      %n%_arr.push([%n%_k, %n%_v]);
    }
"#,
                &[("n", name)],
            ),
            vec![format!("{name}_arr")],
        ),
    }
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Host-side loop that parses the packed `u32 len | utf8 bytes` encoding out
/// of `{n}_bytes` into `let mut {out}: Vec<String>`.
fn rust_parse_packed_strings(name: &str, out: &str) -> String {
    render(
        r#"                    let mut %out%: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
                    let mut %n%_off = 0usize;
                    while %n%_off + 4 <= %n%_bytes.len() {
                        let %n%_slen = u32::from_le_bytes([%n%_bytes[%n%_off], %n%_bytes[%n%_off + 1], %n%_bytes[%n%_off + 2], %n%_bytes[%n%_off + 3]]) as usize;
                        %n%_off += 4;
                        let %n%_chunk = match %n%_bytes.get(%n%_off..%n%_off + %n%_slen) {
                            Some(b) => b,
                            None => break,
                        };
                        let %n%_s = match core::str::from_utf8(%n%_chunk) {
                            Ok(s) => s,
                            Err(_) => break,
                        };
                        %out%.push(alloc::string::String::from(%n%_s));
                        %n%_off += %n%_slen;
                    }
"#,
        &[("n", name), ("out", out)],
    )
}

/// TA-side loop that writes the ArrayBuffers collected in `__{n}_enc` into a
/// single `__{n}_buf` using the packed `u32 len | utf8 bytes` encoding.
/// Expects `__{n}_total` to hold the total byte size.
fn ts_pack_strings(name: &str) -> String {
    render(
        r#"    const __%n%_buf = new ArrayBuffer(__%n%_total);
    let __%n%_off = changetype<usize>(__%n%_buf);
    for (let __%n%_i = 0; __%n%_i < __%n%_enc.length; __%n%_i++) {
      const __%n%_b = __%n%_enc[__%n%_i];
      store<u32>(__%n%_off, __%n%_b.byteLength);
      __%n%_off += 4;
      memory.copy(__%n%_off, changetype<usize>(__%n%_b), __%n%_b.byteLength);
      __%n%_off += __%n%_b.byteLength;
    }
"#,
        &[("n", name)],
    )
}

/// Host-side preamble that bounds-checks and borrows `{n}_len` bytes of guest
/// memory at `{n}_ptr` as `{n}_bytes`. Shared by String and all array types.
/// Traps the guest (`return Err(wasmi::Error)`) on invalid input, so the
/// enclosing closure must return `Result<_, wasmi::Error>`.
fn read_guest_bytes(name: &str) -> String {
    render(
        r#"                    let %n%_memory = match caller.get_export("memory") {
                        Some(wasmi::Extern::Memory(m)) => m,
                        _ => return Err(wasmi::Error::new("guest memory export missing")),
                    };
                    let %n%_start = %n%_ptr as usize;
                    let %n%_end = match %n%_start.checked_add(%n%_len as usize) {
                        Some(e) => e,
                        None => return Err(wasmi::Error::new("guest pointer overflow for `%n%`")),
                    };
                    let %n%_bytes = match %n%_memory.data(&caller).get(%n%_start..%n%_end) {
                        Some(b) => b,
                        None => return Err(wasmi::Error::new("out-of-bounds guest pointer for `%n%`")),
                    };
"#,
        &[("n", name)],
    )
}

fn rust_default_to_ts(default: &str) -> String {
    default
        .replace("Color::WHITE", "Color.WHITE")
        .replace("Color::BLACK", "Color.BLACK")
        .replace("Color::RGB", "new Color")
        .replace("StrokeAlignment::", "StrokeAlignment.")
        .replace("Font::", "Font.")
        .replace("TextAlignment::", "TextAlignment.")
        .replace("Baseline::", "Baseline.")
}
