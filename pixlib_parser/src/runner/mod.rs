#[allow(dead_code)]
pub mod classes;
pub mod common;
mod containers;
mod content;
mod events;
mod filesystem;
mod initable;
pub mod object;
mod parsers;
mod path;
mod script;
#[cfg(test)]
#[allow(clippy::arc_with_non_send_sync)]
mod tests;
mod tree_walking;
mod value;

pub use common::{CallableIdentifier, CallableIdentifierOwned};
use containers::{ObjectContainer, ScriptContainer};
pub use content::CnvContent;
pub use events::{
    ApplicationEvent, CursorEvent, FileEvent, GraphicsEvent, InternalEvent, KeyboardEvent,
    KeyboardKey, MouseEvent, MultimediaEvents, ObjectEvent, ScriptEvent, SoundEvent, SoundSource,
    TimerEvent,
};
pub use filesystem::{FileSystem, GamePaths};
use itertools::Itertools;
pub use object::{CnvObject, ObjectBuildErrorKind, ObjectBuilderError};
pub use path::ScenePath;
pub use script::{CnvScript, ScriptSource};
use thiserror::Error;
pub use tree_walking::{CnvExpression, CnvStatement};
pub use value::CnvValue;

use std::collections::{HashSet, VecDeque};
use std::fmt::Display;
use std::sync::RwLock;
use std::{cell::RefCell, collections::HashMap, sync::Arc};

use events::{IncomingEvents, OutgoingEvents};

use crate::parser::seq_parser::SeqParserError;
use crate::{
    common::{DroppableRefMut, Issue, IssueHandler, IssueKind, IssueManager},
    parser::declarative_parser::{
        self, CnvDeclaration, DeclarativeParser, ParserFatal, ParserIssue,
    },
    scanner::parse_cnv,
};
use classes::{GeneralButton, InternalMouseEvent, Mouse};
use object::CnvObjectBuilder;

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        eprintln!("{:?}", issue);
    }
}

trait SomeWarnable {
    fn warn_if_some(&self);
}

impl<T> SomeWarnable for Option<T>
where
    T: std::fmt::Debug,
{
    fn warn_if_some(&self) {
        if self.is_some() {
            eprintln!("Unexpected value: {:?}", self.as_ref().unwrap());
        }
    }
}

#[derive(Debug, Error)]
pub enum RunnerError {
    #[error("Too many arguments (expected at most {expected_max}, got {actual})")]
    TooManyArguments { expected_max: usize, actual: usize },
    #[error("Too few arguments (expected at least {expected_min}, got {actual})")]
    TooFewArguments { expected_min: usize, actual: usize },
    #[error("Integer {actual} cannot be cast to unsigned")]
    ExpectedUnsignedInteger { actual: i32 },
    #[error("Left operand missing for object {object_name}")]
    MissingLeftOperand { object_name: String },
    #[error("Right operand missing for object {object_name}")]
    MissingRightOperand { object_name: String },
    #[error("Operator missing for object {object_name}")]
    MissingOperator { object_name: String },
    #[error("Object {name} not found")]
    ObjectNotFound { name: String },
    #[error("Expected graphics object")]
    ExpectedGraphicsObject,
    #[error("Expected sound object")]
    ExpectedSoundObject,
    #[error("Expected condition object")]
    ExpectedConditionObject,
    #[error("No animation data loaded for object {0}")]
    NoAnimationDataLoaded(String),
    #[error("No sound data loaded for object {0}")]
    NoSoundDataLoaded(String),
    #[error("No image data loaded for object {0}")]
    NoImageDataLoaded(String),
    #[error("No sequence data loaded for object {0}")]
    NoSequenceDataLoaded(String),
    #[error("Sequence object {0} is not currently playing")]
    SeqNotPlaying(String),
    #[error("Animation object {0} is not currently playing")]
    AnimationNotPlaying(String),
    #[error("Sequence object {0} is not currently sound")]
    SeqNotPlayingSound(String),
    #[error("Object {0} has not been initialized yet")]
    NotInitialized(String),
    #[error("Sequence {sequence_name} not found in object {object_name}")]
    SequenceNameNotFound {
        object_name: String,
        sequence_name: String,
    },
    #[error("Sequence #{index} not found in object {object_name}")]
    SequenceIndexNotFound { object_name: String, index: usize },
    #[error("Frame #{index} not found in sequence {sequence_name} of object {object_name}")]
    FrameIndexNotFound {
        object_name: String,
        sequence_name: String,
        index: usize,
    },
    #[error("Sprite #{index} not found in object {object_name}")]
    SpriteIndexNotFound { object_name: String, index: usize },
    #[error("Method or event handler missing on object {object_name} for callable {callable}")]
    InvalidCallable {
        object_name: String,
        callable: CallableIdentifierOwned,
    },
    #[error("Missing filename to load")]
    MissingFilenameToLoad,

