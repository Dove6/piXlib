use std::{collections::HashMap, path::Path, slice::Iter, sync::Arc};

use super::CnvScript;

#[derive(Debug, Clone, Default)]
pub struct ScriptContainer {
    vec: Vec<Arc<CnvScript>>,
    map: HashMap<Arc<Path>, Arc<CnvScript>>,
}

impl ScriptContainer {
    pub fn get_root_script(&self) -> Option<Arc<CnvScript>> {
        self.vec.get(0).cloned()
    }

    pub fn get_script(&self, path: &Path) -> Option<Arc<CnvScript>> {
        self.map.get(path).cloned()
    }

    pub fn get_script_at(&self, index: usize) -> Option<Arc<CnvScript>> {
        self.vec.get(index).cloned()
    }

    pub fn iter(&self) -> Iter<Arc<CnvScript>> {
        self.vec.iter()
    }

    pub fn remove_script(&mut self, path: &Path) -> Result<(), ()> {
        let Some(index) = self.vec.iter().position(|s| *s.path == *path) else {
            return Err(());
        };
        self.remove_script_at(index)
    }

    pub fn remove_script_at(&mut self, index: usize) -> Result<(), ()> {
        for script in self.vec.drain(index..) {
            self.map.remove(&script.path);
        }
        Ok(())
    }

    pub fn remove_all_scripts(&mut self) {
        self.vec.clear();
        self.map.clear();
    }

    pub fn push_script(&mut self, script: Arc<CnvScript>) -> Result<(), ()> {
        self.map.insert(script.path.clone(), script.clone());
        self.vec.push(script);
        Ok(())
    }
}
