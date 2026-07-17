use crate::codegen::model::{FieldType, ReturnType};

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
        FieldType::U32 | FieldType::Usize => WasmExpansion {
            params: vec![name.to_string()],
            construct: format!("{name} as _"),
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
            construct: format!("crate::drawer::types::Color::Rgb({name}_r as u8, {name}_g as u8, {name}_b as u8)"),
        },
        FieldType::Point => WasmExpansion {
            params: vec![format!("{name}_x"), format!("{name}_y")],
            construct: format!("crate::drawer::types::Point {{ x: {name}_x, y: {name}_y }}"),
        },
        FieldType::Rect => WasmExpansion {
            // Rect fields use short names since they're typically the dominant required field
            params: vec![
                "x".to_string(),
                "y".to_string(),
                "w".to_string(),
                "h".to_string(),
            ],
            construct: "crate::drawer::types::Rect { origin: crate::drawer::types::Point { x, y }, size: crate::drawer::types::Size { width: w, height: h } }"
                .to_string(),
        },
        FieldType::Str => WasmExpansion {
            params: vec![format!("{name}_ptr"), format!("{name}_len")],
            // Construct expression is a sentinel — the Rust generator emits the memory-read
            // block separately and binds it to a variable named `name`.
            construct: format!("__str_{name}"),
        },
        FieldType::Duration => WasmExpansion {
            params: vec![name.to_string()],
            construct: format!("embassy_time::Duration::from_ticks({name} as u64)"),
        },
        FieldType::TimerId => WasmExpansion {
            params: vec![name.to_string()],
            construct: format!("crate::timer::TimerId({name})"),
        },
    }
}

pub fn expand_ts(name: &str, ty: &FieldType, required: bool, default: Option<&str>) -> TsExpansion {
    let ts_default = default.map(rust_default_to_ts);

    match ty {
        FieldType::U32 | FieldType::Usize => {
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
                TsExpansion {
                    ctor_params: vec![(name.to_string(), "Point".to_string())],
                    fields: vec![(format!("_{name}"), "Point".to_string(), None)],
                    call_args: vec![format!("this._{name}.x"), format!("this._{name}.y")],
                }
            } else {
                TsExpansion {
                    ctor_params: vec![],
                    fields: vec![(
                        format!("_{name}"),
                        "Point".to_string(),
                        Some(ts_default.unwrap_or_else(|| "new Point(0, 0)".to_string())),
                    )],
                    call_args: vec![format!("this._{name}.x"), format!("this._{name}.y")],
                }
            }
        }

        FieldType::Rect => {
            if required {
                TsExpansion {
                    ctor_params: vec![(name.to_string(), "Rect".to_string())],
                    fields: vec![(format!("_{name}"), "Rect".to_string(), None)],
                    call_args: vec![
                        format!("this._{name}.x"),
                        format!("this._{name}.y"),
                        format!("this._{name}.width"),
                        format!("this._{name}.height"),
                    ],
                }
            } else {
                TsExpansion {
                    ctor_params: vec![],
                    fields: vec![(
                        format!("_{name}"),
                        "Rect".to_string(),
                        Some(ts_default.unwrap_or_else(|| "new Rect(0, 0, 0, 0)".to_string())),
                    )],
                    call_args: vec![
                        format!("this._{name}.x"),
                        format!("this._{name}.y"),
                        format!("this._{name}.width"),
                        format!("this._{name}.height"),
                    ],
                }
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

        FieldType::Duration => {
            if required {
                TsExpansion {
                    ctor_params: vec![(name.to_string(), "Duration".to_string())],
                    fields: vec![(format!("_{name}"), "Duration".to_string(), None)],
                    call_args: vec![format!("this._{name}.ticks")],
                }
            } else {
                TsExpansion {
                    ctor_params: vec![],
                    fields: vec![(
                        format!("_{name}"),
                        "Duration".to_string(),
                        Some(ts_default.unwrap_or_else(|| "new Duration(0)".to_string())),
                    )],
                    call_args: vec![format!("this._{name}.ticks")],
                }
            }
        }

        FieldType::TimerId => {
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
    }
}

/// The Rust primitive type used for a return value in the Wasmi closure signature.
/// Bool uses `u32` since WASM has no bool type.
pub fn rust_return_type(rt: &ReturnType) -> &'static str {
    match rt {
        ReturnType::U32 | ReturnType::Bool => "u32",
        ReturnType::I32 => "i32",
    }
}

/// A suffix cast applied after the executor method call to coerce its Rust return
/// type to the WASM primitive. Empty string means no cast is needed.
pub fn rust_return_cast(rt: &ReturnType) -> &'static str {
    match rt {
        ReturnType::U32 | ReturnType::I32 => "",
        ReturnType::Bool => " as u32",
    }
}

/// The AssemblyScript / TypeScript type for a return value.
pub fn ts_return_type(rt: &ReturnType) -> &'static str {
    match rt {
        ReturnType::U32 => "u32",
        ReturnType::Bool => "bool",
        ReturnType::I32 => "i32",
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