    #[error("Script {path} not found")]
    ScriptNotFound { path: String },
    #[error("Root script is loaded already")]
    RootScriptAlreadyLoaded,
    #[error("Application script is loaded already")]
    ApplicationScriptAlreadyLoaded,
    #[error("Episode script is loaded already")]
    EpisodeScriptAlreadyLoaded,
    #[error("Scene script is loaded already")]
    SceneScriptAlreadyLoaded,

    #[error("Parser error: {0}")]
    ParserError(ParserFatal),
    #[error("SEQ parser error: {0}")]
    SeqParserError(SeqParserError),

    #[error("IO error: {source}")]
    IoError { source: std::io::Error },
    #[error("Object builder error: {source}")]
    ObjectBuilderError { source: ObjectBuilderError },
    #[error("Other error")]
    Other,
}

impl From<ObjectBuilderError> for RunnerError {
    fn from(value: ObjectBuilderError) -> Self {
        Self::ObjectBuilderError { source: value }
    }
}

pub type RunnerResult<T> = std::result::Result<T, RunnerError>;

#[derive(Clone)]
pub struct CnvRunner {
    pub scripts: RefCell<ScriptContainer>,
    pub events_in: IncomingEvents,
    pub events_out: OutgoingEvents,
    pub internal_events: RefCell<VecDeque<InternalEvent>>,
    pub filesystem: Arc<RwLock<dyn FileSystem>>,
    pub game_paths: Arc<GamePaths>,
    pub issue_manager: Arc<RefCell<IssueManager<RunnerIssue>>>,
    pub global_objects: RefCell<ObjectContainer>,
    pub window_rect: Rect,
    cursor_state: RefCell<CursorState>,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorState {
    pub is_visible: bool,
    pub is_pointer: bool,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            is_visible: true,
            is_pointer: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Rect {
    pub top_left_x: isize,
    pub top_left_y: isize,
    pub bottom_right_x: isize,
    pub bottom_right_y: isize,
}

impl Rect {
    pub fn from(position: (isize, isize), size: (usize, usize)) -> Self {
        Self {
            top_left_x: position.0,
            top_left_y: position.1,
            bottom_right_x: position.0 + size.0 as isize,
            bottom_right_y: position.1 + size.1 as isize,
        }
    }

    pub fn intersect(&self, other: &Self) -> Option<Self> {
        let intersection = Self {
            top_left_x: self.top_left_x.max(other.top_left_x),
            top_left_y: self.top_left_y.max(other.top_left_y),
            bottom_right_x: self.bottom_right_x.min(other.bottom_right_x),
            bottom_right_y: self.bottom_right_y.min(other.bottom_right_y),
        };
        if intersection.bottom_right_x < intersection.top_left_x
            || intersection.bottom_right_y < intersection.top_left_y
        {
            None
        } else {
            Some(intersection)
        }
    }

    pub fn has_inside(&self, x: isize, y: isize) -> bool {
        x.clamp(self.top_left_x, self.bottom_right_x) == x
            && y.clamp(self.top_left_y, self.bottom_right_y) == y
    }

    pub fn get_width(&self) -> usize {
        (self.bottom_right_x - self.top_left_x) as usize
    }

    pub fn get_height(&self) -> usize {
        (self.bottom_right_y - self.top_left_y) as usize
    }

    pub fn get_center(&self) -> (isize, isize) {
        (
            self.top_left_x + self.get_width() as isize / 2,
            self.top_left_y + self.get_height() as isize / 2,
        )
    }
}

impl From<(isize, isize, isize, isize)> for Rect {
    fn from(value: (isize, isize, isize, isize)) -> Self {
        Self {
            top_left_x: value.0,
            top_left_y: value.1,
            bottom_right_x: value.2,
            bottom_right_y: value.3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObjectIndex {
    pub script_idx: usize,
    pub object_idx: usize,
}

impl PartialOrd for ObjectIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ObjectIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.script_idx.cmp(&other.script_idx) {
            std::cmp::Ordering::Equal => self.object_idx.cmp(&other.object_idx),
            ord => ord,
        }
    }
}

struct ButtonDescriptor {
    pub priority: isize,
    pub object_index: ObjectIndex,
    pub object: Arc<CnvObject>,
    pub rect: Rect, // TODO: pixel perfect
}

impl PartialEq for ButtonDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.object_index == other.object_index
    }
}

impl Eq for ButtonDescriptor {}

impl PartialOrd for ButtonDescriptor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ButtonDescriptor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.priority.cmp(&self.priority) {
            core::cmp::Ordering::Equal => self.object_index.cmp(&other.object_index),
            ord => ord,
        }
    }
}

