/// All bindings discovered for one service trait.
pub struct ServiceDef {
    pub name: String,
    pub bindings: Vec<BindingDef>,
}

/// One WASM host function binding derived from a trait method.
pub struct BindingDef {
    pub executor_method: String, // "execute_rect" — also used as the WASM export name
    pub builder_name: String,    // "rect" — used for TS factory method and builder class name
    pub data_type: String,       // "RectData"
    pub fields: Vec<FieldDef>,
    /// Return type inferred from the trait method signature. `None` means void.
    pub return_type: Option<ReturnType>,
}

#[derive(Clone)]
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
    Usize,
    Bool,
    Point,
    Rect,
    Color,
    Str,
    Duration,
    TimerId,
}

/// Return type of a WASM host function.
#[derive(Clone, Debug, PartialEq)]
pub enum ReturnType {
    /// Returns a `u32` (also used for pointer-sized values).
    U32,
    /// Returns a `bool` (passed over the ABI as a `u32`: 0 or 1).
    Bool,
    /// Returns an `i32`.
    I32,
}

/// All event variants discovered from the `WidgetEvent` enum.
pub struct EventsDef {
    pub variants: Vec<EventVariantDef>,
}

/// One variant of the `WidgetEvent` enum.
pub struct EventVariantDef {
    pub name: String, // "TimerInterrupt"
    pub index: usize, // 0, 1, ...
    pub fields: Vec<EventFieldDef>,
}

/// One field within an event variant.
pub struct EventFieldDef {
    pub name: String, // "timer_id"
    pub ty: FieldType,
}
