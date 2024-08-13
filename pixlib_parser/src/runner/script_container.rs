use std::{
    collections::{HashMap, VecDeque},
    slice::Iter,
    sync::Arc,
};

use super::{path::ScenePath, CnvScript, RunnerError, RunnerResult, ScriptSource};

#[derive(Debug, Clone, Default)]
pub struct ScriptContainer {
    vec: Vec<Arc<CnvScript>>,
    map: HashMap<ScenePath, Arc<CnvScript>>,
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

    pub fn get_script(&self, path: &ScenePath) -> Option<Arc<CnvScript>> {
        self.map.get(path).cloned()
    }

    pub fn get_script_at(&self, index: usize) -> Option<Arc<CnvScript>> {
        self.vec.get(index).cloned()
    }

    pub fn iter(&self) -> Iter<Arc<CnvScript>> {
        self.vec.iter()
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn remove_script(&mut self, path: &ScenePath) -> RunnerResult<()> {
        let Some(index) = self.vec.iter().position(|s| s.path == *path) else {
            return Err(RunnerError::ScriptNotFound {
                path: path.to_string(),
            });
        };
        self.remove_script_at(index)
    }

    pub fn remove_script_at(&mut self, index: usize) -> RunnerResult<()> {
        let mut to_remove = VecDeque::new();
        to_remove.push_back(self.vec.remove(index));
        while let Some(script) = to_remove.pop_front() {
            if self
                .application_script
                .as_ref()
                .is_some_and(|s| s.path == script.path)
            {
                self.application_script = None;
            }
            if self
                .episode_script
                .as_ref()
                .is_some_and(|s| s.path == script.path)
            {
                self.episode_script = None;
            }
            if self
                .scene_script
                .as_ref()
                .is_some_and(|s| s.path == script.path)
            {
                self.scene_script = None;
            }
            self.map.remove(&script.path);
            for child in self.vec.iter().filter(|s| {
                s.parent_object
                    .as_ref()
                    .is_some_and(|o| o.parent.path == script.path)
            }) {
                to_remove.push_back(child.clone());
            }
        }
        Ok(())
    }

    pub fn remove_scene_script(&mut self) -> RunnerResult<Option<()>> {
        let Some(ref current_scene) = self.scene_script else {
            return Ok(None);
        };
        let Some(index) = self.vec.iter().position(|s| s.path == current_scene.path) else {
            panic!("Inconsistent state detected.");
        };
        self.remove_script_at(index).map(|_| Some(()))
    }

    pub fn remove_episode_script(&mut self) -> RunnerResult<Option<()>> {
        let Some(ref current_episode) = self.episode_script else {
            return Ok(None);
        };
        let Some(index) = self.vec.iter().position(|s| s.path == current_episode.path) else {
            panic!("Inconsistent state detected.");
        };
        self.remove_script_at(index).map(|_| Some(()))
    }

    pub fn remove_application_script(&mut self) -> RunnerResult<Option<()>> {
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
        self.scene_script = None;
        self.episode_script = None;
        self.application_script = None;
    }

    pub fn push_script(&mut self, script: Arc<CnvScript>) -> RunnerResult<()> {
        match script.source_kind {
            ScriptSource::Root if !self.vec.is_empty() => {
                return Err(RunnerError::RootScriptAlreadyLoaded)
            }
            ScriptSource::Application if self.application_script.is_some() => {
                return Err(RunnerError::ApplicationScriptAlreadyLoaded)
            }
            ScriptSource::Application => self.application_script = Some(Arc::clone(&script)),
            ScriptSource::Episode if self.episode_script.is_some() => {
                return Err(RunnerError::EpisodeScriptAlreadyLoaded)
            }
            ScriptSource::Episode => self.episode_script = Some(Arc::clone(&script)),
            ScriptSource::Scene if self.scene_script.is_some() => {
                return Err(RunnerError::SceneScriptAlreadyLoaded)
            }
            ScriptSource::Scene => self.scene_script = Some(Arc::clone(&script)),
            _ => {}
        }
        self.map.insert(script.path.clone(), script.clone());
        self.vec.push(script);
        Ok(())
    }
}
