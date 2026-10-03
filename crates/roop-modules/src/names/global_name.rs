/// The program-wide name of `name` defined in the module at `path`. The entry
/// module has an empty path and keeps its names.
pub fn global_name(path: &[String], name: &str) -> String {
    if path.is_empty() {
        name.to_string()
    } else {
        format!("{}__{name}", path.join("__"))
    }
}