impl core::fmt::Debug for CnvRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CnvRunner")
            .field(
                "scripts",
                &self
                    .scripts
                    .borrow()
                    .iter()
                    .map(|o| {
                        (
                            o.parent_object.as_ref().map(|p| p.name.clone()),
                            o.path.clone(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .field("events_in", &self.events_in)
            .field("events_out", &self.events_out)
            .field("internal_events", &self.internal_events)
            .field("filesystem", &self.filesystem)
            .field("game_paths", &self.game_paths)
            .field("issue_manager", &self.issue_manager)
            .field("global_objects", &self.global_objects)
            .finish()
    }
}

#[derive(Debug, Error)]
pub enum RunnerIssue {}

impl Issue for RunnerIssue {
    fn kind(&self) -> IssueKind {
        IssueKind::Error
    }
}

#[derive(Debug, Clone)]
pub struct RunnerContext {
    pub runner: Arc<CnvRunner>,
    pub self_object: Arc<CnvObject>,
    pub current_object: Arc<CnvObject>,
    pub arguments: Vec<CnvValue>,
}

impl Display for RunnerContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RunnerContext {{ self: {}, current: {}, arguments: [{}] }}",
            self.self_object.name,
            self.current_object.name,
            self.arguments.iter().map(|v| format!("{}", v)).join(", ")
        )
    }
}

impl RunnerContext {
    pub fn new(
        runner: &Arc<CnvRunner>,
        self_object: &Arc<CnvObject>,
        current_object: &Arc<CnvObject>,
        arguments: &[CnvValue],
    ) -> Self {
        Self {
            runner: runner.clone(),
            self_object: self_object.clone(),
            current_object: current_object.clone(),
            arguments: arguments.to_owned(),
        }
    }

    pub fn new_minimal(runner: &Arc<CnvRunner>, current_object: &Arc<CnvObject>) -> Self {
        Self {
            runner: runner.clone(),
            self_object: current_object.clone(),
            current_object: current_object.clone(),
            arguments: Vec::new(),
        }
    }

    pub fn with_current_object(self, current_object: Arc<CnvObject>) -> Self {
        Self {
            current_object,
            ..self
        }
    }

    pub fn with_arguments(self, arguments: Vec<CnvValue>) -> Self {
        Self { arguments, ..self }
    }
}

#[allow(clippy::arc_with_non_send_sync)]
impl CnvRunner {
    pub fn try_new(
        filesystem: Arc<RwLock<dyn FileSystem>>,
        game_paths: Arc<GamePaths>,
        window_resolution: (usize, usize),
        issue_manager: IssueManager<RunnerIssue>,
    ) -> anyhow::Result<Arc<Self>> {
        let runner = Arc::new(Self {
            scripts: RefCell::new(ScriptContainer::default()),
            filesystem,
            events_in: IncomingEvents::default(),
            events_out: OutgoingEvents::default(),
            internal_events: RefCell::new(VecDeque::new()),
            game_paths,
            issue_manager: Arc::new(RefCell::new(issue_manager)),
            global_objects: RefCell::new(ObjectContainer::default()),
            window_rect: Rect {
                top_left_x: 0,
                top_left_y: 0,
                bottom_right_x: window_resolution.0 as isize,
                bottom_right_y: window_resolution.1 as isize,
            },
            cursor_state: RefCell::new(CursorState::default()),
        });
        let global_script = Arc::new(CnvScript::new(
            Arc::clone(&runner),
            ScenePath {
                dir_path: ".".into(),
                file_path: "__GLOBAL__".into(),
            },
            None,
            ScriptSource::Root,
        ));
        runner
            .global_objects
            .borrow_mut()
            .use_and_drop_mut::<anyhow::Result<()>>(|objects| {
                let mut range = 0usize..;
                objects
                    .push_object({
                        let mut builder = CnvObjectBuilder::new(
                            Arc::clone(&global_script),
                            "RANDOM".to_owned(),
                            range.next().unwrap(),
                        );
                        builder.add_property("TYPE".into(), "RAND".to_owned())?;
                        builder.build().unwrap()
                    })
                    .unwrap();
                objects
                    .push_object({
                        let mut builder = CnvObjectBuilder::new(
                            Arc::clone(&global_script),
                            "KEYBOARD".to_owned(),
                            range.next().unwrap(),
                        );
                        builder.add_property("TYPE".into(), "KEYBOARD".to_owned())?;
                        builder.build().unwrap()
                    })
                    .unwrap();
                objects
                    .push_object({
                        let mut builder = CnvObjectBuilder::new(
                            Arc::clone(&global_script),
                            "MOUSE".to_owned(),
                            range.next().unwrap(),
                        );
                        builder.add_property("TYPE".into(), "MOUSE".to_owned())?;
                        builder.build().unwrap()
                    })
                    .unwrap();
                objects
                    .push_object({
                        let mut builder = CnvObjectBuilder::new(
                            Arc::clone(&global_script),
                            "SYSTEM".to_owned(),
                            range.next().unwrap(),
                        );
                        builder.add_property("TYPE".into(), "SYSTEM".to_owned())?;
                        builder.build().unwrap()
                    })
                    .unwrap();
                objects
                    .push_object({
                        let mut builder = CnvObjectBuilder::new(
                            Arc::clone(&global_script),
                            "CANVAS_OBSERVER".to_owned(),
                            range.next().unwrap(),
                        );
                        builder.add_property("TYPE".into(), "CANVAS_OBSERVER".to_owned())?;
                        builder.build().unwrap()
                    })
                    .unwrap();
                objects
                    .push_object({
                        let mut builder = CnvObjectBuilder::new(
                            Arc::clone(&global_script),
                            "CANVASOBSERVER".to_owned(),
                            range.next().unwrap(),
                        );
                        builder.add_property("TYPE".into(), "CANVASOBSERVER".to_owned())?;
                        builder.build().unwrap()
                    })
                    .unwrap();
                Ok(())
            })?;
        Ok(runner)
    }

