use content::EventHandler;
use initable::Initable;
use parsers::{discard_if_empty, parse_bool, parse_event_handler, parse_i32};
use pixlib_formats::file_formats::ann::{parse_ann, LoopingSettings};
use std::{any::Any, cell::RefCell, sync::Arc};
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    ast::ParsedScript,
    common::DroppableRefMut,
    runner::{InternalEvent, RunnerError},
};

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
    pub file_data: AnimationFileData,
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
    // more temporary
    pub current_frame_duration: f64,

    // related to sound
    pub panning: isize,
    pub volume: isize,
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
            parent: Arc::clone(&parent),
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
            animation.state.borrow_mut().file_data = AnimationFileData::NotLoaded(filename);
        }
        animation
    }

    pub fn is_visible(&self) -> RunnerResult<bool> {
        self.state.borrow().is_visible()
    }

    pub fn get_priority(&self) -> RunnerResult<isize> {
        self.state.borrow().get_priority()
    }

    // custom

    pub fn get_position(&self) -> RunnerResult<(isize, isize)> {
        Ok(self.state.borrow().position)
    }

    pub fn step(&self, seconds: f64) -> RunnerResult<()> {
        self.state.borrow_mut().step(
            RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent),
            seconds,
        )
    }

    pub fn get_frame_to_show(
        &self,
    ) -> RunnerResult<Option<(FrameDefinition, SpriteDefinition, SpriteData)>> {
        // eprintln!("[ANIMO: {}] is_visible: {}", self.parent.name, self.is_visible);
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        let state = self.state.borrow();
        if !state.is_visible {
            return Ok(None);
        }
        let AnimationFileData::Loaded(loaded_data) = &state.file_data else {
            return Ok(None);
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
            });
        };
        let Some(sprite) = loaded_data.sprites.get(frame.sprite_idx) else {
            return Err(RunnerError::SpriteIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: frame.sprite_idx,
            });
        };
        // eprintln!("[ANIMO: {}] [current frame] position: {:?} + {:?}, hash: {:?}", self.parent.name, sprite.0.offset_px, frame.offset_px, sprite.1.hash);
        Ok(Some((frame.clone(), sprite.0.clone(), sprite.1.clone())))
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
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("CLEARCLIPPING") => {
                self.state.borrow_mut().clear_clipping();
                Ok(None)
            }
            CallableIdentifier::Method("DRAWONTO") => {
                self.state.borrow_mut().draw_onto();
                Ok(None)
            }
            CallableIdentifier::Method("FLIPH") => {
                self.state.borrow_mut().flip_h();
                Ok(None)
            }
            CallableIdentifier::Method("FLIPV") => {
                self.state.borrow_mut().flip_v();
                Ok(None)
            }
            CallableIdentifier::Method("GETALPHA") => {
                self.state.borrow().get_alpha();
                Ok(None)
            }
            CallableIdentifier::Method("GETANCHOR") => {
                self.state.borrow().get_anchor();
                Ok(None)
            }
            CallableIdentifier::Method("GETCENTERX") => {
                self.state.borrow().get_center_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETCENTERY") => {
                self.state.borrow().get_center_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETCFRAMEINEVENT") => {
                self.state.borrow().get_cframe_in_event();
                Ok(None)
            }
            CallableIdentifier::Method("GETCURRFRAMEPOSX") => {
                self.state.borrow().get_curr_frame_pos_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETCURRFRAMEPOSY") => {
                self.state.borrow().get_curr_frame_pos_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETENDX") => {
                self.state.borrow().get_end_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETENDY") => {
                self.state.borrow().get_end_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETEVENTNAME") => self
                .state
                .borrow()
                .get_sequence_name(context)
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETEVENTNUMBER") => {
                self.state.borrow().get_sequence_index();
                Ok(None)
            }
            CallableIdentifier::Method("GETFPS") => {
                self.state.borrow().get_fps();
                Ok(None)
            }
            CallableIdentifier::Method("GETFRAME") => {
                self.state.borrow().get_frame();
                Ok(None)
            }
            CallableIdentifier::Method("GETFRAMENAME") => {
                self.state.borrow().get_frame_name();
                Ok(None)
            }
            CallableIdentifier::Method("GETFRAMENO") => self
                .state
                .borrow()
                .get_frame_index()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETHEIGHT") => {
                self.state.borrow().get_height();
                Ok(None)
            }
            CallableIdentifier::Method("GETMAXHEIGHT") => {
                self.state.borrow().get_max_height();
                Ok(None)
            }
            CallableIdentifier::Method("GETMAXWIDTH") => {
                self.state.borrow().get_max_width();
                Ok(None)
            }
            CallableIdentifier::Method("GETNOE") => {
                self.state.borrow().get_sequence_count();
                Ok(None)
            }
            CallableIdentifier::Method("GETNOF") => {
                self.state.borrow().get_total_frame_count();
                Ok(None)
            }
            CallableIdentifier::Method("GETNOFINEVENT") => {
                self.state
                    .borrow()
                    .get_sequence_frame_count(&arguments[0].to_str());
                Ok(None)
            }
            CallableIdentifier::Method("GETOPACITY") => {
                self.state.borrow().get_opacity();
                Ok(None)
            }
            CallableIdentifier::Method("GETPIXEL") => {
                self.state.borrow().get_pixel();
                Ok(None)
            }
            CallableIdentifier::Method("GETPOSITIONX") => self
                .state
                .borrow()
                .get_frame_position_x(context)
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPOSITIONY") => self
                .state
                .borrow()
                .get_frame_position_y(context)
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETPRIORITY") => self
                .state
                .borrow()
                .get_priority()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETWIDTH") => {
                self.state.borrow().get_width();
                Ok(None)
            }
            CallableIdentifier::Method("HIDE") => {
                self.state.borrow_mut().hide();
                Ok(None)
            }
            CallableIdentifier::Method("INVALIDATE") => {
                self.state.borrow_mut().invalidate();
                Ok(None)
            }
            CallableIdentifier::Method("ISAT") => {
                self.state.borrow().is_at();
                Ok(None)
            }
            CallableIdentifier::Method("ISINSIDE") => {
                self.state.borrow().is_inside();
                Ok(None)
            }
            CallableIdentifier::Method("ISNEAR") => {
                let name = arguments[0].to_str();
                let other = self
                    .parent
                    .parent
                    .runner
                    .get_object(&name)
                    .ok_or(RunnerError::ObjectNotFound { name })?;
                self.state
                    .borrow()
                    .is_near(other, arguments[1].to_int() as usize)
                    .map(|v| Some(CnvValue::Bool(v)))
            }
            CallableIdentifier::Method("ISPLAYING") => {
                self.state.borrow().is_playing();
                Ok(None)
            }
            CallableIdentifier::Method("ISVISIBLE") => self
                .state
                .borrow()
                .is_visible()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("LOAD") => {
                self.state
                    .borrow_mut()
                    .load(context, &arguments[0].to_str())?;
                Ok(None)
            }
            CallableIdentifier::Method("MERGEALPHA") => {
                self.state.borrow_mut().merge_alpha();
                Ok(None)
            }
            CallableIdentifier::Method("MONITORCOLLISION") => {
                self.state.borrow_mut().monitor_collision();
                Ok(None)
            }
            CallableIdentifier::Method("MOVE") => {
                self.state.borrow_mut().move_by(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )?;
                Ok(None)
            }
            CallableIdentifier::Method("NEXTFRAME") => {
                self.state.borrow_mut().next_frame();
                Ok(None)
            }
            CallableIdentifier::Method("NPLAY") => {
                self.state.borrow_mut().n_play();
                Ok(None)
            }
            CallableIdentifier::Method("PAUSE") => {
                self.state.borrow_mut().pause(context).map(|_| None)
            }
            CallableIdentifier::Method("PLAY") => self
                .state
                .borrow_mut()
                .play(context, &arguments[0].to_str())
                .map(|_| None),
            CallableIdentifier::Method("PLAYRAND") => {
                self.state.borrow_mut().play_rand(
                    &arguments[0].to_str(),
                    arguments[1].to_int() as usize,
                    arguments[2].to_int() as usize,
                );
                Ok(None)
            }
            CallableIdentifier::Method("PLAYREVERSE") => {
                self.state.borrow_mut().play_reverse();
                Ok(None)
            }
            CallableIdentifier::Method("PREVFRAME") => {
                self.state.borrow_mut().prev_frame();
                Ok(None)
            }
            CallableIdentifier::Method("REMOVEMONITORCOLLISION") => {
                self.state.borrow_mut().remove_monitor_collision();
                Ok(None)
            }
            CallableIdentifier::Method("REPLACECOLOR") => {
                self.state.borrow_mut().replace_color();
                Ok(None)
            }
            CallableIdentifier::Method("RESETFLIPS") => {
                self.state.borrow_mut().reset_flips();
                Ok(None)
            }
            CallableIdentifier::Method("RESUME") => {
                self.state.borrow_mut().resume(context).map(|_| None)
            }
            CallableIdentifier::Method("SETANCHOR") => {
                self.state.borrow_mut().set_anchor(&arguments[0].to_str());
                Ok(None)
            }
            CallableIdentifier::Method("SETASBUTTON") => {
                self.state
                    .borrow_mut()
                    .set_as_button(arguments[0].to_bool(), arguments[1].to_bool());
                Ok(None)
            }
            CallableIdentifier::Method("SETBACKWARD") => {
                self.state.borrow_mut().set_backward();
                Ok(None)
            }
            CallableIdentifier::Method("SETCLIPPING") => {
                self.state.borrow_mut().set_clipping();
                Ok(None)
            }
            CallableIdentifier::Method("SETFORWARD") => {
                self.state.borrow_mut().set_forward();
                Ok(None)
            }
            CallableIdentifier::Method("SETFPS") => {
                self.state
                    .borrow_mut()
                    .set_fps(arguments[0].to_int() as usize);
                Ok(None)
            }
            CallableIdentifier::Method("SETFRAME") => match arguments.len() {
                1 => self
                    .state
                    .borrow_mut()
                    .set_frame(None, arguments[0].to_int() as usize),
                2 => self
                    .state
                    .borrow_mut()
                    .set_frame(Some(&arguments[0].to_str()), arguments[1].to_int() as usize),
                0 => Err(RunnerError::TooFewArguments {
                    expected_min: 1,
                    actual: 0,
                }),
                arg_count => Err(RunnerError::TooManyArguments {
                    expected_max: 2,
                    actual: arg_count,
                }),
            }
            .map(|_| None),
            CallableIdentifier::Method("SETFRAMENAME") => {
                self.state.borrow_mut().set_frame_name();
                Ok(None)
            }
            CallableIdentifier::Method("SETFREQ") => {
                self.state.borrow_mut().set_freq();
                Ok(None)
            }
            CallableIdentifier::Method("SETONFF") => {
                self.state.borrow_mut().set_onff();
                Ok(None)
            }
            CallableIdentifier::Method("SETOPACITY") => {
                self.state.borrow_mut().set_opacity();
                Ok(None)
            }
            CallableIdentifier::Method("SETPOSITION") => self
                .state
                .borrow_mut()
                .set_position(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| None),
            CallableIdentifier::Method("SETPRIORITY") => {
                self.state.borrow_mut().set_priority();
                Ok(None)
            }
            CallableIdentifier::Method("SETPAN") => {
                self.state.borrow_mut().set_pan();
                Ok(None)
            }
            CallableIdentifier::Method("SETVOLUME") => {
                self.state.borrow_mut().set_volume();
                Ok(None)
            }
            CallableIdentifier::Method("SHOW") => {
                self.state.borrow_mut().show();
                Ok(None)
            }
            CallableIdentifier::Method("STOP") => {
                self.state.borrow_mut().stop(if arguments.is_empty() {
                    true
                } else {
                    arguments[0].to_bool()
                });
                Ok(None)
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
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
    fn initialize(&mut self, context: RunnerContext) -> RunnerResult<()> {
        let mut state = self.state.borrow_mut();
        if self.should_preload {
            if let AnimationFileData::NotLoaded(filename) = &state.file_data {
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
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                    arguments: Vec::new(),
                })
            });
        Ok(())
    }
}

