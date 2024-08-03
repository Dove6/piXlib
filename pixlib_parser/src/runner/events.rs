use std::{cell::RefCell, path::Path, sync::Arc};

#[derive(Debug, Clone, Default)]
pub struct IncomingEvents {
    pub timer: RefCell<Vec<TimerEvent>>,
    pub mouse: RefCell<Vec<MouseEvent>>,
    pub keyboard: RefCell<Vec<KeyboardEvent>>,
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

#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    KeyPressed,
}

#[derive(Debug, Clone, Default)]
pub struct OutgoingEvents {
    pub script: RefCell<Vec<ScriptEvent>>,
    pub file: RefCell<Vec<FileEvent>>,
    pub object: RefCell<Vec<ObjectEvent>>,
    pub app: RefCell<Vec<ApplicationEvent>>,
    pub sound: RefCell<Vec<SoundEvent>>,
    pub graphics: RefCell<Vec<GraphicsEvent>>,
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
