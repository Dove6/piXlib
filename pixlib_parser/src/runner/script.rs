use std::{cell::RefCell, path::Path, sync::Arc};

use crate::classes::CnvObject;

use super::{object_container::ObjectContainer, CnvRunner};

#[derive(Debug, Clone)]
pub struct CnvScript {
    pub runner: Arc<CnvRunner>,
    pub objects: RefCell<ObjectContainer>,
    pub source_kind: ScriptSource,
    pub path: Arc<Path>,
    pub parent_path: Option<Arc<Path>>,
}

impl CnvScript {
    pub fn new(
        runner: Arc<CnvRunner>,
        path: Arc<Path>,
        parent_path: Option<Arc<Path>>,
        source_kind: ScriptSource,
    ) -> Self {
        Self {
            runner,
            path,
            parent_path,
            source_kind,
            objects: RefCell::new(ObjectContainer::default()),
        }
    }

    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        for object in self.objects.borrow().iter() {
            if object.name == name {
                return Some(Arc::clone(object));
            }
        }
        None
    }

    pub fn get_object_at(&self, index: usize) -> Option<Arc<CnvObject>> {
        self.objects.borrow().get_object_at(index)
    }

    pub fn find_objects(
        &self,
        predicate: impl Fn(&CnvObject) -> bool,
        buffer: &mut Vec<Arc<CnvObject>>,
    ) {
        buffer.clear();
        for object in self.objects.borrow().iter() {
            if predicate(&object) {
                buffer.push(Arc::clone(object));
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptSource {
    Root,
    Application,
    Episode,
    Scene,
    CnvLoader,
}