impl AnimationState {
    pub fn clear_clipping(&self) {
        // CLEARCLIPPING
        todo!()
    }

    pub fn draw_onto(&self) {
        // DRAWONTO
        todo!()
    }

    pub fn flip_h(&mut self) {
        // FLIPH
        self.is_flipped_horizontally = !self.is_flipped_horizontally;
    }

    pub fn flip_v(&mut self) {
        // FLIPV
        self.is_flipped_vertically = !self.is_flipped_vertically;
    }

    pub fn get_alpha(&self) {
        // GETALPHA
        todo!()
    }

    pub fn get_anchor(&self) -> &str {
        // GETANCHOR STRING
        todo!()
    }

    pub fn get_center_x(&self) {
        // GETCENTERX
        todo!()
    }

    pub fn get_center_y(&self) {
        // GETCENTERY
        todo!()
    }

    pub fn get_cframe_in_event(&self) -> usize {
        // GETCFRAMEINEVENT INTEGER
        todo!()
    }

    pub fn get_curr_frame_pos_x(&self) {
        // GETCURRFRAMEPOSX
        todo!()
    }

    pub fn get_curr_frame_pos_y(&self) {
        // GETCURRFRAMEPOSY
        todo!()
    }

    pub fn get_end_x(&self) {
        // GETENDX
        todo!()
    }

