use std::{collections::HashMap, path::Path, slice::Iter, sync::Arc};

use super::{CnvScript, ScriptSource};

#[derive(Debug, Clone, Default)]
pub struct ScriptContainer {
    vec: Vec<Arc<CnvScript>>,
    map: HashMap<Arc<Path>, Arc<CnvScript>>,
    application_script: Option<Arc<CnvScript>>,
    episode_script: Option<Arc<CnvScript>>,
    scene_script: Option<Arc<CnvScript>>,
}

impl ScriptContainer {
    pub fn get_root_script(&self) -> Option<Arc<CnvScript>> {
        self.vec.get(0).cloned()
    }

    pub fn get_application_script(&self) -> Option<Arc<CnvScript>> {
        self.application_script.as_ref().map(Arc::clone)
    }

    pub fn get_episode_script(&self) -> Option<Arc<CnvScript>> {
        self.episode_script.as_ref().map(Arc::clone)
    }

    pub fn get_scene_script(&self) -> Option<Arc<CnvScript>> {
        self.scene_script.as_ref().map(Arc::clone)
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

    pub fn remove_scene_script(&mut self) -> Result<Option<()>, ()> {
        let Some(ref current_scene) = self.scene_script else {
            return Ok(None);
        };
        let Some(index) = self.vec.iter().position(|s| s.path == current_scene.path) else {
            panic!("Inconsistent state detected.");
        };
        self.remove_script_at(index).map(|_| Some(()))
    }

    pub fn remove_episode_script(&mut self) -> Result<Option<()>, ()> {
        let Some(ref current_episode) = self.episode_script else {
            return Ok(None);
        };
        let Some(index) = self.vec.iter().position(|s| s.path == current_episode.path) else {
            panic!("Inconsistent state detected.");
        };
        self.remove_script_at(index).map(|_| Some(()))
    }

    pub fn remove_application_script(&mut self) -> Result<Option<()>, ()> {
        let Some(ref current_application) = self.application_script else {
            return Ok(None);
        };
        let Some(index) = self
            .vec
            .iter()
            .position(|s| s.path == current_application.path)
        else {
            panic!("Inconsistent state detected.");
        };
        self.remove_script_at(index).map(|_| Some(()))
    }

    pub fn remove_all_scripts(&mut self) {
        self.vec.clear();
        self.map.clear();
    }

    pub fn push_script(&mut self, script: Arc<CnvScript>) -> Result<(), ()> {
        match script.source_kind {
            ScriptSource::Root if !self.vec.is_empty() => return Err(()),
            ScriptSource::Application if self.application_script.is_some() => return Err(()),
            ScriptSource::Application => self.application_script = Some(Arc::clone(&script)),
            ScriptSource::Episode if self.episode_script.is_some() => return Err(()),
            ScriptSource::Episode => self.episode_script = Some(Arc::clone(&script)),
            ScriptSource::Scene if self.scene_script.is_some() => return Err(()),
            ScriptSource::Scene => self.scene_script = Some(Arc::clone(&script)),
            _ => {}
        }
        self.map.insert(script.path.clone(), script.clone());
        self.vec.push(script);
        Ok(())
    }
}
