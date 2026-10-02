use roop_syntax::FnDef;
use std::collections::HashMap;

/// The generic functions, and the instances asked for so far.
pub struct Instances<'p> {
    pub defs: HashMap<&'p str, &'p FnDef>,
    made: HashMap<(String, Vec<i64>), String>,
    pub queue: Vec<(String, Vec<i64>, String)>,
}

impl<'p> Instances<'p> {
    pub fn new(defs: HashMap<&'p str, &'p FnDef>) -> Self {
        Instances {
            defs,
            made: HashMap::new(),
            queue: Vec::new(),
        }
    }

    /// The name of `callee` instantiated at `lengths`, queued once.
    pub fn request(&mut self, callee: &str, lengths: Vec<i64>) -> String {
        let key = (callee.to_string(), lengths);
        if let Some(name) = self.made.get(&key) {
            return name.clone();
        }
        let suffix: Vec<String> = key.1.iter().map(i64::to_string).collect();
        let name = format!("{callee}__{}", suffix.join("_"));
        self.made.insert(key.clone(), name.clone());
        self.queue.push((key.0, key.1, name.clone()));
        name
    }
}
