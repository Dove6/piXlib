use std::{cell::RefCell, collections::VecDeque, fmt::Display, sync::Arc};

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
    pub multimedia: RefCell<VecDeque<MultimediaEvents>>,
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
    MiddleButtonPressed,
    MiddleButtonReleased,
    RightButtonPressed,
    RightButtonReleased,
}

pub use keyboard_types::Code as KeyboardKey;

use super::{common::SoundData, path::ScenePath, CallableIdentifierOwned, CnvObject, CnvValue};

#[derive(Debug, Clone)]
pub enum KeyboardEvent {
    KeyPressed { key_code: keyboard_types::Code },
}

#[derive(Debug, Clone)]
pub enum MultimediaEvents {
    SoundFinishedPlaying(SoundSource),
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SoundSource {
    BackgroundMusic,
    Sound {
        script_path: ScenePath,
        object_name: String,
    },
    AnimationSfx {
        script_path: ScenePath,
        object_name: String,
    },
}

#[derive(Debug, Clone)]
pub enum SoundEvent {
    SoundLoaded {
        source: SoundSource,
        sound_data: SoundData,
    },
    SoundStarted(SoundSource),
    SoundPaused(SoundSource),
    SoundResumed(SoundSource),
    SoundStopped(SoundSource),
}

impl SoundEvent {
    pub fn get_source(&self) -> &SoundSource {
        match self {
            SoundEvent::SoundLoaded { source, .. } => source,
            SoundEvent::SoundStarted(source) => source,
            SoundEvent::SoundPaused(source) => source,
            SoundEvent::SoundResumed(source) => source,
            SoundEvent::SoundStopped(source) => source,
        }
    }
}

impl Display for SoundEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SoundEvent::{}({:?})",
            match self {
                SoundEvent::SoundLoaded { .. } => "SoundLoaded",
                SoundEvent::SoundStarted(_) => "SoundStarted",
                SoundEvent::SoundPaused(_) => "SoundPaused",
                SoundEvent::SoundResumed(_) => "SoundResumed",
                SoundEvent::SoundStopped(_) => "SoundStopped",
            },
            self.get_source()
        )
    }
}

#[derive(Debug, Clone)]
pub enum GraphicsEvent {
    GraphicsHidden,
    GraphicsShown,
    GraphicsLoaded,
    GraphicsFlipped,
    FrameChanged,
}
