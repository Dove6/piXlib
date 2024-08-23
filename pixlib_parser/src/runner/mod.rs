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
    ApplicationEvent, FileEvent, GraphicsEvent, InternalEvent, KeyboardEvent, KeyboardKey,
    MouseEvent, MultimediaEvents, ObjectEvent, ScriptEvent, SoundEvent, SoundSource, TimerEvent,
};
pub use filesystem::{FileSystem, GamePaths};
use itertools::Itertools;
pub use object::{CnvObject, ObjectBuildErrorKind, ObjectBuilderError};
pub use path::ScenePath;
pub use script::{CnvScript, ScriptSource};
use thiserror::Error;
pub use tree_walking::{CnvExpression, CnvStatement};
pub use value::CnvValue;

use std::collections::VecDeque;
use std::fmt::Display;
use std::{cell::RefCell, collections::HashMap, sync::Arc};

use events::{IncomingEvents, OutgoingEvents};

use crate::{
    common::{DroppableRefMut, Issue, IssueHandler, IssueKind, IssueManager},
    parser::declarative_parser::{
        self, CnvDeclaration, DeclarativeParser, ParserFatal, ParserIssue,
    },
    scanner::parse_cnv,
};
use classes::{Animation, InternalMouseEvent, Mouse, Scene, Sound, Timer};
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

#[derive(Debug)]
pub enum RunnerError {
    TooManyArguments {
        expected_max: usize,
        actual: usize,
    },
    TooFewArguments {
        expected_min: usize,
        actual: usize,
    },
    ExpectedUnsignedInteger {
        actual: i32,
    },
    MissingLeftOperand {
        object_name: String,
    },
    MissingRightOperand {
        object_name: String,
    },
    MissingOperator {
        object_name: String,
    },
    ObjectNotFound {
        name: String,
    },
    ExpectedGraphicsObject,
    NoDataLoaded,
    SequenceNameNotFound {
        object_name: String,
        sequence_name: String,
    },
    SequenceIndexNotFound {
        object_name: String,
        index: usize,
    },
    FrameIndexNotFound {
        object_name: String,
        sequence_name: String,
        index: usize,
    },
    SpriteIndexNotFound {
        object_name: String,
        index: usize,
    },
    InvalidCallable {
        object_name: String,
        callable: CallableIdentifierOwned,
    },
    MissingFilenameToLoad,

    ScriptNotFound {
        path: String,
    },
    RootScriptAlreadyLoaded,
    ApplicationScriptAlreadyLoaded,
    EpisodeScriptAlreadyLoaded,
    SceneScriptAlreadyLoaded,

    ParserError(ParserFatal),

    IoError {
        source: std::io::Error,
    },
    ObjectBuilderError {
        source: ObjectBuilderError,
    },
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
    pub filesystem: Arc<RefCell<dyn FileSystem>>,
    pub game_paths: Arc<GamePaths>,
    pub issue_manager: Arc<RefCell<IssueManager<RunnerIssue>>>,
    pub global_objects: RefCell<ObjectContainer>,
    pub window_rect: Rect,
    visible_graphics: RefCell<Vec<GraphicsDescriptor>>,
}

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub top_left_x: isize,
    pub top_left_y: isize,
    pub bottom_right_x: isize,
    pub bottom_right_y: isize,
}

impl Rect {
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

#[derive(Debug, Clone)]
struct GraphicsDescriptor {
    pub priority: isize,
    pub object_index: ObjectIndex,
    pub object: Arc<CnvObject>,
    pub rect: Rect,
}

impl PartialEq for GraphicsDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.object_index == other.object_index
    }
}

impl Eq for GraphicsDescriptor {}

impl PartialOrd for GraphicsDescriptor {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GraphicsDescriptor {
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
        filesystem: Arc<RefCell<dyn FileSystem>>,
        game_paths: Arc<GamePaths>,
        window_resolution: (usize, usize),
        issue_manager: IssueManager<RunnerIssue>,
    ) -> RunnerResult<Arc<Self>> {
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
            visible_graphics: RefCell::new(Vec::new()),
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
            .use_and_drop_mut::<RunnerResult<()>>(|objects| {
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
                Ok(())
            })?;
        Ok(runner)
    }

