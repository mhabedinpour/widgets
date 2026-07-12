use crate::codegen::model::FieldType;

/// A field expanded into flat WASM u32 parameters and a Rust construction expression.
pub struct WasmExpansion {
    /// Flat parameter names for the WASM closure signature, e.g. ["x", "y", "w", "h"]
    pub params: Vec<String>,
    /// Rust expression that constructs the field value from those params,
    /// e.g. `Rect { origin: Point { x, y }, size: Size { width: w, height: h } }`
    pub construct: String,
}

/// A field expanded for the TypeScript builder.
pub struct TsExpansion {
    /// Constructor parameters (name, ts_type) — only for required fields
    pub ctor_params: Vec<(String, String)>,
    /// Private field declarations (name, ts_type, default_expr)
    pub fields: Vec<(String, String, Option<String>)>,
    /// Expression(s) to pass this field to the env.* call, in order
    pub call_args: Vec<String>,
}

pub fn expand_wasm(name: &str, ty: &FieldType) -> WasmExpansion {
    match ty {
        FieldType::U32 => WasmExpansion {
            params: vec![name.to_string()],
            construct: name.to_string(),
        },
        FieldType::Bool => WasmExpansion {
            params: vec![name.to_string()],
            construct: format!("{name} != 0"),
        },
        FieldType::Color => WasmExpansion {
            params: vec![
                format!("{name}_r"),
                format!("{name}_g"),
                format!("{name}_b"),
            ],
            construct: format!("Color::Rgb({name}_r as u8, {name}_g as u8, {name}_b as u8)"),
        },
        FieldType::Point => WasmExpansion {
            params: vec![format!("{name}_x"), format!("{name}_y")],
            construct: format!("Point {{ x: {name}_x, y: {name}_y }}"),
        },
        FieldType::Rect => WasmExpansion {
            // Rect fields use short names since they're typically the dominant required field
            params: vec![
                "x".to_string(),
                "y".to_string(),
                "w".to_string(),
                "h".to_string(),
            ],
            construct: "Rect { origin: Point { x, y }, size: Size { width: w, height: h } }"
                .to_string(),
        },
        FieldType::Str => WasmExpansion {
            params: vec![format!("{name}_ptr"), format!("{name}_len")],
            // Construct expression is a sentinel — the Rust generator emits the memory-read
            // block separately and binds it to a variable named `name`.
            construct: format!("__str_{name}"),
        },
    }
}

pub fn expand_ts(name: &str, ty: &FieldType, required: bool, default: Option<&str>) -> TsExpansion {
    let ts_default = default.map(rust_default_to_ts);

    match ty {
        FieldType::U32 => {
            if required {
                TsExpansion {
                    ctor_params: vec![(name.to_string(), "u32".to_string())],
                    fields: vec![(format!("_{name}"), "u32".to_string(), None)],
                    call_args: vec![format!("this._{name}")],
                }
            } else {
                TsExpansion {
                    ctor_params: vec![],
                    fields: vec![(
                        format!("_{name}"),
                        "u32".to_string(),
                        Some(ts_default.unwrap_or_else(|| "0".to_string())),
                    )],
                    call_args: vec![format!("this._{name}")],
                }
            }
        }

        FieldType::Bool => {
            if required {
                TsExpansion {
                    ctor_params: vec![(name.to_string(), "bool".to_string())],
                    fields: vec![(format!("_{name}"), "bool".to_string(), None)],
                    call_args: vec![format!("this._{name} ? 1 : 0")],
                }
            } else {
                TsExpansion {
                    ctor_params: vec![],
                    fields: vec![(
                        format!("_{name}"),
                        "bool".to_string(),
                        Some(ts_default.unwrap_or_else(|| "false".to_string())),
                    )],
                    call_args: vec![format!("this._{name} ? 1 : 0")],
                }
            }
        }

        FieldType::Color => {
            // Color is always optional (never a required ctor arg)
            TsExpansion {
                ctor_params: vec![],
                fields: vec![(
                    format!("_{name}"),
                    "Color".to_string(),
                    Some(ts_default.unwrap_or_else(|| "Color.WHITE".to_string())),
                )],
                call_args: vec![
                    format!("this._{name}.r"),
                    format!("this._{name}.g"),
                    format!("this._{name}.b"),
                ],
            }
        }

        FieldType::Point => {
            if required {
                // Required Points expand to two flat u32 ctor args
                TsExpansion {
                    ctor_params: vec![
                        (format!("{name}_x"), "u32".to_string()),
                        (format!("{name}_y"), "u32".to_string()),
                    ],
                    fields: vec![
                        (format!("_{name}_x"), "u32".to_string(), None),
                        (format!("_{name}_y"), "u32".to_string(), None),
                    ],
                    call_args: vec![format!("this._{name}_x"), format!("this._{name}_y")],
                }
            } else {
                TsExpansion {
                    ctor_params: vec![],
                    fields: vec![
                        (
                            format!("_{name}_x"),
                            "u32".to_string(),
                            Some("0".to_string()),
                        ),
                        (
                            format!("_{name}_y"),
                            "u32".to_string(),
                            Some("0".to_string()),
                        ),
                    ],
                    call_args: vec![format!("this._{name}_x"), format!("this._{name}_y")],
                }
            }
        }

        FieldType::Rect => {
            // Rect always required (it's the geometry of the shape)
            TsExpansion {
                ctor_params: vec![
                    ("x".to_string(), "u32".to_string()),
                    ("y".to_string(), "u32".to_string()),
                    ("w".to_string(), "u32".to_string()),
                    ("h".to_string(), "u32".to_string()),
                ],
                fields: vec![
                    ("_x".to_string(), "u32".to_string(), None),
                    ("_y".to_string(), "u32".to_string(), None),
                    ("_w".to_string(), "u32".to_string(), None),
                    ("_h".to_string(), "u32".to_string(), None),
                ],
                call_args: vec![
                    "this._x".to_string(),
                    "this._y".to_string(),
                    "this._w".to_string(),
                    "this._h".to_string(),
                ],
            }
        }

        FieldType::Str => {
            // Str is always required
            TsExpansion {
                ctor_params: vec![(name.to_string(), "string".to_string())],
                fields: vec![(format!("_{name}"), "string".to_string(), None)],
                // Matches the preamble vars emitted by the TS generator (__{name}_ptr/len)
                call_args: vec![format!("__{name}_ptr"), format!("__{name}_len")],
            }
        }
    }
}

/// Convert a Rust default expression to its TypeScript equivalent.
/// Handles the common cases: Color constants, bool, numeric literals.
fn rust_default_to_ts(default: &str) -> String {
    default
        .replace("Color::WHITE", "Color.WHITE")
        .replace("Color::BLACK", "Color.BLACK")
        .replace("Color::RGB", "new Color")
        .replace("true", "true")
        .replace("false", "false")
}
