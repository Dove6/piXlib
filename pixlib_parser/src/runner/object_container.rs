use std::{collections::HashMap, slice::Iter, sync::Arc};

use super::CnvObject;

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

    pub fn remove_object(&mut self, name: &str) -> Result<(), ()> {
        let Some(index) = self.vec.iter().position(|s| *s.name == *name) else {
            return Err(());
        };
        self.remove_object_at(index)
    }

    pub fn remove_object_at(&mut self, index: usize) -> Result<(), ()> {
        let removed_object = self.vec.remove(index);
        self.map.remove(&removed_object.name);
        Ok(())
    }

    pub fn remove_all_objects(&mut self) {
        self.vec.clear();
        self.map.clear();
    }

    pub fn push_object(&mut self, object: Arc<CnvObject>) -> Result<(), ()> {
        self.map.insert(object.name.clone(), object.clone());
        self.vec.push(object);
        Ok(())
    }

    pub fn push_objects<I: Iterator<Item = Arc<CnvObject>>>(
        &mut self,
        objects: I,
    ) -> Result<(), ()> {
        for object in objects {
            self.map.insert(object.name.clone(), object.clone());
            self.vec.push(object);
        }
        Ok(())
    }
}
