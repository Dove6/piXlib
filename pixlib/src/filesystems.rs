use std::{
    ops::Deref,
    sync::{Arc, RwLock},
};

use bevy::{asset::Handle, ecs::system::Resource};
use pixlib_parser::runner::FileSystem;

use crate::plugins::ui_plugin::Blob;

#[derive(Default, Debug)]
pub struct LayeredFileSystem {
    layers: Vec<Arc<RwLock<dyn FileSystem>>>,
}

impl FileSystem for LayeredFileSystem {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Arc<Vec<u8>>> {
        for filesystem in self.layers.iter().rev() {
            match filesystem.write().unwrap().read_file(filename) {
                Ok(v) => return Ok(v),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        for filesystem in self.layers.iter().rev() {
            match filesystem.write().unwrap().write_file(filename, data) {
                Err(e) if e.kind() == std::io::ErrorKind::Unsupported => continue,
                Err(e) => return Err(e),
                _ => return Ok(()),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

impl LayeredFileSystem {
    pub fn new(main: Arc<RwLock<dyn FileSystem>>) -> Self {
        Self { layers: vec![main] }
    }

    pub fn set_main(&mut self, main: Arc<RwLock<dyn FileSystem>>, clear_layers: bool) {
        if !self.layers.is_empty() {
            self.layers.remove(0);
        }
        if clear_layers {
            self.layers.clear();
        }
        self.layers.insert(0, main);
    }

    pub fn push_layer(&mut self, fs: Arc<RwLock<dyn FileSystem>>) {
        self.layers.push(fs);
    }

    pub fn pop_layer(&mut self) -> Option<Arc<RwLock<dyn FileSystem>>> {
        if self.layers.len() > 1 {
            self.layers.pop()
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct PendingHandle {
    handle: Handle<Blob>,
    is_main: bool,
    clear_layers_on_insert: bool,
}

impl PendingHandle {
    pub fn new(handle: Handle<Blob>, is_main: bool, clear_layers_on_insert: bool) -> Self {
        Self {
            handle,
            is_main,
            clear_layers_on_insert,
        }
    }

    pub fn is_main(&self) -> bool {
        self.is_main
    }
}

impl Deref for PendingHandle {
    type Target = Handle<Blob>;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

#[derive(Resource, Default)]
pub struct FileSystemResource {
    pending_handles: Vec<PendingHandle>,
    filesystem: Arc<RwLock<LayeredFileSystem>>,
}

impl FileSystemResource {
    pub fn new(filesystem: Arc<RwLock<LayeredFileSystem>>) -> Self {
        Self {
            filesystem,
            pending_handles: Vec::new(),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.pending_handles.is_empty()
    }

    pub fn insert_handle(&mut self, handle: PendingHandle) {
        self.pending_handles.push(handle);
    }

    pub fn get_pending_handle(&self) -> Option<PendingHandle> {
        self.pending_handles.first().cloned()
    }

    pub fn set_as_failed(&mut self, handle: &Handle<Blob>) -> std::io::Result<()> {
        if !self.pending_handles.first().is_some_and(|h| **h == *handle) {
            return Err(std::io::Error::other("Unexpected handle"));
        };
        self.pending_handles.remove(0);
        Ok(())
    }

    pub fn insert_loaded(
        &mut self,
        handle: &Handle<Blob>,
        filesystem: Arc<RwLock<dyn FileSystem>>,
    ) -> std::io::Result<()> {
        if !self.pending_handles.first().is_some_and(|h| **h == *handle) {
            return Err(std::io::Error::other("Unexpected handle"));
        };
        let pending_handle = self.pending_handles.remove(0);
        if pending_handle.is_main {
            self.filesystem
                .write()
                .unwrap()
                .set_main(filesystem, pending_handle.clear_layers_on_insert);
        } else {
            self.filesystem.write().unwrap().push_layer(filesystem);
        }
        Ok(())
    }
}

impl Deref for FileSystemResource {
    type Target = Arc<RwLock<LayeredFileSystem>>;

    fn deref(&self) -> &Self::Target {
        &self.filesystem
    }
}
