/// All bindings discovered from one source directory (e.g. src/drawer/).
pub struct ServiceDef {
    pub name: String,
    pub bindings: Vec<BindingDef>,
}

/// One WASM host function derived from a single `*Data` struct.
pub struct BindingDef {
    pub wasm_module: String,     // "env"
    pub wasm_fn: String,         // "draw_rect"
    pub executor_method: String, // "execute_rect"
    pub data_type: String,       // "RectData"
    pub fields: Vec<FieldDef>,
}

pub struct FieldDef {
    pub name: String,
    pub ty: FieldType,
    pub required: bool,
    pub default: Option<String>, // raw string: "Color::WHITE", "true", "0"
    pub setter_name: Option<String>, // overrides field name for the TS setter method
}

#[derive(Clone, Debug, PartialEq)]
pub enum FieldType {
    U32,
    Bool,
    Point,
    Rect,
    Color,
    Str,
}