    #[allow(clippy::mutable_key_type)]
    pub fn step(self: &Arc<CnvRunner>) -> anyhow::Result<()> {
        let mut to_init = Vec::new();
        self.find_objects(|o| !*o.initialized.read().unwrap(), &mut to_init);
        for object in to_init {
            object.init(None)?;
        }
        let mut finished_animations = HashSet::new();
        self.events_in
            .timer
            .borrow_mut()
            .use_and_drop_mut::<anyhow::Result<()>>(|events| {
                while let Some(evt) = events.pop_front() {
                    match evt {
                        TimerEvent::Elapsed { seconds } => {
                            let mut buffer = Vec::new();
                            self.find_objects(
                                |o| matches!(&o.content, CnvContent::Animation(_)),
                                &mut buffer,
                            );
                            for animation_object in buffer.iter() {
                                let CnvContent::Animation(animation) = &animation_object.content
                                else {
                                    unreachable!();
                                };
                                let was_playing = animation.is_playing()?;
                                animation.step(seconds)?;
                                if was_playing && !animation.is_playing()? {
                                    finished_animations.insert(animation_object.clone());
                                }
                            }
                            self.find_objects(
                                |o| matches!(&o.content, CnvContent::Timer(_)),
                                &mut buffer,
                            );
                            for timer_object in buffer.iter() {
                                let CnvContent::Timer(ref timer) = &timer_object.content else {
                                    unreachable!();
                                };
                                timer.step(seconds)?;
                            }
                        }
                    }
                }
                Ok(())
            })?;
        let mut sequences = Vec::new();
        self.find_objects(
            |o| matches!(&o.content, CnvContent::Sequence(_)),
            &mut sequences,
        );
        for sequence in sequences.iter() {
            let CnvContent::Sequence(sequence) = &sequence.content else {
                unreachable!()
            };
            if let Some(played_animation) = sequence.get_currently_played_animation()? {
                if finished_animations.contains(&played_animation) {
                    sequence.handle_animation_finished()?;
                }
            }
        }
        self.events_in
            .mouse
            .borrow_mut()
            .use_and_drop_mut::<anyhow::Result<()>>(|events| {
                while let Some(evt) = events.pop_front() {
                    // eprintln!("Handling incoming mouse event: {:?}", evt);
                    Mouse::handle_incoming_event(evt)?;
                }
                Ok(())
            })?;
        self.events_in
            .multimedia
            .borrow_mut()
            .use_and_drop_mut::<anyhow::Result<()>>(|events| {
                while let Some(evt) = events.pop_front() {
                    match &evt {
                        MultimediaEvents::SoundFinishedPlaying(source) => {
                            match source {
                                SoundSource::BackgroundMusic => {
                                    let Some(scene_object) = self.get_current_scene() else {
                                        eprintln!("No current scene to handle event {:?}", evt);
                                        continue;
                                    };
                                    let CnvContent::Scene(ref scene) = &scene_object.content else {
                                        panic!();
                                    };
                                    scene.handle_music_finished()?;
                                }
                                SoundSource::Sound {
                                    script_path,
                                    object_name,
                                } => {
                                    let Some(sound_object) = self
                                        .get_script(script_path)
                                        .and_then(|s| s.get_object(object_name))
                                    else {
                                        eprintln!(
                                            "Object {} / {} not found for event {:?}",
                                            script_path.to_str(),
                                            object_name,
                                            evt
                                        );
                                        continue;
                                    };
                                    let CnvContent::Sound(ref sound) = &sound_object.content else {
                                        unreachable!();
                                    };
                                    sound.handle_finished()?;
                                }
                                SoundSource::Sequence {
                                    script_path,
                                    object_name,
                                } => {
                                    let Some(sequence_object) = self
                                        .get_script(script_path)
                                        .and_then(|s| s.get_object(object_name))
                                    else {
                                        eprintln!(
                                            "Object {} / {} not found for event {:?}",
                                            script_path.to_str(),
                                            object_name,
                                            evt
                                        );
                                        continue;
                                    };
                                    let CnvContent::Sequence(ref sequence) =
                                        &sequence_object.content
                                    else {
                                        unreachable!();
                                    };
                                    if sequence.is_currently_playing_sound()? {
                                        sequence.handle_sound_finished()?;
                                    }
                                }
                                SoundSource::AnimationSfx { .. } => {}
                            };
                        }
                    }
                }
                Ok(())
            })?;
        let mut enabled_buttons = Vec::new();
        self.filter_map_objects(
            |id, o| {
                let button: &dyn GeneralButton = match &o.content {
                    CnvContent::Animation(a) => a,
                    CnvContent::Button(b) => b,
                    CnvContent::Image(i) => i,
                    _ => return Ok(None),
                };
                if !button.is_enabled()? {
                    return Ok(None);
                }
                let Some(rect) = button.get_rect()? else {
                    return Ok(None);
                };
                Ok(Some(ButtonDescriptor {
                    priority: button.get_priority()?,
                    object_index: id,
                    object: o.clone(),
                    rect,
                }))
            },
            &mut enabled_buttons,
        )?;
        enabled_buttons.sort();
        let mouse_position = Mouse::get_position()?;
        let found_button_index =
            self.find_relevant_button(enabled_buttons.as_ref(), mouse_position)?;
        for (i, ButtonDescriptor { object: o, .. }) in enabled_buttons.iter().enumerate() {
            let button: &dyn GeneralButton = match &o.content {
                CnvContent::Animation(a) => a,
                CnvContent::Button(b) => b,
                CnvContent::Image(i) => i,
                _ => unreachable!(),
            };
            if found_button_index.is_some_and(|found| found == i) {
                button.handle_cursor_over()
            } else {
                button.handle_cursor_away()
            }?
        }
        if found_button_index.is_some() && !self.cursor_state.borrow().is_pointer {
            let button: Option<&dyn GeneralButton> =
                match &enabled_buttons[found_button_index.unwrap()].object.content {
                    CnvContent::Animation(a) => Some(a),
                    CnvContent::Button(b) => Some(b),
                    CnvContent::Image(i) => Some(i),
                    _ => None,
                };
            if button
                .map(|b| b.makes_cursor_pointer())
                .transpose()?
                .unwrap_or_default()
            {
                self.cursor_state.borrow_mut().is_pointer = true;
                self.events_out
                    .cursor
                    .borrow_mut()
                    .use_and_drop_mut(|events| events.push_back(CursorEvent::CursorSetToPointer));
            }
        } else if found_button_index.is_none() && self.cursor_state.borrow().is_pointer {
            self.cursor_state.borrow_mut().is_pointer = false;
            self.events_out
                .cursor
                .borrow_mut()
                .use_and_drop_mut(|events| events.push_back(CursorEvent::CursorSetToDefault));
        }
        let mut mouse_objects = Vec::new();
        self.find_objects(
            |o| matches!(&o.content, CnvContent::Mouse(_)),
            &mut mouse_objects,
        );
        Mouse::handle_outgoing_events(|mouse_event| {
            // eprintln!("Handling internal mouse event: {:?}", mouse_event);
            if let InternalMouseEvent::LeftButtonPressed { x, y } = &mouse_event {
                if let Some(button_idx) =
                    self.find_relevant_button(enabled_buttons.as_ref(), (*x, *y))?
                {
                    let button: &dyn GeneralButton =
                        match &enabled_buttons[button_idx].object.content {
                            CnvContent::Animation(a) => a,
                            CnvContent::Button(b) => b,
                            CnvContent::Image(i) => i,
                            _ => unreachable!(),
                        };
                    button.handle_lmb_pressed()?;
                }
            }
            if let InternalMouseEvent::LeftButtonReleased { x, y } = &mouse_event {
                if let Some(button_idx) =
                    self.find_relevant_button(enabled_buttons.as_ref(), (*x, *y))?
                {
                    let button: &dyn GeneralButton =
                        match &enabled_buttons[button_idx].object.content {
                            CnvContent::Animation(a) => a,
                            CnvContent::Button(b) => b,
                            CnvContent::Image(i) => i,
                            _ => unreachable!(),
                        };
                    button.handle_lmb_released()?;
                }
            }
            let callable = CallableIdentifier::Event(match mouse_event {
                InternalMouseEvent::LeftButtonPressed { .. }
                | InternalMouseEvent::MiddleButtonPressed { .. }
                | InternalMouseEvent::RightButtonPressed { .. } => "ONCLICK",
                InternalMouseEvent::LeftButtonReleased { .. }
                | InternalMouseEvent::MiddleButtonReleased { .. }
                | InternalMouseEvent::RightButtonReleased { .. } => "ONRELEASE",
                InternalMouseEvent::LeftButtonDoubleClicked { .. } => "ONDBLCLICK",
                InternalMouseEvent::MovedBy { .. } => "ONMOVE",
                _ => return Ok(()),
            });
            let arguments = match mouse_event {
                InternalMouseEvent::LeftButtonPressed { .. }
                | InternalMouseEvent::LeftButtonReleased { .. }
                | InternalMouseEvent::LeftButtonDoubleClicked { .. } => {
                    vec![CnvValue::String("LEFT".into())]
                }
                InternalMouseEvent::MiddleButtonPressed { .. }
                | InternalMouseEvent::MiddleButtonReleased { .. } => {
                    vec![CnvValue::String("MIDDLE".into())]
                }
                InternalMouseEvent::RightButtonPressed { .. }
                | InternalMouseEvent::RightButtonReleased { .. } => {
                    vec![CnvValue::String("RIGHT".into())]
                }
                _ => Vec::new(),
            };
            for mouse_object in mouse_objects.iter() {
                self.internal_events
                    .borrow_mut()
                    .use_and_drop_mut(|internal_events| {
                        internal_events.push_back(InternalEvent {
                            context: RunnerContext::new(
                                self,
                                mouse_object,
                                mouse_object,
                                &arguments,
                            ),
                            callable: callable.to_owned(),
                        })
                    });
            }
            Ok(())
        })?;
        let mut collidable = Vec::new();
        self.find_objects(
            |o| match &o.content {
                CnvContent::Animation(a) => a.does_monitor_collision().unwrap(),
                CnvContent::Image(i) => i.does_monitor_collision().unwrap(),
                _ => false,
            },
            &mut collidable,
        );
        if collidable.len() > 1 {
            for i in 0..(collidable.len() - 1) {
                for j in (i + 1)..collidable.len() {
                    let left = &collidable[i];
                    let right = &collidable[j];
                    let (left_position, left_size, left_pixel_perfect) = match &left.content {
                        CnvContent::Animation(a) => (
                            a.get_frame_position()?,
                            a.get_frame_size()?,
                            a.does_monitor_collision_pixel_perfect()?,
                        ),
                        CnvContent::Image(i) => (
                            i.get_position()?,
                            i.get_size()?,
                            i.does_monitor_collision_pixel_perfect()?,
                        ),
                        _ => unreachable!(),
                    };
                    let (right_position, right_size, right_pixel_perfect) = match &right.content {
                        CnvContent::Animation(a) => (
                            a.get_frame_position()?,
                            a.get_frame_size()?,
                            a.does_monitor_collision_pixel_perfect()?,
                        ),
                        CnvContent::Image(i) => (
                            i.get_position()?,
                            i.get_size()?,
                            i.does_monitor_collision_pixel_perfect()?,
                        ),
                        _ => unreachable!(),
                    };
                    let _pixel_perfect = left_pixel_perfect && right_pixel_perfect; // TODO: handle pixel perfect collisions
                    let left_top_left = left_position;
                    let left_bottom_right = (
                        left_position.0 + left_size.0 as isize,
                        left_position.1 + left_size.1 as isize,
                    );
                    let right_top_left = right_position;
                    let right_bottom_right = (
                        right_position.0 + right_size.0 as isize,
                        right_position.1 + right_size.1 as isize,
                    );
                    let do_collide = (right_top_left.0.clamp(left_top_left.0, left_bottom_right.0)
                        == right_top_left.0
                        || right_bottom_right
                            .0
                            .clamp(left_top_left.0, left_bottom_right.0)
                            == right_bottom_right.0)
                        && (right_top_left.1.clamp(left_top_left.1, left_bottom_right.1)
                            == right_top_left.1
                            || right_bottom_right
                                .1
                                .clamp(left_top_left.1, left_bottom_right.1)
                                == right_bottom_right.1);
                    if do_collide {
                        let callable = CallableIdentifier::Event("ONCOLLISION");
                        self.internal_events
                            .borrow_mut()
                            .use_and_drop_mut(|events| {
                                events.push_back(InternalEvent {
                                    context: RunnerContext::new(
                                        self,
                                        left,
                                        left,
                                        &[CnvValue::String(right.name.clone())],
                                    ),
                                    callable: callable.to_owned(),
                                });
                                events.push_back(InternalEvent {
                                    context: RunnerContext::new(
                                        self,
                                        right,
                                        right,
                                        &[CnvValue::String(left.name.clone())],
                                    ),
                                    callable: callable.to_owned(),
                                });
                            })
                    }
                }
            }
        }
        while let Some(evt) = self
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| events.pop_front())
        {
            // println!("Internal event: {:?} with context {}", evt.callable, evt.context);
            evt.context.current_object.call_method(
                (&evt.callable).into(),
                &evt.context.arguments,
                Some(evt.context.clone().with_arguments(Vec::new())),
            )?;
        }
        Ok(())
    }

    fn find_relevant_button(
        &self,
        buttons: &[ButtonDescriptor],
        mouse_position: (isize, isize),
    ) -> anyhow::Result<Option<usize>> {
        let mut result_index = None;
        for (i, button) in buttons.iter().enumerate() {
            if result_index.is_some() {
                break;
            }
            if let Some(visible_rect) = button.rect.intersect(&self.window_rect) {
                if visible_rect.has_inside(mouse_position.0, mouse_position.1) {
                    result_index = Some(i);
                    break;
                }
            }
        }
        Ok(result_index)
    }

    pub fn load_script(
        self: &Arc<Self>,
        path: ScenePath,
        contents: impl Iterator<Item = declarative_parser::ParserInput>,
        parent_object: Option<Arc<CnvObject>>,
        source_kind: ScriptSource,
    ) -> anyhow::Result<()> {
        let mut parser_issue_manager: IssueManager<ParserIssue> = Default::default();
        parser_issue_manager.set_handler(Box::new(IssuePrinter));
        let mut issue_manager: IssueManager<ObjectBuilderError> = Default::default();
        issue_manager.set_handler(Box::new(IssuePrinter));
        let mut dec_parser =
            DeclarativeParser::new(contents, Default::default(), parser_issue_manager).peekable();
        let mut objects: Vec<CnvObjectBuilder> = Vec::new();
        let mut name_to_object: HashMap<String, usize> = HashMap::new();
        let script = Arc::new(CnvScript::new(
            Arc::clone(self),
            path.clone(),
            parent_object.clone(),
            source_kind,
        ));
        while let Some(Ok((_pos, dec, _))) = dec_parser.next_if(|result| result.is_ok()) {
            match dec {
                CnvDeclaration::ObjectInitialization(name) => {
                    objects.push(CnvObjectBuilder::new(
                        Arc::clone(&script),
                        name.trim().to_owned(),
                        objects.len(),
                    ));
                    name_to_object
                        .insert(name.trim().to_owned(), objects.len() - 1)
                        .warn_if_some();
                }
                CnvDeclaration::PropertyAssignment {
                    parent,
                    property,
                    property_key,
                    value,
                } => {
                    let Some(obj) = name_to_object
                        .get(parent.trim())
                        .and_then(|i| objects.get_mut(*i))
                    else {
                        panic!(
                            "Expected {} element to be in dict, the element list is: {:?}",
                            &parent, &objects
                        );
                    };
                    obj.add_property(
                        property_key
                            .map(|suffix| property.clone() + "^" + &suffix)
                            .unwrap_or(property),
                        value,
                    )?;
                }
            }
        }
        if let Some(Err(err)) = dec_parser.next_if(|result| result.is_err()) {
            return Err(RunnerError::ParserError(err).into());
        }
        script
            .objects
            .borrow_mut()
            .push_objects(objects.into_iter().map(|builder| match builder.build() {
                Ok(built_object) => built_object,
                Err(e) => {
                    issue_manager.emit_issue(e);
                    panic!();
                }
            }))?;

        let mut container = self.scripts.borrow_mut();
        container.push_script(script)?; // TODO: err if present
        self.events_out
            .script
            .borrow_mut()
            .push_back(ScriptEvent::ScriptLoaded { path: path.clone() });
        Ok(())
    }

    pub fn get_script(&self, path: &ScenePath) -> Option<Arc<CnvScript>> {
        self.scripts.borrow().get_script(path)
    }

    pub fn get_root_script(&self) -> Option<Arc<CnvScript>> {
        self.scripts.borrow().get_root_script()
    }

    pub fn find_scripts(
        &self,
        predicate: impl Fn(&CnvScript) -> bool,
        buffer: &mut Vec<Arc<CnvScript>>,
    ) {
        buffer.clear();
        for script in self.scripts.borrow().iter() {
            if predicate(script.as_ref()) {
                buffer.push(Arc::clone(script));
            }
        }
    }

    pub fn unload_all_scripts(&self) {
        self.scripts.borrow_mut().remove_all_scripts();
    }

    pub fn unload_script(&self, path: &ScenePath) -> anyhow::Result<()> {
        self.scripts.borrow_mut().remove_script(path)
    }

    pub fn get_object(&self, name: &str) -> Option<Arc<CnvObject>> {
        // println!("Getting object: {:?}", name);
        self.scripts
            .borrow()
            .iter()
            .rev()
            .map(|s| s.get_object(name))
            .find(|o| o.is_some())
            .flatten()
            .or(self.global_objects.borrow().get_object(name))
    }

    pub fn find_object(&self, predicate: impl Fn(&CnvObject) -> bool) -> Option<Arc<CnvObject>> {
        self.scripts
            .borrow()
            .iter()
            .rev()
            .map(|s| s.find_object(&predicate))
            .find(|o| o.is_some())
            .flatten()
            .or(self.global_objects.borrow().find_object(&predicate))
    }

    pub fn find_objects(
        &self,
        predicate: impl Fn(&CnvObject) -> bool,
        buffer: &mut Vec<Arc<CnvObject>>,
    ) {
        buffer.clear();
        for object in self.global_objects.borrow().iter() {
            if predicate(object) {
                buffer.push(Arc::clone(object));
            }
        }
        for script in self.scripts.borrow().iter() {
            for object in script.objects.borrow().iter() {
                if predicate(object) {
                    buffer.push(Arc::clone(object));
                }
            }
        }
    }

    pub fn filter_map_objects<T>(
        &self,
        f: impl Fn(ObjectIndex, &Arc<CnvObject>) -> anyhow::Result<Option<T>>,
        buffer: &mut Vec<T>,
    ) -> anyhow::Result<()> {
        buffer.clear();
        for object in self.global_objects.borrow().iter() {
            if let Some(result) = f(ObjectIndex::default(), object)? {
                buffer.push(result);
            }
        }
        for (script_idx, script) in self.scripts.borrow().iter().enumerate() {
            for (object_idx, object) in script.objects.borrow().iter().enumerate() {
                if let Some(result) = f(
                    ObjectIndex {
                        script_idx,
                        object_idx,
                    },
                    object,
                )? {
                    buffer.push(result);
                }
            }
        }
        Ok(())
    }

    pub fn change_scene(self: &Arc<Self>, scene_name: &str) -> anyhow::Result<()> {
        self.internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| events.clear());
        self.scripts.borrow_mut().remove_scene_script()?;
        let Some(scene_object) = self.get_object(scene_name) else {
            return Err(RunnerError::ObjectNotFound {
                name: scene_name.to_owned(),
            }
            .into());
        };
        let CnvContent::Scene(ref scene) = &scene_object.content else {
            panic!();
        };
        let scene_name = scene_object.name.clone();
        let scene_path = scene.get_script_path().unwrap();
        let contents = (*self.filesystem)
            .write()
            .unwrap()
            .read_scene_asset(
                self.game_paths.clone(),
                &ScenePath::new(&scene_path, &(scene_name.clone() + ".cnv")),
            )
            .unwrap();
        let contents = parse_cnv(&contents);
        self.load_script(
            ScenePath::new(&scene_path, &scene_name),
            contents.as_parser_input(),
            Some(Arc::clone(&scene_object)),
            ScriptSource::Scene,
        )?;
        scene.handle_scene_loaded()
    }

    pub fn get_current_scene(&self) -> Option<Arc<CnvObject>> {
        self.scripts
            .borrow()
            .get_scene_script()
            .and_then(|s| s.parent_object.as_ref().cloned())
    }
}

pub enum BehaviorRunningError {
    ScriptNotFound,
    ObjectNotFound,
    InvalidType,
    RunnerError(RunnerError),
}