    pub fn step(self: &Arc<CnvRunner>) -> RunnerResult<()> {
        let mut to_init = Vec::new();
        self.find_objects(|o| !*o.initialized.borrow(), &mut to_init);
        for object in to_init {
            object.init(None)?;
        }
        self.events_in
            .timer
            .borrow_mut()
            .use_and_drop_mut::<RunnerResult<()>>(|events| {
                while let Some(evt) = events.pop_front() {
                    match evt {
                        TimerEvent::Elapsed { seconds } => {
                            let mut buffer = Vec::new();
                            self.find_objects(
                                |o| matches!(&*o.content.borrow(), CnvContent::Animation(_)),
                                &mut buffer,
                            );
                            for animation_object in buffer.iter() {
                                let guard = animation_object.content.borrow();
                                let animation: Option<&Animation> = (&*guard).into();
                                let animation = animation.unwrap();
                                animation.step(seconds)?;
                            }
                            self.find_objects(
                                |o| matches!(&*o.content.borrow(), CnvContent::Timer(_)),
                                &mut buffer,
                            );
                            for timer_object in buffer.iter() {
                                let guard = timer_object.content.borrow();
                                let timer: Option<&Timer> = (&*guard).into();
                                let timer = timer.unwrap();
                                timer.step(seconds)?;
                            }
                        }
                    }
                }
                Ok(())
            })?;
        self.events_in
            .mouse
            .borrow_mut()
            .use_and_drop_mut::<RunnerResult<()>>(|events| {
                while let Some(evt) = events.pop_front() {
                    // eprintln!("Handling incoming mouse event: {:?}", evt);
                    Mouse::handle_incoming_event(evt)?;
                }
                Ok(())
            })?;
        self.events_in
            .multimedia
            .borrow_mut()
            .use_and_drop_mut::<RunnerResult<()>>(|events| {
                while let Some(evt) = events.pop_front() {
                    match &evt {
                        MultimediaEvents::SoundFinishedPlaying(source) => {
                            match source {
                                SoundSource::BackgroundMusic => {
                                    let Some(scene_object) = self.get_current_scene() else {
                                        eprintln!("No current scene to handle event {:?}", evt);
                                        continue;
                                    };
                                    let guard = scene_object.content.borrow();
                                    let scene: Option<&Scene> = (&*guard).into();
                                    let scene = scene.unwrap();
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
                                    let guard = sound_object.content.borrow();
                                    let sound: Option<&Sound> = (&*guard).into();
                                    let sound = sound.unwrap();
                                    sound.handle_finished()?;
                                }
                                SoundSource::AnimationSfx { .. } => {}
                            };
                        }
                    }
                }
                Ok(())
            })?;
        let mut buttons = Vec::new();
        self.find_objects(
            |o| matches!(&*o.content.borrow(), CnvContent::Button(_)),
            &mut buttons,
        );
        self.filter_map_objects(
            |id, o| {
                Ok(match &*o.content.borrow() {
                    CnvContent::Animation(a) => Some(GraphicsDescriptor {
                        priority: a.get_priority()?,
                        object_index: id,
                        object: o.clone(),
                        rect: {
                            let position = a.get_frame_position()?;
                            let size = a.get_frame_size()?;
                            Rect {
                                top_left_x: position.0,
                                top_left_y: position.1,
                                bottom_right_x: position.0 + size.0 as isize,
                                bottom_right_y: position.1 + size.1 as isize,
                            }
                        },
                    }),
                    CnvContent::Image(i) => Some(GraphicsDescriptor {
                        priority: i.get_priority()?,
                        object_index: id,
                        object: o.clone(),
                        rect: {
                            let position = i.get_position()?;
                            let size = i.get_size()?;
                            Rect {
                                top_left_x: position.0,
                                top_left_y: position.1,
                                bottom_right_x: position.0 + size.0 as isize,
                                bottom_right_y: position.1 + size.1 as isize,
                            }
                        },
                    }),
                    _ => None,
                })
            },
            &mut *self.visible_graphics.borrow_mut(),
        )?;
        self.visible_graphics.borrow_mut().sort();
        let mouse_position = Mouse::get_position()?;
        let mouse_is_left_button_down = Mouse::is_left_button_down()?;
        let found_button_index = self.find_active_button(&buttons, mouse_position)?;
        for (i, o) in buttons.iter().enumerate() {
            let CnvContent::Button(ref button) = &*o.content.borrow() else {
                panic!();
            };
            if found_button_index.is_some_and(|found| found == i) {
                if mouse_is_left_button_down {
                    button.keep_pressing()
                } else {
                    button.set_hovering()
                }
            } else {
                button.set_normal()
            }?
        }
        let mut mouse_objects = Vec::new();
        self.find_objects(
            |o| matches!(*o.content.borrow(), CnvContent::Mouse(_)),
            &mut mouse_objects,
        );
        Mouse::handle_outgoing_events(|mouse_event| {
            // eprintln!("Handling internal mouse event: {:?}", mouse_event);
            if let InternalMouseEvent::LeftButtonPressed { x, y } = &mouse_event {
                if let Some(button_idx) = self.find_active_button(&buttons, (*x, *y))? {
                    let CnvContent::Button(ref button) = &*buttons[button_idx].content.borrow()
                    else {
                        panic!();
                    };
                    button.set_pressing()?;
                }
            }
            if let InternalMouseEvent::LeftButtonReleased { x, y } = &mouse_event {
                if let Some(button_index) = self.find_active_button(&buttons, (*x, *y))? {
                    self.internal_events
                        .borrow_mut()
                        .use_and_drop_mut(|internal_events| {
                            internal_events.push_back(InternalEvent {
                                object: buttons[button_index].clone(),
                                callable: CallableIdentifier::Event("ONACTION").to_owned(),
                                arguments: Vec::new(),
                            })
                        });
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
                            object: Arc::clone(mouse_object),
                            callable: callable.to_owned(),
                            arguments: arguments.clone(),
                        })
                    });
            }
            Ok(())
        })?;
        let mut collidable = Vec::new();
        self.find_objects(
            |o| match &*o.content.borrow() {
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
                    let (left_position, left_size, left_pixel_perfect) =
                        match &*left.content.borrow() {
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
                    let (right_position, right_size, right_pixel_perfect) =
                        match &*right.content.borrow() {
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
                                    object: Arc::clone(left),
                                    callable: callable.to_owned(),
                                    arguments: vec![CnvValue::String(right.name.clone())],
                                });
                                events.push_back(InternalEvent {
                                    object: Arc::clone(right),
                                    callable: callable.to_owned(),
                                    arguments: vec![CnvValue::String(left.name.clone())],
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
            evt.object
                .call_method((&evt.callable).into(), &evt.arguments, None)?;
        }
        Ok(())
    }

    fn find_active_button(
        &self,
        buttons: &[Arc<CnvObject>],
        mouse_position: (isize, isize),
    ) -> RunnerResult<Option<usize>> {
        let mut result_index = None;
        for graphics in self.visible_graphics.borrow().iter() {
            if result_index.is_some() {
                break;
            }
            if let Some(visible_rect) = graphics.rect.intersect(&self.window_rect) {
                if visible_rect.has_inside(mouse_position.0, mouse_position.1) {
                    for (i, o) in buttons.iter().enumerate() {
                        let CnvContent::Button(ref button) = &*o.content.borrow() else {
                            panic!();
                        };
                        if button.is_displaying(&graphics.object.name)? {
                            result_index = Some(i);
                            break;
                        }
                    }
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
    ) -> RunnerResult<()> {
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
            return Err(RunnerError::ParserError(err));
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

    pub fn unload_script(&self, path: &ScenePath) -> RunnerResult<()> {
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
        f: impl Fn(ObjectIndex, &Arc<CnvObject>) -> RunnerResult<Option<T>>,
        buffer: &mut Vec<T>,
    ) -> RunnerResult<()> {
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

    pub fn change_scene(self: &Arc<Self>, scene_name: &str) -> RunnerResult<()> {
        self.internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| events.clear());
        self.scripts.borrow_mut().remove_scene_script()?;
        let Some(scene_object) = self.get_object(scene_name) else {
            return Err(RunnerError::ObjectNotFound {
                name: scene_name.to_owned(),
            });
        };
        let scene_guard = scene_object.content.borrow();
        let scene: Option<&Scene> = (&*scene_guard).into();
        let scene = scene.unwrap();
        let scene_name = scene_object.name.clone();
        let scene_path = scene.get_script_path().unwrap();
        let contents = (*self.filesystem)
            .borrow_mut()
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
