use super::super::{
    content::EventHandler,
    initable::Initable,
    parsers::{discard_if_empty, parse_bool, parse_event_handler, parse_i32},
};
use ::rand::{seq::SliceRandom, thread_rng};
use pixlib_formats::file_formats::ann::{parse_ann, LoopingSettings};
use std::{any::Any, cell::RefCell, sync::Arc};
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    common::{add_tuples, pair_i32_to_isize, pair_u32_to_usize, DroppableRefMut},
    parser::ast::ParsedScript,
    runner::{InternalEvent, RunnerError},
};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct AnimationProperties {
    // ANIMO
    pub as_button: Option<bool>,               // ASBUTTON
    pub filename: Option<String>,              // FILENAME
    pub flush_after_played: Option<bool>,      // FLUSHAFTERPLAYED
    pub fps: Option<i32>,                      // FPS
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    pub preload: Option<bool>,                 // PRELOAD
    pub priority: Option<i32>,                 // PRIORITY
    pub release: Option<bool>,                 // RELEASE
    pub to_canvas: Option<bool>,               // TOCANVAS
    pub visible: Option<bool>,                 // VISIBLE

    pub on_click: Option<Arc<ParsedScript>>, // ONCLICK signal
    pub on_collision: HashMap<String, Arc<ParsedScript>>, // ONCOLLISION signal
    pub on_collision_finished: HashMap<String, Arc<ParsedScript>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<ParsedScript>>,  // ONDONE signal
    pub on_finished: HashMap<String, Arc<ParsedScript>>, // ONFINISHED signal
    pub on_first_frame: HashMap<String, Arc<ParsedScript>>, // ONFIRSTFRAME signal
    pub on_focus_off: Option<Arc<ParsedScript>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<ParsedScript>>, // ONFOCUSON signal
    pub on_frame_changed: HashMap<String, Arc<ParsedScript>>, // ONFRAMECHANGED signal
    pub on_init: Option<Arc<ParsedScript>>,  // ONINIT signal
    pub on_paused: HashMap<String, Arc<ParsedScript>>, // ONPAUSED signal
    pub on_release: Option<Arc<ParsedScript>>, // ONRELEASE signal
    pub on_resumed: HashMap<String, Arc<ParsedScript>>, // ONRESUMED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_started: HashMap<String, Arc<ParsedScript>>, // ONSTARTED signal
}

#[derive(Debug, Clone, Default)]
struct AnimationState {
    // initialized from properties
    pub is_button: bool,
    pub file_data: Arc<AnimationFileData>,
    pub fps: usize,
    pub does_monitor_collision: bool,
    pub priority: isize,
    pub is_visible: bool,

    // general graphics state
    pub position: (isize, isize),
    pub opacity: usize,
    // anchor: ???,
    pub is_flipped_horizontally: bool,
    pub is_flipped_vertically: bool,

    // related to animation
    pub is_playing: bool,
    pub is_paused: bool,
    pub is_reversed: bool,
    pub current_frame: FrameIdentifier,
    pub sprite_idx_override: Option<usize>,
    // more temporary
    pub current_frame_duration: f64,

    // related to sound
    pub panning: isize,
    pub volume: isize,
    pub current_sfx: SoundFileData,

    // related to button
    pub cursor_interaction: CursorInteraction,
    pub should_show_pointer_on_hover: bool,
}

#[derive(Debug, Clone)]
pub struct AnimationEventHandlers {
    pub on_click: Option<Arc<ParsedScript>>, // ONCLICK signal
    pub on_collision: HashMap<String, Arc<ParsedScript>>, // ONCOLLISION signal
    pub on_collision_finished: HashMap<String, Arc<ParsedScript>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<ParsedScript>>,  // ONDONE signal
    pub on_finished: HashMap<String, Arc<ParsedScript>>, // ONFINISHED signal
    pub on_first_frame: HashMap<String, Arc<ParsedScript>>, // ONFIRSTFRAME signal
    pub on_focus_off: Option<Arc<ParsedScript>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<ParsedScript>>, // ONFOCUSON signal
    pub on_frame_changed: HashMap<String, Arc<ParsedScript>>, // ONFRAMECHANGED signal
    pub on_init: Option<Arc<ParsedScript>>,  // ONINIT signal
    pub on_paused: HashMap<String, Arc<ParsedScript>>, // ONPAUSED signal
    pub on_release: Option<Arc<ParsedScript>>, // ONRELEASE signal
    pub on_resumed: HashMap<String, Arc<ParsedScript>>, // ONRESUMED signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_started: HashMap<String, Arc<ParsedScript>>, // ONSTARTED signal
}

impl EventHandler for AnimationEventHandlers {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONCLICK" => self.on_click.as_ref(),
            "ONCOLLISION" => argument
                .and_then(|a| self.on_collision.get(a))
                .or(self.on_collision.get("")),
            "ONCOLLISIONFINISHED" => argument
                .and_then(|a| self.on_collision_finished.get(a))
                .or(self.on_collision_finished.get("")),
            "ONDONE" => self.on_done.as_ref(),
            "ONFINISHED" => argument
                .and_then(|a| self.on_finished.get(a))
                .or(self.on_finished.get("")),
            "ONFIRSTFRAME" => argument
                .and_then(|a| self.on_first_frame.get(a))
                .or(self.on_first_frame.get("")),
            "ONFOCUSOFF" => self.on_focus_off.as_ref(),
            "ONFOCUSON" => self.on_focus_on.as_ref(),
            "ONFRAMECHANGED" => argument
                .and_then(|a| self.on_frame_changed.get(a))
                .or(self.on_frame_changed.get("")),
            "ONINIT" => self.on_init.as_ref(),
            "ONPAUSED" => argument
                .and_then(|a| self.on_paused.get(a))
                .or(self.on_paused.get("")),
            "ONRELEASE" => self.on_release.as_ref(),
            "ONRESUMED" => argument
                .and_then(|a| self.on_resumed.get(a))
                .or(self.on_resumed.get("")),
            "ONSIGNAL" => argument
                .and_then(|a| self.on_signal.get(a))
                .or(self.on_signal.get("")),
            "ONSTARTED" => argument
                .and_then(|a| self.on_started.get(a))
                .or(self.on_started.get("")),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    // ANIMO
    parent: Arc<CnvObject>,

    state: RefCell<AnimationState>,
    event_handlers: AnimationEventHandlers,

    should_flush_after_played: bool,
    should_collisions_respect_alpha: bool,
    should_preload: bool,
    should_release: bool,
    should_draw_to_canvas: bool,
}

