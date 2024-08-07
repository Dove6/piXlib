use std::{collections::HashMap, slice::Iter, sync::Arc};

use super::{CnvObject, RunnerError, RunnerResult};

#[derive(Debug, Clone, Default)]
pub struct ObjectContainer {
    vec: Vec<Arc<CnvObject>>,
    map: HashMap<String, Arc<CnvObject>>,
}

impl ObjectContainer {
    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        self.map.get(name).cloned()
    }

    pub fn get_object_at(&self, index: usize) -> Option<Arc<CnvObject>> {
        self.vec.get(index).cloned()
    }

    pub fn iter(&self) -> Iter<Arc<CnvObject>> {
        self.vec.iter()
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn remove_object(&mut self, name: &str) -> RunnerResult<()> {
        let Some(index) = self.vec.iter().position(|s| *s.name == *name) else {
            return Err(RunnerError::ObjectNotFound {
                name: name.to_owned(),
            });
        };
        self.remove_object_at(index)
    }

    pub fn remove_object_at(&mut self, index: usize) -> RunnerResult<()> {
        let removed_object = self.vec.remove(index);
        self.map.remove(&removed_object.name);
        Ok(())
    }

    pub fn remove_all_objects(&mut self) {
        self.vec.clear();
        self.map.clear();
    }

    pub fn push_object(&mut self, object: Arc<CnvObject>) -> RunnerResult<()> {
        self.map.insert(object.name.clone(), object.clone());
        self.vec.push(object);
        Ok(())
    }

    pub fn push_objects<I: Iterator<Item = Arc<CnvObject>>>(
        &mut self,
        objects: I,
    ) -> RunnerResult<()> {
        for object in objects {
            self.map.insert(object.name.clone(), object.clone());
            self.vec.push(object);
        }
        Ok(())
    }
}
