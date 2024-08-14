use std::{cell::RefCell, collections::VecDeque, sync::Arc};

#[derive(Debug, Clone)]
pub struct InternalEvent {
    pub object: Arc<CnvObject>,
    pub callable: CallableIdentifierOwned,
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
    MovedTo { x: isize, y: isize },
    LeftButtonPressed,
    LeftButtonReleased,
    RightButtonPressed,
    RightButtonReleased,
}

pub use keyboard_types::Code as KeyboardKey;

use super::{common::SoundFileData, path::ScenePath, CallableIdentifierOwned, CnvObject, CnvValue};

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
    ScriptLoaded { path: ScenePath },
    ScriptUnloaded { path: ScenePath },
}

#[derive(Debug, Clone)]
pub enum FileEvent {
    FileRead { path: ScenePath },
    FileWritten { path: ScenePath },
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
    SoundStarted(SoundFileData),
    SoundPaused(SoundFileData),
    SoundStopped(SoundFileData),
}

#[derive(Debug, Clone)]
pub enum GraphicsEvent {
    GraphicsHidden,
    GraphicsShown,
    GraphicsLoaded,
    GraphicsFlipped,
    FrameChanged,
}