impl Animation {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: AnimationProperties) -> Self {
        let animation = Self {
            parent: parent.clone(),
            state: RefCell::new(AnimationState {
                is_button: props.as_button.unwrap_or_default(),
                fps: props.fps.unwrap_or(16) as usize,
                does_monitor_collision: props.monitor_collision.unwrap_or_default(),
                priority: props.priority.unwrap_or_default() as isize,
                is_visible: props.visible.unwrap_or(true),
                ..AnimationState::default()
            }),
            event_handlers: AnimationEventHandlers {
                on_click: props.on_click,
                on_collision: props.on_collision,
                on_collision_finished: props.on_collision_finished,
                on_done: props.on_done,
                on_finished: props.on_finished,
                on_first_frame: props.on_first_frame,
                on_focus_off: props.on_focus_off,
                on_focus_on: props.on_focus_on,
                on_frame_changed: props.on_frame_changed,
                on_init: props.on_init,
                on_paused: props.on_paused,
                on_release: props.on_release,
                on_resumed: props.on_resumed,
                on_signal: props.on_signal,
                on_started: props.on_started,
            },
            should_flush_after_played: props.flush_after_played.unwrap_or_default(),
            should_collisions_respect_alpha: props.monitor_collision_alpha.unwrap_or_default(),
            should_preload: props.preload.unwrap_or_default(),
            should_release: props.release.unwrap_or(true),
            should_draw_to_canvas: props.to_canvas.unwrap_or(true),
        };
        if let Some(filename) = props.filename {
            animation.state.borrow_mut().file_data =
                Arc::new(AnimationFileData::NotLoaded(filename));
        }
        animation
    }

    // custom

    pub fn get_base_position(&self) -> anyhow::Result<(isize, isize)> {
        self.state.borrow().get_base_position()
    }

    pub fn get_frame_position(&self) -> anyhow::Result<(isize, isize)> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().get_frame_position(context)
    }

    pub fn get_frame_size(&self) -> anyhow::Result<(usize, usize)> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().get_frame_size(context)
    }

    pub fn get_frame_rect(&self) -> anyhow::Result<Rect> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().get_frame_rect(context)
    }

    pub fn get_center_frame_position(&self) -> anyhow::Result<(isize, isize)> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().get_center_frame_position(context)
    }

    pub fn does_monitor_collision(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().does_monitor_collision)
    }

    pub fn does_monitor_collision_pixel_perfect(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().does_monitor_collision && self.should_collisions_respect_alpha)
    }

    pub fn step(&self, seconds: f64) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().step(context, seconds)
    }

    pub fn get_frame_to_show(&self) -> anyhow::Result<Option<(Rect, SpriteData)>> {
        // log::trace!("[ANIMO: {}] is_visible: {}", self.parent.name, self.is_visible);
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        let state = self.state.borrow();
        if !state.is_visible {
            return Ok(None);
        }
        let AnimationFileData::Loaded(ref loaded_data) = *state.file_data else {
            return Ok(None);
        };
        if loaded_data.sprites.is_empty() {
            return Ok(None);
        }
        if let Some(sprite_idx_override) = state.sprite_idx_override {
            let Some(sprite) = loaded_data.sprites.get(sprite_idx_override) else {
                return Err(RunnerError::SpriteIndexNotFound {
                    object_name: context.current_object.name.clone(),
                    index: sprite_idx_override,
                }
                .into());
            };
            let position = add_tuples(state.position, pair_i32_to_isize(sprite.0.offset_px));
            let size = pair_u32_to_usize(sprite.0.size_px);
            return Ok(Some((Rect::from(position, size), sprite.1.clone())));
        };
        if loaded_data.sequences.is_empty() {
            return Ok(None);
        }
        let sequence = &loaded_data.sequences[state.current_frame.sequence_idx];
        if sequence.frames.is_empty() {
            return Ok(None);
        }
        let Some(frame) = sequence.frames.get(state.current_frame.frame_idx) else {
            return Err(RunnerError::FrameIndexNotFound {
                object_name: context.current_object.name.clone(),
                sequence_name: sequence.name.clone(),
                index: state.current_frame.frame_idx,
            }
            .into());
        };
        let Some(sprite) = loaded_data.sprites.get(frame.sprite_idx) else {
            return Err(RunnerError::SpriteIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: frame.sprite_idx,
            }
            .into());
        };
        let position = add_tuples(state.position, pair_i32_to_isize(sprite.0.offset_px));
        let position = add_tuples(position, pair_i32_to_isize(frame.offset_px));
        let size = pair_u32_to_usize(sprite.0.size_px);
        // log::trace!("[ANIMO: {}] [current frame] position: {:?} + {:?}, hash: {:?}", self.parent.name, sprite.0.offset_px, frame.offset_px, sprite.1.hash);
        Ok(Some((Rect::from(position, size), sprite.1.clone())))
    }

    pub fn play(&self, sequence_name: &str) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().play(context, sequence_name)
    }

    pub fn stop(&self, emit_on_finished: bool) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().stop(context, emit_on_finished)
    }

    pub fn pause(&self) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().pause(context)
    }

    pub fn resume(&self) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().resume(context)
    }

    pub fn get_filename(&self) -> anyhow::Result<Option<String>> {
        self.state.borrow().get_filename()
    }

    pub fn has_sequence(&self, name: &str) -> anyhow::Result<bool> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().has_sequence(context, name)
    }

    pub fn is_playing(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().is_playing)
    }
}

impl GeneralGraphics for Animation {
    fn hide(&self) -> anyhow::Result<()> {
        self.state.borrow_mut().hide()
    }

    fn show(&self) -> anyhow::Result<()> {
        self.state.borrow_mut().show()
    }

    fn is_visible(&self) -> anyhow::Result<bool> {
        self.state.borrow().is_visible()
    }

    fn get_rect(&self) -> anyhow::Result<Option<Rect>> {
        self.get_frame_rect().map(Some)
    }

    fn get_priority(&self) -> anyhow::Result<isize> {
        self.state.borrow().get_priority()
    }
}

