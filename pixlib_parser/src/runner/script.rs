use std::{cell::RefCell, sync::Arc};

use super::{containers::ObjectContainer, path::ScenePath, CnvObject, CnvRunner};

#[derive(Clone)]
pub struct CnvScript {
    pub runner: Arc<CnvRunner>,
    pub path: ScenePath,
    pub objects: RefCell<ObjectContainer>,
    pub source_kind: ScriptSource,
    pub parent_object: Option<Arc<CnvObject>>,
}

impl core::fmt::Debug for CnvScript {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CnvScript")
            .field(
                "runner",
                &format!(
                    "CnvRunner with {} scripts loaded",
                    &self.runner.scripts.borrow().len()
                ),
            )
            .field(
                "objects",
                &self
                    .objects
                    .borrow()
                    .iter()
                    .map(|o| o.name.clone())
                    .collect::<Vec<_>>(),
            )
            .field("source_kind", &self.source_kind)
            .field("path", &self.path)
            .field(
                "parent_object",
                &self.parent_object.as_ref().map(|o| o.name.clone()),
            )
            .finish()
    }
}

impl CnvScript {
    pub fn new(
        runner: Arc<CnvRunner>,
        path: ScenePath,
        parent_object: Option<Arc<CnvObject>>,
        source_kind: ScriptSource,
    ) -> Self {
        Self {
            runner,
            path,
            parent_object,
            source_kind,
            objects: RefCell::new(ObjectContainer::default()),
        }
    }

    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        self.objects.borrow().get_object(name).clone()
    }

    pub fn get_object_at(&self, index: usize) -> Option<Arc<CnvObject>> {
        self.objects.borrow().get_object_at(index)
    }

    pub fn find_object(&self, predicate: &impl Fn(&CnvObject) -> bool) -> Option<Arc<CnvObject>> {
        self.objects.borrow().find_object(predicate)
    }

    pub fn find_objects(
        &self,
        predicate: impl Fn(&CnvObject) -> bool,
        buffer: &mut Vec<Arc<CnvObject>>,
    ) {
        buffer.clear();
        for object in self.objects.borrow().iter() {
            if predicate(object) {
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
