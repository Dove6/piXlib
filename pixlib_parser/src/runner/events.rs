use std::{cell::RefCell, collections::VecDeque, path::Path, sync::Arc};

#[derive(Debug, Clone)]
pub struct InternalEvent {
    pub script_path: Arc<Path>,
    pub object_name: String,
    pub event_name: String,
    pub arguments: Vec<CnvValue>,
}

#[derive(Debug, Clone, Default)]
pub struct IncomingEvents {
    pub timer: RefCell<VecDeque<TimerEvent>>,
    pub mouse: RefCell<VecDeque<MouseEvent>>,
    pub keyboard: RefCell<VecDeque<KeyboardEvent>>,
}

#[derive(Debug, Clone)]
pub enum TimerEvent {
    Elapsed { seconds: f64 },
}

#[derive(Debug, Clone)]
pub enum MouseEvent {
    LeftButtonPressed { x: u32, y: u32 },
    RightButtonPressed { x: u32, y: u32 },
}

pub use keyboard_types::Code as KeyboardKey;

use super::CnvValue;

#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    KeyPressed { key_code: keyboard_types::Code },
}

#[derive(Debug, Clone, Default)]
pub struct OutgoingEvents {
    pub script: RefCell<VecDeque<ScriptEvent>>,
    pub file: RefCell<VecDeque<FileEvent>>,
    pub object: RefCell<VecDeque<ObjectEvent>>,
    pub app: RefCell<VecDeque<ApplicationEvent>>,
    pub sound: RefCell<VecDeque<SoundEvent>>,
    pub graphics: RefCell<VecDeque<GraphicsEvent>>,
}

#[derive(Debug, Clone)]
pub enum ScriptEvent {
    ScriptLoaded { path: Arc<Path> },
    ScriptUnloaded { path: Arc<Path> },
}

#[derive(Debug, Clone)]
pub enum FileEvent {
    FileRead { path: Arc<Path> },
    FileWritten { path: Arc<Path> },
}

#[derive(Debug, Clone)]
pub enum ObjectEvent {
    ObjectCreated { name: String },
}

#[derive(Debug, Clone)]
pub enum ApplicationEvent {
    ApplicationExited,
}

#[derive(Debug, Clone)]
pub enum SoundEvent {
    SoundStarted,
    SoundPaused,
    SoundStopped,
}

#[derive(Debug, Clone)]
pub enum GraphicsEvent {
    GraphicsHidden,
    GraphicsShown,
    GraphicsLoaded,
    GraphicsFlipped,
    FrameChanged,
}