impl GeneralButton for Animation {
    fn is_enabled(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().is_button)
    }

    fn get_rect(&self) -> anyhow::Result<Option<Rect>> {
        let graphics: &dyn GeneralGraphics = self;
        graphics.get_rect()
    }

    fn get_priority(&self) -> anyhow::Result<isize> {
        let graphics: &dyn GeneralGraphics = self;
        graphics.get_priority()
    }

    fn handle_lmb_pressed(&self) -> anyhow::Result<()> {
        if self.state.borrow_mut().use_and_drop_mut(|state| {
            let prev_interaction = state.cursor_interaction;
            state.cursor_interaction = CursorInteraction::Pressing;
            prev_interaction != state.cursor_interaction
        }) {
            let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|internal_events| {
                    internal_events.push_back(InternalEvent {
                        context: context.clone(),
                        callable: CallableIdentifier::Event("ONCLICK").to_owned(),
                    })
                });
        }
        Ok(())
    }

    fn handle_lmb_released(&self) -> anyhow::Result<()> {
        if self.state.borrow_mut().use_and_drop_mut(|state| {
            let prev_interaction = state.cursor_interaction;
            state.cursor_interaction = CursorInteraction::Hovering;
            prev_interaction != state.cursor_interaction
        }) {
            let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|internal_events| {
                    internal_events.push_back(InternalEvent {
                        context: context.clone(),
                        callable: CallableIdentifier::Event("ONRELEASE").to_owned(),
                    })
                });
        }
        Ok(())
    }

    fn handle_cursor_over(&self) -> anyhow::Result<()> {
        if self.state.borrow_mut().use_and_drop_mut(|state| {
            if state.cursor_interaction == CursorInteraction::Pressing {
                return false;
            }
            let prev_interaction = state.cursor_interaction;
            state.cursor_interaction = CursorInteraction::Hovering;
            prev_interaction != state.cursor_interaction
        }) {
            let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|internal_events| {
                    internal_events.push_back(InternalEvent {
                        context: context.clone(),
                        callable: CallableIdentifier::Event("ONFOCUSON").to_owned(),
                    })
                });
        }
        Ok(())
    }

    fn handle_cursor_away(&self) -> anyhow::Result<()> {
        let (unfocused, released) = self.state.borrow_mut().use_and_drop_mut(|state| {
            let prev_interaction = state.cursor_interaction;
            state.cursor_interaction = CursorInteraction::None;
            (
                prev_interaction != state.cursor_interaction,
                prev_interaction == CursorInteraction::Pressing,
            )
        });
        if unfocused {
            let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|internal_events| {
                    internal_events.push_back(InternalEvent {
                        context: context.clone(),
                        callable: CallableIdentifier::Event("ONFOCUSOFF").to_owned(),
                    });
                    if released {
                        internal_events.push_back(InternalEvent {
                            context: context.clone(),
                            callable: CallableIdentifier::Event("ONRELEASE").to_owned(),
                        });
                    }
                });
        }
        Ok(())
    }

    fn makes_cursor_pointer(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().should_show_pointer_on_hover)
    }
}

