pub mod events;
pub mod model;
pub mod parser;
pub mod rust_wasm;
pub mod type_map;
pub mod typescript;

/// Substitute `%key%` placeholders in `template` with the given values.
/// Using `%` delimiters means generated code braces never need escaping.
pub fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut s = template.to_string();
    for (k, v) in vars {
        s = s.replace(&format!("%{k}%"), v);
    }
    s
}
