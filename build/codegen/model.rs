use crate::codegen::type_map::Expansion;

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
    /// Fully expanded return-type data, or `None` for void.
    pub return_expansion: Option<Expansion>,
}

pub struct FieldDef {
    pub name: String,
    pub expansion: Expansion,
    pub setter_name: Option<String>, // overrides field name for the TS setter method
}

/// All event variants discovered from the `WidgetEvent` enum.
pub struct EventsDef {
    pub variants: Vec<EventVariantDef>,
}

/// One variant of the `WidgetEvent` enum.
pub struct EventVariantDef {
    pub name: String,
    pub index: usize,
    pub fields: Vec<EventFieldDef>,
}

/// One field within an event variant.
pub struct EventFieldDef {
    pub name: String,
    pub expansion: Expansion,
}