impl CnvType for Animation {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "ANIMO"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        // log::trace!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("CLEARCLIPPING") => self
                .state
                .borrow_mut()
                .clear_clipping()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("DRAWONTO") => {
                self.state.borrow_mut().draw_onto().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("FLIPH") => {
                self.state.borrow_mut().flip_h().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("FLIPV") => {
                self.state.borrow_mut().flip_v().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETALPHA") => {
                self.state.borrow().get_alpha().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETANCHOR") => self
                .state
                .borrow()
                .get_anchor()
                .map(|v| CnvValue::String(v.to_owned())),
            CallableIdentifier::Method("GETCENTERX") => {
                self.state.borrow().get_center_x().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETCENTERY") => {
                self.state.borrow().get_center_y().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETCFRAMEINEVENT") => self
                .state
                .borrow()
                .get_cframe_in_event()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETCURRFRAMEPOSX") => self
                .state
                .borrow()
                .get_curr_frame_pos_x()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETCURRFRAMEPOSY") => self
                .state
                .borrow()
                .get_curr_frame_pos_y()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETENDX") => {
                self.state.borrow().get_end_x().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETENDY") => {
                self.state.borrow().get_end_y().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETEVENTNAME") => self
                .state
                .borrow()
                .get_sequence_name(context)
                .map(CnvValue::String),
            CallableIdentifier::Method("GETEVENTNUMBER") => self
                .state
                .borrow()
                .get_sequence_index()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETFPS") => {
                self.state.borrow().get_fps().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETFRAME") => self
                .state
                .borrow()
                .get_frame()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETFRAMENAME") => {
                self.state.borrow().get_frame_name().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETFRAMENO") => self
                .state
                .borrow()
                .get_frame_index()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETHEIGHT") => {
                self.state.borrow().get_height().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETMAXHEIGHT") => {
                self.state.borrow().get_max_height().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETMAXWIDTH") => {
                self.state.borrow().get_max_width().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETNOE") => self
                .state
                .borrow()
                .get_sequence_count()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETNOF") => self
                .state
                .borrow()
                .get_total_frame_count()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETNOFINEVENT") => self
                .state
                .borrow()
                .get_sequence_frame_count(&arguments[0].to_str())
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETOPACITY") => {
                self.state.borrow().get_opacity().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETPIXEL") => {
                self.state.borrow().get_pixel().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETPOSITIONX") => self
                .state
                .borrow()
                .get_frame_position_x(context)
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETPOSITIONY") => self
                .state
                .borrow()
                .get_frame_position_y(context)
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETPRIORITY") => self
                .state
                .borrow()
                .get_priority()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETWIDTH") => {
                self.state.borrow().get_width().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("HIDE") => {
                self.state.borrow_mut().hide().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("INVALIDATE") => {
                self.state.borrow_mut().invalidate().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISAT") => {
                self.state.borrow().is_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISINSIDE") => {
                self.state.borrow().is_inside().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISNEAR") => {
                let name = arguments[0].to_str();
                let other = context
                    .runner
                    .get_object(&name)
                    .ok_or(RunnerError::ObjectNotFound { name })?;
                self.state
                    .borrow()
                    .is_near(context, other, arguments[1].to_int().max(0) as usize)
                    .map(CnvValue::Bool)
            }
            CallableIdentifier::Method("ISPLAYING") => {
                self.state.borrow().is_playing().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("ISVISIBLE") => {
                self.state.borrow().is_visible().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("LOAD") => self
                .state
                .borrow_mut()
                .load(context, &arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("MERGEALPHA") => self
                .state
                .borrow_mut()
                .merge_alpha()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("MONITORCOLLISION") => self
                .state
                .borrow_mut()
                .monitor_collision()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("MOVE") => self
                .state
                .borrow_mut()
                .move_by(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("NEXTFRAME") => {
                self.state.borrow_mut().next_frame().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("NPLAY") => {
                self.state.borrow_mut().n_play().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("PAUSE") => self
                .state
                .borrow_mut()
                .pause(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("PLAY") => self
                .state
                .borrow_mut()
                .play(context, &arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("PLAYRAND") => self
                .state
                .borrow_mut()
                .play_rand(
                    &arguments[0].to_str(),
                    arguments[1].to_int() as usize,
                    arguments[2].to_int() as usize,
                )
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("PLAYREVERSE") => self
                .state
                .borrow_mut()
                .play_reverse()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("PREVFRAME") => {
                self.state.borrow_mut().prev_frame().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("REMOVEMONITORCOLLISION") => self
                .state
                .borrow_mut()
                .remove_monitor_collision()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("REPLACECOLOR") => self
                .state
                .borrow_mut()
                .replace_color()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("RESETFLIPS") => self
                .state
                .borrow_mut()
                .reset_flips()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("RESUME") => self
                .state
                .borrow_mut()
                .resume(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETANCHOR") => self
                .state
                .borrow_mut()
                .set_anchor(&arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETASBUTTON") => self
                .state
                .borrow_mut()
                .set_as_button(arguments[0].to_bool(), arguments[1].to_bool())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETBACKWARD") => self
                .state
                .borrow_mut()
                .set_backward()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETCLIPPING") => self
                .state
                .borrow_mut()
                .set_clipping()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETFORWARD") => self
                .state
                .borrow_mut()
                .set_forward()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETFPS") => self
                .state
                .borrow_mut()
                .set_fps(arguments[0].to_int() as usize)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETFRAME") => {
                let (sequence_name, frame_no) = match arguments.len() {
                    1 => (None, arguments[0].to_int()),
                    2 => (Some(arguments[0].to_str()), arguments[1].to_int()),
                    0 => {
                        return Err(RunnerError::TooFewArguments {
                            expected_min: 1,
                            actual: 0,
                        }
                        .into())
                    }
                    arg_count => {
                        return Err(RunnerError::TooManyArguments {
                            expected_max: 2,
                            actual: arg_count,
                        }
                        .into())
                    }
                };
                // if frame_no < 0 {
                //     return Err(RunnerError::ExpectedUnsignedInteger { actual: frame_no });
                // }
                self.state
                    .borrow_mut()
                    .set_frame(context, sequence_name.as_deref(), frame_no.max(0) as usize)
                    .map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETFRAMENAME") => self
                .state
                .borrow_mut()
                .set_frame_name()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETFREQ") => {
                self.state.borrow_mut().set_freq().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETONFF") => {
                self.state.borrow_mut().set_onff().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETOPACITY") => self
                .state
                .borrow_mut()
                .set_opacity()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETPOSITION") => self
                .state
                .borrow_mut()
                .set_position(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETPRIORITY") => self
                .state
                .borrow_mut()
                .set_priority(arguments[0].to_int() as isize)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETPAN") => {
                self.state.borrow_mut().set_pan().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETVOLUME") => {
                self.state.borrow_mut().set_volume().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SHOW") => {
                self.state.borrow_mut().show().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("STOP") => self
                .state
                .borrow_mut()
                .stop(
                    context,
                    if arguments.is_empty() {
                        true // TODO: check
                    } else {
                        arguments[0].to_bool()
                    },
                )
                .map(|_| CnvValue::Null),
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context).map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
            }
            ident => Err(RunnerError::InvalidCallable {
                object_name: self.parent.name.clone(),
                callable: ident.to_owned(),
            }
            .into()),
        }
    }

    fn new_content(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let as_button = properties
            .remove("ASBUTTON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let fps = properties
            .remove("FPS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let release = properties
            .remove("RELEASE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let on_click = properties
            .remove("ONCLICK")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_collision = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONCOLLISION" {
                on_collision.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONCOLLISION^") {
                on_collision.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_collision_finished = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONCOLLISIONFINISHED" {
                on_collision_finished.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONCOLLISIONFINISHED^") {
                on_collision_finished
                    .insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_finished = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONFINISHED" {
                on_finished.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONFINISHED^") {
                on_finished.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_first_frame = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONFIRSTFRAME" {
                on_first_frame.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONFIRSTFRAME^") {
                on_first_frame.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let on_focus_off = properties
            .remove("ONFOCUSOFF")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_focus_on = properties
            .remove("ONFOCUSON")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_frame_changed = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONFRAMECHANGED" {
                on_frame_changed.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONFRAMECHANGED^") {
                on_frame_changed.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_paused = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONPAUSED" {
                on_paused.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONPAUSED^") {
                on_paused.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let on_release = properties
            .remove("ONRELEASE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_resumed = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONRESUMED" {
                on_resumed.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONRESUMED^") {
                on_resumed.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_signal = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSIGNAL" {
                on_signal.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONSIGNAL^") {
                on_signal.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_started = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSTARTED" {
                on_started.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONSTARTED^") {
                on_started.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        Ok(CnvContent::Animation(Animation::from_initial_properties(
            parent,
            AnimationProperties {
                as_button,
                filename,
                flush_after_played,
                fps,
                monitor_collision,
                monitor_collision_alpha,
                preload,
                priority,
                release,
                to_canvas,
                visible,
                on_click,
                on_collision,
                on_collision_finished,
                on_done,
                on_finished,
                on_first_frame,
                on_focus_off,
                on_focus_on,
                on_frame_changed,
                on_init,
                on_paused,
                on_release,
                on_resumed,
                on_signal,
                on_started,
            },
        )))
    }
}

impl Initable for Animation {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        let mut state = self.state.borrow_mut();
        if self.should_preload {
            if let AnimationFileData::NotLoaded(ref filename) = *state.file_data {
                let filename = filename.clone();
                state.load(context.clone(), &filename)?;
            };
        }
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(Vec::new()),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                })
            });
        Ok(())
    }
}

impl AnimationState {
    pub fn clear_clipping(&self) -> anyhow::Result<()> {
        // CLEARCLIPPING
        todo!()
    }

    pub fn draw_onto(&self) -> anyhow::Result<()> {
        // DRAWONTO
        todo!()
    }

    pub fn flip_h(&mut self) -> anyhow::Result<()> {
        // FLIPH
        self.is_flipped_horizontally = !self.is_flipped_horizontally;
        Ok(())
    }

    pub fn flip_v(&mut self) -> anyhow::Result<()> {
        // FLIPV
        self.is_flipped_vertically = !self.is_flipped_vertically;
        Ok(())
    }

    pub fn get_alpha(&self) -> anyhow::Result<()> {
        // GETALPHA
        todo!()
    }

    pub fn get_anchor(&self) -> anyhow::Result<&str> {
        // GETANCHOR STRING
        todo!()
    }

    pub fn get_center_x(&self) -> anyhow::Result<()> {
        // GETCENTERX
        todo!()
    }

    pub fn get_center_y(&self) -> anyhow::Result<()> {
        // GETCENTERY
        todo!()
    }

    pub fn get_cframe_in_event(&self) -> anyhow::Result<usize> {
        // GETCFRAMEINEVENT INTEGER
        todo!()
    }

    pub fn get_curr_frame_pos_x(&self) -> anyhow::Result<()> {
        // GETCURRFRAMEPOSX
        todo!()
    }

    pub fn get_curr_frame_pos_y(&self) -> anyhow::Result<()> {
        // GETCURRFRAMEPOSY
        todo!()
    }

    pub fn get_end_x(&self) -> anyhow::Result<()> {
        // GETENDX
        todo!()
    }

    pub fn get_end_y(&self) -> anyhow::Result<()> {
        // GETENDY
        todo!()
    }

    pub fn get_sequence_name(&self, context: RunnerContext) -> anyhow::Result<String> {
        // GETEVENTNAME
        let sequence = self.get_sequence_data(context)?;
        Ok(sequence.name.clone())
    }

    pub fn get_sequence_index(&self) -> anyhow::Result<()> {
        // GETEVENTNUMBER
        todo!()
    }

    pub fn get_fps(&self) -> anyhow::Result<()> {
        // GETFPS
        todo!()
    }

    pub fn get_frame(&self) -> anyhow::Result<usize> {
        // GETFRAME INTEGER
        todo!()
    }

    pub fn get_frame_name(&self) -> anyhow::Result<()> {
        // GETFRAMENAME
        todo!()
    }

    pub fn get_frame_index(&self) -> anyhow::Result<usize> {
        // GETFRAMENO INTEGER
        Ok(self.current_frame.frame_idx)
    }

    pub fn get_height(&self) -> anyhow::Result<()> {
        // GETHEIGHT
        todo!()
    }

    pub fn get_max_height(&self) -> anyhow::Result<()> {
        // GETMAXHEIGHT
        todo!()
    }

    pub fn get_max_width(&self) -> anyhow::Result<()> {
        // GETMAXWIDTH
        todo!()
    }

    pub fn get_sequence_count(&self) -> anyhow::Result<()> {
        // GETNOE
        todo!()
    }

    pub fn get_total_frame_count(&self) -> anyhow::Result<()> {
        // GETNOF
        todo!()
    }

    pub fn get_sequence_frame_count(&self, _sequence_name: &str) -> anyhow::Result<usize> {
        // GETNOFINEVENT INTEGER (STRING event)
        todo!()
    }

    pub fn get_opacity(&self) -> anyhow::Result<()> {
        // GETOPACITY
        todo!()
    }

    pub fn get_pixel(&self) -> anyhow::Result<()> {
        // GETPIXEL
        todo!()
    }

    pub fn get_frame_position_x(&self, context: RunnerContext) -> anyhow::Result<isize> {
        // GETPOSITIONX
        self.get_frame_position(context).map(|p| p.0)
    }

    pub fn get_frame_position_y(&self, context: RunnerContext) -> anyhow::Result<isize> {
        // GETPOSITIONY
        self.get_frame_position(context).map(|p| p.1)
    }

    pub fn get_priority(&self) -> anyhow::Result<isize> {
        // GETPRIORITY
        Ok(self.priority)
    }

    pub fn get_width(&self) -> anyhow::Result<()> {
        // GETWIDTH
        todo!()
    }

    pub fn hide(&mut self) -> anyhow::Result<()> {
        // HIDE
        self.is_visible = false;
        Ok(())
    }

    pub fn invalidate(&self) -> anyhow::Result<()> {
        // INVALIDATE
        todo!()
    }

    pub fn is_at(&self) -> anyhow::Result<()> {
        // ISAT
        todo!()
    }

    pub fn is_inside(&self) -> anyhow::Result<()> {
        // ISINSIDE
        todo!()
    }

    pub fn is_near(
        &self,
        context: RunnerContext,
        other: Arc<CnvObject>,
        min_iou_percent: usize,
    ) -> anyhow::Result<bool> {
        // ISNEAR
        let current_position = self.get_frame_position(context.clone())?;
        let current_size = self.get_frame_size(context.clone())?;
        let (other_position, other_size) = match &other.content {
            CnvContent::Animation(a) => (a.get_frame_position()?, a.get_frame_size()?),
            CnvContent::Image(i) => (i.get_position()?, i.get_size()?),
            _ => return Err(RunnerError::ExpectedGraphicsObject.into()),
        };
        let current_area = current_size.0 * current_size.1;
        let other_area = other_size.0 * other_size.1;
        if current_area == 0 || other_area == 0 {
            return Ok(false);
        } else if min_iou_percent == 0 {
            return Ok(true);
        } else if min_iou_percent > 100 {
            return Ok(false);
        }
        let current_top_left = current_position;
        let current_bottom_right = (
            current_position.0 + current_size.0 as isize,
            current_position.1 + current_size.1 as isize,
        );
        let other_top_left = other_position;
        let other_bottom_right = (
            other_position.0 + other_size.0 as isize,
            other_position.1 + other_size.1 as isize,
        );
        let intersection_top_left = (
            current_top_left.0.max(other_top_left.0),
            current_top_left.1.max(other_top_left.1),
        );
        let intersection_bottom_right = (
            current_bottom_right.0.min(other_bottom_right.0),
            current_bottom_right.1.min(other_bottom_right.1),
        );
        let intersection_area = if intersection_top_left.0 > intersection_bottom_right.0
            || intersection_top_left.1 > intersection_bottom_right.1
        {
            0
        } else {
            intersection_top_left
                .0
                .abs_diff(intersection_bottom_right.0)
                * intersection_top_left
                    .1
                    .abs_diff(intersection_bottom_right.1)
        };
        let union_area = current_area + other_area - intersection_area;
        Ok(intersection_area * 100 / union_area > min_iou_percent)
    }

    pub fn is_playing(&self) -> anyhow::Result<bool> {
        // ISPLAYING BOOL
        Ok(self.is_playing && !self.is_paused)
    }

    pub fn is_visible(&self) -> anyhow::Result<bool> {
        // ISVISIBLE
        Ok(self.is_visible)
    }

    pub fn load(&mut self, context: RunnerContext, filename: &str) -> anyhow::Result<()> {
        // LOAD
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .write()
            .unwrap()
            .read_scene_asset(
                Arc::clone(&script.runner.game_paths),
                &script.path.with_file_path(filename),
            )
            .map_err(|e| {
                self.file_data = Arc::new(AnimationFileData::Empty);
                RunnerError::IoError { source: e }
            })?;
        let data = parse_ann(&data);
        self.current_frame = FrameIdentifier {
            sequence_idx: data
                .sequences
                .iter()
                .position(|s| !s.frames.is_empty())
                .unwrap_or_default(),
            frame_idx: 0,
        };
        self.file_data = Arc::new(AnimationFileData::Loaded(LoadedAnimation {
            filename: Some(filename.to_owned()),
            sequences: data
                .sequences
                .into_iter()
                .map(|s| SequenceDefinition {
                    name: s.header.name.0,
                    opacity: s.header.opacity,
                    looping: s.header.looping,
                    frames: s
                        .frames
                        .into_iter()
                        .enumerate()
                        .map(|(i, f)| FrameDefinition {
                            name: f.name.0,
                            offset_px: (f.x_position_px.into(), f.y_position_px.into()),
                            opacity: f.opacity,
                            sprite_idx: s.header.frame_to_sprite_mapping[i].into(),
                            sfx: f
                                .random_sfx_list
                                .map(|d| d.as_ref().to_owned())
                                .unwrap_or_default(),
                        })
                        .collect(),
                })
                .collect(),
            sprites: data
                .sprites
                .into_iter()
                .map(|s| {
                    let converted_data = s
                        .image_data
                        .to_rgba8888(data.header.color_format, s.header.compression_type);
                    (
                        SpriteDefinition {
                            name: s.header.name.0,
                            size_px: (s.header.width_px.into(), s.header.height_px.into()),
                            offset_px: (
                                s.header.x_position_px.into(),
                                s.header.y_position_px.into(),
                            ),
                        },
                        SpriteData {
                            hash: xxh3_64(&converted_data),
                            data: converted_data,
                        },
                    )
                })
                .collect(),
        }));
        Ok(())
    }

    pub fn merge_alpha(&self) -> anyhow::Result<()> {
        // MERGEALPHA
        todo!()
    }

    pub fn monitor_collision(&mut self) -> anyhow::Result<()> {
        // MONITORCOLLISION
        self.does_monitor_collision = true;
        Ok(())
    }

    pub fn move_by(&mut self, x: isize, y: isize) -> anyhow::Result<()> {
        // MOVE
        self.position = (self.position.0 + x, self.position.1 + y);
        Ok(())
    }

    pub fn next_frame(&self) -> anyhow::Result<()> {
        // NEXTFRAME
        todo!()
    }

    pub fn n_play(&self) -> anyhow::Result<()> {
        // NPLAY
        todo!()
    }

    pub fn pause(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // PAUSE
        self.is_paused = true;
        let current_sequence_name = match *self.file_data {
            AnimationFileData::Loaded(LoadedAnimation { ref sequences, .. }) => sequences
                .get(self.current_frame.sequence_idx)
                .map(|s| s.name.clone()),
            _ => None,
        };
        let arguments = if let Some(current_sequence_name) = current_sequence_name {
            vec![CnvValue::String(current_sequence_name)]
        } else {
            Vec::new()
        };
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(arguments),
                    callable: CallableIdentifier::Event("ONPAUSED").to_owned(),
                });
            });
        Ok(())
    }

    pub fn play(&mut self, context: RunnerContext, sequence_name: &str) -> anyhow::Result<()> {
        // PLAY (STRING)
        self.load_if_needed(context.clone())?;
        let AnimationFileData::Loaded(ref loaded_data) = *self.file_data.clone() else {
            return Err(
                RunnerError::NoAnimationDataLoaded(context.current_object.name.clone()).into(),
            );
        };
        let (sequence_idx, sequence) = loaded_data
            .sequences
            .iter()
            .find_position(|s| s.name.eq_ignore_ascii_case(sequence_name))
            .ok_or(RunnerError::SequenceNameNotFound {
                object_name: context.current_object.name.clone(),
                sequence_name: sequence_name.to_owned(),
            })?;
        self.current_frame = FrameIdentifier {
            sequence_idx,
            frame_idx: 0,
        };
        self.is_playing = true;
        self.is_paused = false;
        self.is_reversed = false;
        if let Some(sfx) = sequence.frames[0].sfx.choose(&mut thread_rng()).cloned() {
            self.play_sfx(context.clone(), &sfx).ok_or_error();
        }
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context
                        .clone()
                        .with_arguments(vec![CnvValue::String(sequence_name.to_owned())]),
                    callable: CallableIdentifier::Event("ONSTARTED").to_owned(),
                });
                events.push_back(InternalEvent {
                    context: context
                        .clone()
                        .with_arguments(vec![CnvValue::String(sequence_name.to_owned())]),
                    callable: CallableIdentifier::Event("ONFIRSTFRAME").to_owned(),
                })
            });
        self.is_visible = true;
        Ok(())
    }

    pub fn play_rand(&self, _arg1: &str, _arg2: usize, _arg3: usize) -> anyhow::Result<()> {
        // PLAYRAND (STRING, INT, INT)
        todo!()
    }

    pub fn play_reverse(&self) -> anyhow::Result<()> {
        // PLAYREVERSE
        todo!()
    }

    pub fn prev_frame(&self) -> anyhow::Result<()> {
        // PREVFRAME
        todo!()
    }

    pub fn remove_monitor_collision(&mut self) -> anyhow::Result<()> {
        // REMOVEMONITORCOLLISION
        self.does_monitor_collision = false;
        Ok(())
    }

    pub fn replace_color(&self) -> anyhow::Result<()> {
        // REPLACECOLOR
        todo!()
    }

    pub fn reset_flips(&self) -> anyhow::Result<()> {
        // RESETFLIPS
        todo!()
    }

    pub fn resume(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // RESUME
        self.is_paused = false;
        let current_sequence_name = match *self.file_data {
            AnimationFileData::Loaded(LoadedAnimation { ref sequences, .. }) => sequences
                .get(self.current_frame.sequence_idx)
                .map(|s| s.name.clone()),
            _ => None,
        };
        let arguments = if let Some(current_sequence_name) = current_sequence_name {
            vec![CnvValue::String(current_sequence_name)]
        } else {
            Vec::new()
        };
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(arguments),
                    callable: CallableIdentifier::Event("ONRESUMED").to_owned(),
                });
            });
        Ok(())
    }

    pub fn set_anchor(&self, _arg1: &str) -> anyhow::Result<()> {
        // SETANCHOR (STRING)
        todo!()
    }

    pub fn set_as_button(
        &mut self,
        enabled: bool,
        should_show_pointer_on_hover: bool,
    ) -> anyhow::Result<()> {
        // SETASBUTTON (BOOL enabled, BOOL)
        if self.is_button != enabled {
            self.cursor_interaction = CursorInteraction::None;
            // TODO: should call ONFOCUSOFF and ONRELEASE events?
        }
        self.is_button = enabled;
        self.should_show_pointer_on_hover = should_show_pointer_on_hover;
        Ok(())
    }

    pub fn set_backward(&mut self) -> anyhow::Result<()> {
        // SETBACKWARD
        self.is_reversed = true;
        Ok(())
    }

    pub fn set_clipping(&self) -> anyhow::Result<()> {
        // SETCLIPPING
        todo!()
    }

    pub fn set_forward(&mut self) -> anyhow::Result<()> {
        // SETFORWARD
        self.is_reversed = false;
        Ok(())
    }

    pub fn set_fps(&mut self, fps: usize) -> anyhow::Result<()> {
        // SETFPS
        self.fps = fps;
        Ok(())
    }

    pub fn set_frame(
        &mut self,
        context: RunnerContext,
        sequence_name: Option<&str>,
        frame_no: usize,
    ) -> anyhow::Result<()> {
        // SETFRAME ([STRING], INTEGER)
        self.load_if_needed(context)?;
        let AnimationFileData::Loaded(ref loaded_data) = *self.file_data else {
            return Ok(());
        };
        if let Some(_sequence_name) = sequence_name {
            todo!()
        } else if loaded_data.sprites.len() > frame_no {
            self.sprite_idx_override = Some(frame_no);
        }
        Ok(())
    }

    pub fn set_frame_name(&self) -> anyhow::Result<()> {
        // SETFRAMENAME
        todo!()
    }

    pub fn set_freq(&self) -> anyhow::Result<()> {
        // SETFREQ
        todo!()
    }

    pub fn set_onff(&self) -> anyhow::Result<()> {
        // SETONFF
        todo!()
    }

    pub fn set_opacity(&self) -> anyhow::Result<()> {
        // SETOPACITY
        todo!()
    }

    pub fn set_position(&mut self, x: isize, y: isize) -> anyhow::Result<()> {
        // SETPOSITION
        self.position = (x, y);
        Ok(())
    }

    pub fn set_priority(&mut self, priority: isize) -> anyhow::Result<()> {
        // SETPRIORITY
        self.priority = priority;
        Ok(())
    }

    pub fn set_pan(&self) -> anyhow::Result<()> {
        // SETPAN
        todo!()
    }

    pub fn set_volume(&self) -> anyhow::Result<()> {
        // SETVOLUME
        todo!()
    }

    pub fn show(&mut self) -> anyhow::Result<()> {
        // SHOW
        self.is_visible = true;
        Ok(())
    }

    pub fn stop(&mut self, context: RunnerContext, emit_on_finished: bool) -> anyhow::Result<()> {
        // STOP ([BOOL])
        self.load_if_needed(context.clone())?;
        let AnimationFileData::Loaded(ref loaded_data) = *self.file_data.clone() else {
            return Err(
                RunnerError::NoAnimationDataLoaded(context.current_object.name.clone()).into(),
            );
        };
        if !self.is_playing {
            return Ok(());
        }
        let sequence = &loaded_data.sequences[self.current_frame.sequence_idx];
        self.current_frame = self.current_frame.with_frame_idx(0);
        self.is_playing = false;
        self.is_paused = false;
        self.is_reversed = false;
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundStopped(SoundSource::AnimationSfx {
                    script_path: context.current_object.parent.path.clone(),
                    object_name: context.current_object.name.clone(),
                }))
            });
        if emit_on_finished {
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        context: context
                            .clone()
                            .with_arguments(vec![CnvValue::String(sequence.name.clone())]),
                        callable: CallableIdentifier::Event("ONFINISHED").to_owned(),
                    })
                });
        }
        Ok(())
    }

    // custom

    fn get_max_frame_duration(&self) -> anyhow::Result<f64> {
        Ok(1f64 / (self.fps as f64))
    }

    pub fn get_base_position(&self) -> anyhow::Result<(isize, isize)> {
        Ok(self.position)
    }

    pub fn get_frame_position(&self, context: RunnerContext) -> anyhow::Result<(isize, isize)> {
        let (_, frame, sprite) = self.get_frame_sprite_data(context)?;
        Ok((
            self.position.0 + frame.offset_px.0 as isize + sprite.0.offset_px.0 as isize,
            self.position.1 + frame.offset_px.1 as isize + sprite.0.offset_px.1 as isize,
        ))
    }

    pub fn get_frame_size(&self, context: RunnerContext) -> anyhow::Result<(usize, usize)> {
        let (_, _, sprite) = self.get_frame_sprite_data(context)?;
        Ok((sprite.0.size_px.0 as usize, sprite.0.size_px.1 as usize))
    }

    pub fn get_frame_rect(&self, context: RunnerContext) -> anyhow::Result<Rect> {
        self.get_sprite_data(context).map(|d| d.0)
    }

    pub fn get_center_frame_position(
        &self,
        context: RunnerContext,
    ) -> anyhow::Result<(isize, isize)> {
        let (_, frame, sprite) = self.get_frame_sprite_data(context)?;
        Ok((
            self.position.0
                + frame.offset_px.0 as isize
                + sprite.0.offset_px.0 as isize
                + (sprite.0.size_px.0 / 2) as isize,
            self.position.1
                + frame.offset_px.1 as isize
                + sprite.0.offset_px.1 as isize
                + (sprite.0.size_px.1 / 2) as isize,
        ))
    }

    fn get_sequence_data(&self, context: RunnerContext) -> anyhow::Result<&SequenceDefinition> {
        let AnimationFileData::Loaded(ref loaded_file) = *self.file_data else {
            return Err(
                RunnerError::NoAnimationDataLoaded(context.current_object.name.clone()).into(),
            );
        };
        let Some(sequence) = loaded_file.sequences.get(self.current_frame.sequence_idx) else {
            return Err(RunnerError::SequenceIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: self.current_frame.sequence_idx,
            }
            .into());
        };
        Ok(sequence)
    }

    fn get_frame_data(
        &self,
        context: RunnerContext,
    ) -> anyhow::Result<(&SequenceDefinition, &FrameDefinition)> {
        let sequence = self.get_sequence_data(context.clone())?;
        let Some(frame) = sequence.frames.get(self.current_frame.frame_idx) else {
            return Err(RunnerError::FrameIndexNotFound {
                object_name: context.current_object.name.clone(),
                sequence_name: sequence.name.clone(),
                index: self.current_frame.frame_idx,
            }
            .into());
        };
        Ok((sequence, frame))
    }

    fn get_frame_sprite_data(
        &self,
        context: RunnerContext,
    ) -> anyhow::Result<(
        &SequenceDefinition,
        &FrameDefinition,
        &(SpriteDefinition, SpriteData),
    )> {
        let AnimationFileData::Loaded(ref loaded_file) = *self.file_data else {
            return Err(
                RunnerError::NoAnimationDataLoaded(context.current_object.name.clone()).into(),
            );
        };
        let (sequence, frame) = self.get_frame_data(context.clone())?;
        let Some(sprite) = loaded_file.sprites.get(frame.sprite_idx) else {
            return Err(RunnerError::SpriteIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: frame.sprite_idx,
            }
            .into());
        };
        Ok((sequence, frame, sprite))
    }

    fn get_sprite_data(&self, context: RunnerContext) -> anyhow::Result<(Rect, SpriteData)> {
        let AnimationFileData::Loaded(ref loaded_file) = *self.file_data else {
            return Err(
                RunnerError::NoAnimationDataLoaded(context.current_object.name.clone()).into(),
            );
        };
        if let Some(sprite_idx_override) = self.sprite_idx_override {
            let Some(sprite) = loaded_file.sprites.get(sprite_idx_override) else {
                return Err(RunnerError::SpriteIndexNotFound {
                    object_name: context.current_object.name.clone(),
                    index: sprite_idx_override,
                }
                .into());
            };
            let position = add_tuples(self.position, pair_i32_to_isize(sprite.0.offset_px));
            let size = pair_u32_to_usize(sprite.0.size_px);
            return Ok((Rect::from(position, size), sprite.1.clone()));
        };
        let (_, frame) = self.get_frame_data(context.clone())?;
        let Some(sprite) = loaded_file.sprites.get(frame.sprite_idx) else {
            return Err(RunnerError::SpriteIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: frame.sprite_idx,
            }
            .into());
        };
        let position = add_tuples(self.position, pair_i32_to_isize(sprite.0.offset_px));
        let position = add_tuples(position, pair_i32_to_isize(frame.offset_px));
        let size = pair_u32_to_usize(sprite.0.size_px);
        Ok((Rect::from(position, size), sprite.1.clone()))
    }

    pub fn step(&mut self, context: RunnerContext, seconds: f64) -> anyhow::Result<()> {
        let file_data = self.file_data.clone();
        let AnimationFileData::Loaded(ref loaded_data) = *file_data else {
            return Ok(());
        };
        if !self.is_playing || self.is_paused {
            return Ok(());
        }
        // log::trace!("Ticking animation {} with time {}, current frame: {:?}", animation.parent.name, duration, self.current_frame);
        self.sprite_idx_override = None;
        let sequence = &loaded_data.sequences[self.current_frame.sequence_idx];
        let sequence_looping = sequence.looping;
        let sequence_length = sequence.frames.len();
        let sequence_name = sequence.name.clone();
        self.current_frame_duration += seconds;
        let max_frame_duration = self.get_max_frame_duration()?;
        while self.current_frame_duration >= max_frame_duration {
            // log::trace!("{} / {}", self.current_frame_duration, max_frame_duration);
            self.current_frame_duration -= max_frame_duration;
            let prev_frame_idx = self.current_frame.frame_idx;
            let finished = if self.is_reversed {
                if self.current_frame.frame_idx == 0 {
                    true
                } else {
                    self.current_frame.frame_idx -= 1;
                    false
                } // TODO: looping after x
            } else {
                let limit = match sequence_looping {
                    LoopingSettings::LoopingAfter(frame_count) => frame_count,
                    LoopingSettings::NoLooping => sequence_length,
                }
                .saturating_sub(1);
                if self.current_frame.frame_idx == limit {
                    true
                } else {
                    self.current_frame.frame_idx += 1;
                    false
                }
            };
            if finished {
                self.is_playing = false;
                self.is_paused = false;
                self.is_reversed = false;
                context
                    .runner
                    .events_out
                    .sound
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(SoundEvent::SoundStopped(SoundSource::AnimationSfx {
                            script_path: context.current_object.parent.path.clone(),
                            object_name: context.current_object.name.clone(),
                        }))
                    });
                context
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(InternalEvent {
                            context: context
                                .clone()
                                .with_arguments(vec![CnvValue::String(sequence_name.clone())]),
                            callable: CallableIdentifier::Event("ONFINISHED").to_owned(),
                        })
                    });
            } else if self.current_frame.frame_idx != prev_frame_idx {
                if let Some(sfx) = sequence.frames[self.current_frame.frame_idx]
                    .sfx
                    .choose(&mut thread_rng())
                    .cloned()
                {
                    self.play_sfx(context.clone(), &sfx).ok_or_error();
                }
                context
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(InternalEvent {
                            context: context
                                .clone()
                                .with_arguments(vec![CnvValue::String(sequence_name.clone())]),
                            callable: CallableIdentifier::Event("ONFRAMECHANGED").to_owned(),
                        })
                    });
            }
        }
        // log::trace!("Moved animation {} to frame: {:?}", animation.parent.name, self.current_frame);
        Ok(())
    }

    fn load_sfx(&mut self, context: RunnerContext, path: &ScenePath) -> anyhow::Result<()> {
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .write()
            .unwrap()
            .read_sound(Arc::clone(&script.runner.game_paths), path)
            .map_err(|e| RunnerError::IoError { source: e })?;
        let converted_data: Arc<[u8]> = data.into();
        let sound_data = SoundData {
            hash: xxh3_64(&converted_data),
            data: converted_data,
        };
        self.current_sfx = SoundFileData::Loaded(LoadedSound {
            filename: Some(path.file_path.to_str()),
            sound: sound_data.clone(),
        });
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundLoaded {
                    source: SoundSource::AnimationSfx {
                        script_path: context.current_object.parent.path.clone(),
                        object_name: context.current_object.name.clone(),
                    },
                    sound_data,
                })
            });
        Ok(())
    }

    fn play_sfx(&mut self, context: RunnerContext, path: &str) -> anyhow::Result<()> {
        if !matches!(self.current_sfx, SoundFileData::Loaded(ref loaded) if loaded.filename.as_deref() == Some(path))
        {
            self.load_sfx(
                context.clone(),
                &context.current_object.parent.path.with_file_path(path),
            )?;
        }
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundStopped(SoundSource::AnimationSfx {
                    script_path: context.current_object.parent.path.clone(),
                    object_name: context.current_object.name.clone(),
                }));
                events.push_back(SoundEvent::SoundStarted(SoundSource::AnimationSfx {
                    script_path: context.current_object.parent.path.clone(),
                    object_name: context.current_object.name.clone(),
                }))
            });
        Ok(())
    }

    fn load_if_needed(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        if let AnimationFileData::NotLoaded(ref filename) = *self.file_data {
            let filename = filename.clone();
            self.load(context, &filename)?;
        };
        Ok(())
    }

    pub fn get_filename(&self) -> anyhow::Result<Option<String>> {
        Ok(match &*self.file_data {
            AnimationFileData::Empty => None,
            AnimationFileData::NotLoaded(filename) => Some(filename.to_owned()),
            AnimationFileData::Loaded(LoadedAnimation { filename, .. }) => filename.clone(),
        })
    }

    pub fn has_sequence(&self, context: RunnerContext, name: &str) -> anyhow::Result<bool> {
        let AnimationFileData::Loaded(ref loaded_file) = *self.file_data else {
            return Err(
                RunnerError::NoAnimationDataLoaded(context.current_object.name.clone()).into(),
            );
        };
        Ok(loaded_file.sequences.iter().any(|s| s.name == name))
    }
}
