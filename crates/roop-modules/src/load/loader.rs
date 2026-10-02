use crate::{Module, ModuleError, Tree, child_dir, find_module_file};
use roop_config::Config;
use roop_syntax::Item;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};

/// Reads module files, following `mod` declarations, and the `Roop.toml`
/// roots on demand.
pub struct Loader<'c> {
    config: &'c Config,
    pub tree: Tree,
    loaded: HashMap<PathBuf, usize>,
}

impl<'c> Loader<'c> {
    pub fn new(config: &'c Config) -> Self {
        Loader {
            config,
            tree: Tree::default(),
            loaded: HashMap::new(),
        }
    }

    pub fn load_entry(&mut self, file: &Path) -> Result<(), ModuleError> {
        self.tree.entry = self.load_file(file, Vec::new(), true)?;
        Ok(())
    }

    /// Loads the root `name` declared in `Roop.toml`, once.
    pub fn load_root(&mut self, name: &str) -> Result<usize, ModuleError> {
        if let Some(&id) = self.tree.roots.get(name) {
            return Ok(id);
        }
        let dir = self
            .config
            .modules
            .get(name)
            .ok_or_else(|| ModuleError::UnknownModule(name.into()))?;
        let file = self.config.root.join(dir).join("mod.roop");
        let id = self.load_file(&file, vec![name.to_string()], false)?;
        self.tree.roots.insert(name.to_string(), id);
        Ok(id)
    }

    fn load_file(
        &mut self,
        file: &Path,
        path: Vec<String>,
        is_entry: bool,
    ) -> Result<usize, ModuleError> {
        let key = file.canonicalize().unwrap_or_else(|_| file.to_path_buf());
        if let Some(&id) = self.loaded.get(&key) {
            return Ok(id);
        }
        let shown = file.display().to_string();
        let text =
            std::fs::read_to_string(file).map_err(|e| ModuleError::Read(shown.clone(), e))?;
        let program = roop_syntax::parse(&text).map_err(|e| ModuleError::Parse(shown, e))?;
        let dir = child_dir(file, is_entry);
        let id = self.tree.modules.len();
        self.loaded.insert(key, id);
        self.tree.modules.push(Module {
            path: path.clone(),
            dir: dir.clone(),
            items: Vec::new(),
            children: BTreeMap::new(),
        });
        let mut children = BTreeMap::new();
        for item in &program.items {
            if let Item::Mod(name) = item {
                let child_file =
                    find_module_file(&dir, name).map_err(|tried| ModuleError::MissingFile {
                        module: name.clone(),
                        tried,
                    })?;
                let mut child_path = path.clone();
                child_path.push(name.clone());
                children.insert(
                    name.clone(),
                    self.load_file(&child_file, child_path, false)?,
                );
            }
        }
        let module = &mut self.tree.modules[id];
        module.items = program.items;
        module.children = children;
        Ok(id)
    }
}