    pub fn get_end_y(&self) {
        // GETENDY
        todo!()
    }

    pub fn get_sequence_name(&self, context: RunnerContext) -> RunnerResult<String> {
        // GETEVENTNAME
        let AnimationFileData::Loaded(loaded_file) = &self.file_data else {
            return Err(RunnerError::NoDataLoaded);
        };
        let Some(sequence) = loaded_file.sequences.get(self.current_frame.sequence_idx) else {
            return Err(RunnerError::SequenceIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: self.current_frame.sequence_idx,
            });
        };
        Ok(sequence.name.clone())
    }

    pub fn get_sequence_index(&self) {
        // GETEVENTNUMBER
        todo!()
    }

    pub fn get_fps(&self) {
        // GETFPS
        todo!()
    }

    pub fn get_frame(&self) -> usize {
        // GETFRAME INTEGER
        todo!()
    }

    pub fn get_frame_name(&self) {
        // GETFRAMENAME
        todo!()
    }

    pub fn get_frame_index(&self) -> RunnerResult<usize> {
        // GETFRAMENO INTEGER
        Ok(self.current_frame.frame_idx)
    }

    pub fn get_height(&self) {
        // GETHEIGHT
        todo!()
    }

    pub fn get_max_height(&self) {
        // GETMAXHEIGHT
        todo!()
    }

    pub fn get_max_width(&self) {
        // GETMAXWIDTH
        todo!()
    }

    pub fn get_sequence_count(&self) {
        // GETNOE
        todo!()
    }

    pub fn get_total_frame_count(&self) {
        // GETNOF
        todo!()
    }

    pub fn get_sequence_frame_count(&self, _sequence_name: &str) -> usize {
        // GETNOFINEVENT INTEGER (STRING event)
        todo!()
    }

    pub fn get_opacity(&self) {
        // GETOPACITY
        todo!()
    }

    pub fn get_pixel(&self) {
        // GETPIXEL
        todo!()
    }

    pub fn get_frame_position_x(&self, context: RunnerContext) -> RunnerResult<isize> {
        // GETPOSITIONX
        let AnimationFileData::Loaded(loaded_file) = &self.file_data else {
            return Err(RunnerError::NoDataLoaded);
        };
        let Some(sequence) = loaded_file.sequences.get(self.current_frame.sequence_idx) else {
            return Err(RunnerError::SequenceIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: self.current_frame.sequence_idx,
            });
        };
        let Some(frame) = sequence.frames.get(self.current_frame.frame_idx) else {
            return Err(RunnerError::FrameIndexNotFound {
                object_name: context.current_object.name.clone(),
                sequence_name: sequence.name.clone(),
                index: self.current_frame.frame_idx,
            });
        };
        let Some(sprite) = loaded_file.sprites.get(frame.sprite_idx) else {
            return Err(RunnerError::SpriteIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: frame.sprite_idx,
            });
        };
        Ok(self.position.0 + frame.offset_px.0 as isize + sprite.0.offset_px.0 as isize)
    }

    pub fn get_frame_position_y(&self, context: RunnerContext) -> RunnerResult<isize> {
        // GETPOSITIONY
        let AnimationFileData::Loaded(loaded_file) = &self.file_data else {
            return Err(RunnerError::NoDataLoaded);
        };
        let Some(sequence) = loaded_file.sequences.get(self.current_frame.sequence_idx) else {
            return Err(RunnerError::SequenceIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: self.current_frame.sequence_idx,
            });
        };
        let Some(frame) = sequence.frames.get(self.current_frame.frame_idx) else {
            return Err(RunnerError::FrameIndexNotFound {
                object_name: context.current_object.name.clone(),
                sequence_name: sequence.name.clone(),
                index: self.current_frame.frame_idx,
            });
        };
        let Some(sprite) = loaded_file.sprites.get(frame.sprite_idx) else {
            return Err(RunnerError::SpriteIndexNotFound {
                object_name: context.current_object.name.clone(),
                index: frame.sprite_idx,
            });
        };
        Ok(self.position.1 + frame.offset_px.1 as isize + sprite.0.offset_px.1 as isize)
    }

    pub fn get_priority(&self) -> RunnerResult<isize> {
        // GETPRIORITY
        Ok(self.priority)
    }

    pub fn get_width(&self) {
        // GETWIDTH
        todo!()
    }

    pub fn hide(&mut self) {
        // HIDE
        self.is_visible = false;
    }

    pub fn invalidate(&self) {
        // INVALIDATE
        todo!()
    }

    pub fn is_at(&self) {
        // ISAT
        todo!()
    }

    pub fn is_inside(&self) {
        // ISINSIDE
        todo!()
    }

    pub fn is_near(&self, other: Arc<CnvObject>, range: usize) -> RunnerResult<bool> {
        // ISNEAR
        let other_guard = other.content.borrow();
        let other: Option<&Animation> = (&*other_guard).into();
        let other = other.unwrap();
        let self_position = self.position;
        let other_position = other.get_position()?;
        Ok(self_position.0.abs_diff(other_position.0).pow(2)
            + self_position.1.abs_diff(other_position.1).pow(2)
            < range.pow(2))
    }

    pub fn is_playing(&self) -> bool {
        // ISPLAYING BOOL
        todo!()
    }

    pub fn is_visible(&self) -> RunnerResult<bool> {
        // ISVISIBLE
        Ok(self.is_visible)
    }

    pub fn load(&mut self, context: RunnerContext, filename: &str) -> RunnerResult<()> {
        // LOAD
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .borrow_mut()
            .read_scene_file(
                Arc::clone(&script.runner.game_paths),
                &script.path.with_file_path(filename),
            )
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_ann(&data);
        self.current_frame = FrameIdentifier {
            sequence_idx: data.sequences.len().saturating_sub(1),
            frame_idx: 0,
        };
        self.file_data = AnimationFileData::Loaded(LoadedAnimation {
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
        });
        Ok(())
    }

    pub fn merge_alpha(&self) {
        // MERGEALPHA
        todo!()
    }

    pub fn monitor_collision(&mut self) {
        // MONITORCOLLISION
        self.does_monitor_collision = true;
    }

    pub fn move_by(&mut self, x: isize, y: isize) -> RunnerResult<()> {
        // MOVE
        self.position = (self.position.0 + x, self.position.1 + y);
        Ok(())
    }

    pub fn next_frame(&self) {
        // NEXTFRAME
        todo!()
    }

    pub fn n_play(&self) {
        // NPLAY
        todo!()
    }

    pub fn pause(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // PAUSE
        self.is_paused = true;
        let current_sequence_name = match &self.file_data {
            AnimationFileData::Loaded(LoadedAnimation { sequences, .. }) => sequences
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
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONPAUSED").to_owned(),
                    arguments,
                });
            });
        Ok(())
    }

    pub fn play(&mut self, context: RunnerContext, sequence_name: &str) -> RunnerResult<()> {
        // PLAY (STRING)
        if let AnimationFileData::NotLoaded(filename) = &self.file_data {
            let filename = filename.clone();
            self.load(context.clone(), &filename)?;
        };
        let AnimationFileData::Loaded(loaded_data) = &self.file_data else {
            return Err(RunnerError::NoDataLoaded);
        };
        if self.is_playing
            && self.is_paused
            && loaded_data.sequences[self.current_frame.sequence_idx].name == sequence_name
        {
            // TODO: check if applicable
            self.is_paused = false;
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifier::Event("ONRESUMED").to_owned(),
                        arguments: vec![CnvValue::String(sequence_name.to_owned())],
                    })
                });
        } else {
            self.current_frame = FrameIdentifier {
                sequence_idx: loaded_data
                    .sequences
                    .iter()
                    .position(|s| s.name == sequence_name)
                    .ok_or(RunnerError::SequenceNameNotFound {
                        object_name: context.current_object.name.clone(),
                        sequence_name: sequence_name.to_owned(),
                    })?,
                frame_idx: 0,
            };
            self.is_playing = true;
            self.is_paused = false;
            self.is_reversed = false;
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifier::Event("ONSTARTED").to_owned(),
                        arguments: vec![CnvValue::String(sequence_name.to_owned())],
                    });
                    events.push_back(InternalEvent {
                        object: context.current_object.clone(),
                        callable: CallableIdentifier::Event("ONFIRSTFRAME").to_owned(),
                        arguments: vec![CnvValue::String(sequence_name.to_owned())],
                    })
                });
        }
        self.is_visible = true;
        Ok(())
    }

    pub fn play_rand(&self, _arg1: &str, _arg2: usize, _arg3: usize) {
        // PLAYRAND (STRING, INT, INT)
        todo!()
    }

    pub fn play_reverse(&self) {
        // PLAYREVERSE
        todo!()
    }

    pub fn prev_frame(&self) {
        // PREVFRAME
        todo!()
    }

    pub fn remove_monitor_collision(&mut self) {
        // REMOVEMONITORCOLLISION
        self.does_monitor_collision = false;
    }

    pub fn replace_color(&self) {
        // REPLACECOLOR
        todo!()
    }

    pub fn reset_flips(&self) {
        // RESETFLIPS
        todo!()
    }

    pub fn resume(&mut self, context: RunnerContext) -> RunnerResult<()> {
        // RESUME
        self.is_paused = false;
        let current_sequence_name = match &self.file_data {
            AnimationFileData::Loaded(LoadedAnimation { sequences, .. }) => sequences
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
                    object: context.current_object.clone(),
                    callable: CallableIdentifier::Event("ONRESUMED").to_owned(),
                    arguments,
                });
            });
        Ok(())
    }

    pub fn set_anchor(&self, _arg1: &str) {
        // SETANCHOR (STRING)
        todo!()
    }

    pub fn set_as_button(&self, _enabled: bool, _arg2: bool) {
        // SETASBUTTON (BOOL enabled, BOOL)
        todo!()
    }

    pub fn set_backward(&mut self) {
        // SETBACKWARD
        self.is_reversed = true;
    }

    pub fn set_clipping(&self) {
        // SETCLIPPING
        todo!()
    }

    pub fn set_forward(&mut self) {
        // SETFORWARD
        self.is_reversed = false;
    }

    pub fn set_fps(&mut self, fps: usize) {
        // SETFPS
        self.fps = fps;
    }

    pub fn set_frame(&mut self, sequence_name: Option<&str>, frame_no: usize) -> RunnerResult<()> {
        // SETFRAME ([STRING], INTEGER)
        if let Some(_sequence_name) = sequence_name {
            todo!()
        } else {
            self.current_frame.frame_idx = frame_no;
        }
        Ok(())
    }

    pub fn set_frame_name(&self) {
        // SETFRAMENAME
        todo!()
    }

    pub fn set_freq(&self) {
        // SETFREQ
        todo!()
    }

    pub fn set_onff(&self) {
        // SETONFF
        todo!()
    }

    pub fn set_opacity(&self) {
        // SETOPACITY
        todo!()
    }

    pub fn set_position(&mut self, x: isize, y: isize) -> RunnerResult<()> {
        // SETPOSITION
        self.position = (x, y);
        Ok(())
    }

    pub fn set_priority(&self) {
        // SETPRIORITY
        todo!()
    }

    pub fn set_pan(&self) {
        // SETPAN
        todo!()
    }

    pub fn set_volume(&self) {
        // SETVOLUME
        todo!()
    }

    pub fn show(&mut self) {
        // SHOW
        self.is_visible = true;
    }

    pub fn stop(&self, _emit_on_finished: bool) {
        // STOP ([BOOL])
        todo!()
    }

    // custom

    pub fn get_max_frame_duration(&self) -> f64 {
        1f64 / (self.fps as f64)
    }

    pub fn step(&mut self, context: RunnerContext, seconds: f64) -> RunnerResult<()> {
        let AnimationFileData::Loaded(loaded_data) = &self.file_data else {
            return Ok(());
        };
        if !self.is_playing || self.is_paused {
            return Ok(());
        }
        // eprintln!("Ticking animation {} with time {}, current frame: {:?}", animation.parent.name, duration, self.current_frame);
        let sequence = &loaded_data.sequences[self.current_frame.sequence_idx];
        let sequence_looping = sequence.looping;
        let sequence_length = sequence.frames.len();
        self.current_frame_duration += seconds;
        let max_frame_duration = self.get_max_frame_duration();
        while self.current_frame_duration >= max_frame_duration {
            // eprintln!("{} / {}", self.current_frame_duration, max_frame_duration);
            self.current_frame_duration -= max_frame_duration;
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
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(InternalEvent {
                            object: context.current_object.clone(),
                            callable: CallableIdentifier::Event("ONFINISHED").to_owned(),
                            arguments: vec![CnvValue::String(
                                loaded_data.sequences[self.current_frame.sequence_idx]
                                    .name
                                    .clone(),
                            )],
                        })
                    });
            } else {
                context
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(InternalEvent {
                            object: context.current_object.clone(),
                            callable: CallableIdentifier::Event("ONFRAMECHANGED").to_owned(),
                            arguments: vec![CnvValue::String(
                                loaded_data.sequences[self.current_frame.sequence_idx]
                                    .name
                                    .clone(),
                            )],
                        })
                    });
            }
        }
        // eprintln!("Moved animation {} to frame: {:?}", animation.parent.name, self.current_frame);
        Ok(())
    }
}
