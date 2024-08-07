use parsers::{discard_if_empty, parse_bool, parse_i32, parse_program};
use pixlib_formats::file_formats::ann::{parse_ann, LoopingSettings};
use std::{any::Any, cell::RefCell, sync::Arc};
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    ast::IgnorableProgram,
    common::DroppableRefMut,
    runner::{InternalEvent, RunnerError},
};

use super::*;

#[derive(Debug, Clone)]
pub struct AnimationProperties {
    // ANIMO
    pub as_button: Option<bool>,               // ASBUTTON [s]
    pub filename: Option<String>,              // FILENAME [s]
    pub flush_after_played: Option<bool>,      // FLUSHAFTERPLAYED []
    pub fps: Option<i32>,                      // FPS [gs]
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION [ss]
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA []
    pub preload: Option<bool>,                 // PRELOAD []
    pub priority: Option<i32>,                 // PRIORITY [gs]
    pub release: Option<bool>,                 // RELEASE []
    pub to_canvas: Option<bool>,               // TOCANVAS []
    pub visible: Option<bool>,                 // VISIBLE [gss]

    pub on_click: Option<Arc<IgnorableProgram>>, // ONCLICK signal
    pub on_collision: Option<Arc<IgnorableProgram>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<IgnorableProgram>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<IgnorableProgram>>,  // ONDONE signal
    pub on_finished: Option<Arc<IgnorableProgram>>, // ONFINISHED signal
    pub on_first_frame: Option<Arc<IgnorableProgram>>, // ONFIRSTFRAME signal
    pub on_focus_off: Option<Arc<IgnorableProgram>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<IgnorableProgram>>, // ONFOCUSON signal
    pub on_frame_changed: Option<Arc<IgnorableProgram>>, // ONFRAMECHANGED signal
    pub on_init: Option<Arc<IgnorableProgram>>,  // ONINIT signal
    pub on_paused: Option<Arc<IgnorableProgram>>, // ONPAUSED signal
    pub on_release: Option<Arc<IgnorableProgram>>, // ONRELEASE signal
    pub on_resumed: Option<Arc<IgnorableProgram>>, // ONRESUMED signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
    pub on_started: Option<Arc<IgnorableProgram>>, // ONSTARTED signal
}

#[derive(Debug, Clone, Default)]
struct AnimationState {
    pub initialized: bool,

    // initialized from properties
    pub is_button: bool,
    pub file_data: AnimationFileData,
    pub fps: usize,
    pub does_monitor_collision: bool,
    pub priority: isize,
    pub is_visible: bool,

    // general graphics state
    pub position: (i32, i32),
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
    pub on_click: Option<Arc<IgnorableProgram>>, // ONCLICK signal
    pub on_collision: Option<Arc<IgnorableProgram>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<IgnorableProgram>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<IgnorableProgram>>,  // ONDONE signal
    pub on_finished: Option<Arc<IgnorableProgram>>, // ONFINISHED signal
    pub on_first_frame: Option<Arc<IgnorableProgram>>, // ONFIRSTFRAME signal
    pub on_focus_off: Option<Arc<IgnorableProgram>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<IgnorableProgram>>, // ONFOCUSON signal
    pub on_frame_changed: Option<Arc<IgnorableProgram>>, // ONFRAMECHANGED signal
    pub on_init: Option<Arc<IgnorableProgram>>,  // ONINIT signal
    pub on_paused: Option<Arc<IgnorableProgram>>, // ONPAUSED signal
    pub on_release: Option<Arc<IgnorableProgram>>, // ONRELEASE signal
    pub on_resumed: Option<Arc<IgnorableProgram>>, // ONRESUMED signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
    pub on_started: Option<Arc<IgnorableProgram>>, // ONSTARTED signal
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
        let filename = props.filename;
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
        if let Some(filename) = filename {
            if animation.should_preload {
                animation
                    .state
                    .borrow_mut()
                    .load(&animation, &filename)
                    .unwrap();
            } else {
                animation.state.borrow_mut().file_data = AnimationFileData::NotLoaded(filename);
            }
        }
        animation
    }

    pub fn is_visible(&self) -> bool {
        self.state.borrow().is_visible()
    }

    ///

    pub fn tick(&self, duration: f64) -> RunnerResult<()> {
        self.state.borrow_mut().tick(&self, duration)
    }

    pub fn get_frame_to_show(
        &self,
    ) -> RunnerResult<Option<(FrameDefinition, SpriteDefinition, SpriteData)>> {
        // eprintln!("[ANIMO: {}] is_visible: {}", self.parent.name, self.is_visible);
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
        let frame = &sequence.frames[state.current_frame.frame_idx];
        let sprite = &loaded_data.sprites[frame.sprite_idx];
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

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONCLICK"
                | "ONCOLLISION"
                | "ONCOLLISIONFINISHED"
                | "ONDONE"
                | "ONFINISHED"
                | "ONFIRSTFRAME"
                | "ONFOCUSOFF"
                | "ONFOCUSON"
                | "ONFRAMECHANGED"
                | "ONINIT"
                | "ONPAUSED"
                | "ONRELEASE"
                | "ONRESUMED"
                | "ONSIGNAL"
                | "ONSTARTED"
        )
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: &mut RunnerContext,
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
                self.state.borrow_mut().get_alpha();
                Ok(None)
            }
            CallableIdentifier::Method("GETANCHOR") => {
                self.state.borrow_mut().get_anchor();
                Ok(None)
            }
            CallableIdentifier::Method("GETCENTERX") => {
                self.state.borrow_mut().get_center_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETCENTERY") => {
                self.state.borrow_mut().get_center_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETCFRAMEINEVENT") => {
                self.state.borrow_mut().get_cframe_in_event();
                Ok(None)
            }
            CallableIdentifier::Method("GETCURRFRAMEPOSX") => {
                self.state.borrow_mut().get_curr_frame_pos_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETCURRFRAMEPOSY") => {
                self.state.borrow_mut().get_curr_frame_pos_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETENDX") => {
                self.state.borrow_mut().get_end_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETENDY") => {
                self.state.borrow_mut().get_end_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETEVENTNAME") => {
                self.state.borrow_mut().get_event_name();
                Ok(None)
            }
            CallableIdentifier::Method("GETEVENTNUMBER") => {
                self.state.borrow_mut().get_event_number();
                Ok(None)
            }
            CallableIdentifier::Method("GETFPS") => {
                self.state.borrow_mut().get_fps();
                Ok(None)
            }
            CallableIdentifier::Method("GETFRAME") => {
                self.state.borrow_mut().get_frame();
                Ok(None)
            }
            CallableIdentifier::Method("GETFRAMENAME") => {
                self.state.borrow_mut().get_frame_name();
                Ok(None)
            }
            CallableIdentifier::Method("GETFRAMENO") => {
                self.state.borrow_mut().get_frame_no();
                Ok(None)
            }
            CallableIdentifier::Method("GETHEIGHT") => {
                self.state.borrow_mut().get_height();
                Ok(None)
            }
            CallableIdentifier::Method("GETMAXHEIGHT") => {
                self.state.borrow_mut().get_max_height();
                Ok(None)
            }
            CallableIdentifier::Method("GETMAXWIDTH") => {
                self.state.borrow_mut().get_max_width();
                Ok(None)
            }
            CallableIdentifier::Method("GETNOE") => {
                self.state.borrow_mut().get_noe();
                Ok(None)
            }
            CallableIdentifier::Method("GETNOF") => {
                self.state.borrow_mut().get_nof();
                Ok(None)
            }
            CallableIdentifier::Method("GETNOFINEVENT") => {
                self.state
                    .borrow_mut()
                    .get_nof_in_event(&arguments[0].to_string());
                Ok(None)
            }
            CallableIdentifier::Method("GETOPACITY") => {
                self.state.borrow_mut().get_opacity();
                Ok(None)
            }
            CallableIdentifier::Method("GETPIXEL") => {
                self.state.borrow_mut().get_pixel();
                Ok(None)
            }
            CallableIdentifier::Method("GETPOSITIONX") => {
                self.state.borrow_mut().get_position_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETPOSITIONY") => {
                self.state.borrow_mut().get_position_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETPRIORITY") => {
                self.state.borrow_mut().get_priority();
                Ok(None)
            }
            CallableIdentifier::Method("GETWIDTH") => {
                self.state.borrow_mut().get_width();
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
                self.state.borrow_mut().is_at();
                Ok(None)
            }
            CallableIdentifier::Method("ISINSIDE") => {
                self.state.borrow_mut().is_inside();
                Ok(None)
            }
            CallableIdentifier::Method("ISNEAR") => {
                self.state.borrow_mut().is_near();
                Ok(None)
            }
            CallableIdentifier::Method("ISPLAYING") => {
                self.state.borrow_mut().is_playing();
                Ok(None)
            }
            CallableIdentifier::Method("ISVISIBLE") => {
                self.state.borrow_mut().is_visible();
                Ok(None)
            }
            CallableIdentifier::Method("LOAD") => {
                self.state
                    .borrow_mut()
                    .load(&self, &arguments[0].to_string())?;
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
                self.state.borrow_mut().move_to();
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
                self.state.borrow_mut().pause(&self)?;
                Ok(None)
            }
            CallableIdentifier::Method("PLAY") => {
                self.state
                    .borrow_mut()
                    .play(&self, &arguments[0].to_string())?;
                Ok(None)
            }
            CallableIdentifier::Method("PLAYRAND") => {
                self.state.borrow_mut().play_rand(
                    &arguments[0].to_string(),
                    arguments[1].to_integer() as usize,
                    arguments[2].to_integer() as usize,
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
                self.state.borrow_mut().resume(&self)?;
                Ok(None)
            }
            CallableIdentifier::Method("SETANCHOR") => {
                self.state
                    .borrow_mut()
                    .set_anchor(&arguments[0].to_string());
                Ok(None)
            }
            CallableIdentifier::Method("SETASBUTTON") => {
                self.state
                    .borrow_mut()
                    .set_as_button(arguments[0].to_boolean(), arguments[1].to_boolean());
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
                    .set_fps(arguments[0].to_integer() as usize);
                Ok(None)
            }
            CallableIdentifier::Method("SETFRAME") => {
                self.state.borrow_mut().set_frame(
                    &arguments[0].to_string(),
                    arguments[1].to_integer() as usize,
                );
                Ok(None)
            }
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
            CallableIdentifier::Method("SETPOSITION") => {
                self.state.borrow_mut().set_position();
                Ok(None)
            }
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
                    arguments[0].to_boolean()
                });
                Ok(None)
            }
            CallableIdentifier::Event("ONCLICK") => {
                if let Some(v) = self.event_handlers.on_click.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONCOLLISION") => {
                if let Some(v) = self.event_handlers.on_collision.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONCOLLISIONFINISHED") => {
                if let Some(v) = self.event_handlers.on_collision_finished.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.event_handlers.on_done.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONFINISHED") => {
                if let Some(v) = self.event_handlers.on_finished.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONFIRSTFRAME") => {
                if let Some(v) = self.event_handlers.on_first_frame.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONFOCUSOFF") => {
                if let Some(v) = self.event_handlers.on_focus_off.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONFOCUSON") => {
                if let Some(v) = self.event_handlers.on_focus_on.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONFRAMECHANGED") => {
                if let Some(v) = self.event_handlers.on_frame_changed.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.event_handlers.on_init.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONPAUSED") => {
                if let Some(v) = self.event_handlers.on_paused.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONRELEASE") => {
                if let Some(v) = self.event_handlers.on_release.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONRESUMED") => {
                if let Some(v) = self.event_handlers.on_resumed.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONSIGNAL") => {
                if let Some(v) = self.event_handlers.on_signal.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONSTARTED") => {
                if let Some(v) = self.event_handlers.on_started.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            _ => todo!(),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
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
            .map(parse_program)
            .transpose()?;
        let on_collision = properties
            .remove("ONCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_collision_finished = properties
            .remove("ONCOLLISIONFINISHED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_finished = properties
            .remove("ONFINISHED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_first_frame = properties
            .remove("ONFIRSTFRAME")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_focus_off = properties
            .remove("ONFOCUSOFF")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_focus_on = properties
            .remove("ONFOCUSON")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_frame_changed = properties
            .remove("ONFRAMECHANGED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_paused = properties
            .remove("ONPAUSED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_release = properties
            .remove("ONRELEASE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_resumed = properties
            .remove("ONRESUMED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_started = properties
            .remove("ONSTARTED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Animation::from_initial_properties(
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
        ))
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

    pub fn get_event_name(&self) {
        // GETEVENTNAME
        todo!()
    }

    pub fn get_event_number(&self) {
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

    pub fn get_frame_no(&self) -> usize {
        // GETFRAMENO INTEGER
        todo!()
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

    pub fn get_noe(&self) {
        // GETNOE
        todo!()
    }

    pub fn get_nof(&self) {
        // GETNOF
        todo!()
    }

    pub fn get_nof_in_event(&self, _sequence_name: &str) -> usize {
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

    pub fn get_position_x(&self) {
        // GETPOSITIONX
        todo!()
    }

    pub fn get_position_y(&self) {
        // GETPOSITIONY
        todo!()
    }

    pub fn get_priority(&self) -> isize {
        // GETPRIORITY
        self.priority
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

    pub fn is_near(&self) {
        // ISNEAR
        todo!()
    }

    pub fn is_playing(&self) -> bool {
        // ISPLAYING BOOL
        todo!()
    }

    pub fn is_visible(&self) -> bool {
        // ISVISIBLE
        self.is_visible
    }

    pub fn load(&mut self, animation: &Animation, filename: &str) -> RunnerResult<()> {
        // LOAD
        let script = animation.parent.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .borrow()
            .read_scene_file(
                Arc::clone(&script.runner.game_paths),
                Some(script.path.with_file_name("").to_str().unwrap()),
                filename,
                Some("ANN"),
            )
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_ann(&data.0);
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

    pub fn monitor_collision(&self) {
        // MONITORCOLLISION
        todo!()
    }

    pub fn move_to(&self) {
        // MOVE
        todo!()
    }

    pub fn next_frame(&self) {
        // NEXTFRAME
        todo!()
    }

    pub fn n_play(&self) {
        // NPLAY
        todo!()
    }

    pub fn pause(&mut self, animation: &Animation) -> RunnerResult<()> {
        // PAUSE
        self.is_paused = true;
        animation
            .parent
            .parent
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    script_path: animation.parent.parent.path.clone(),
                    object_name: animation.parent.name.clone(),
                    event_name: "ONPAUSED".into(),
                    arguments: Vec::new(),
                })
            });
        Ok(())
    }

    pub fn play(&mut self, animation: &Animation, sequence_name: &str) -> RunnerResult<()> {
        // PLAY (STRING)
        let AnimationFileData::Loaded(loaded_data) = &self.file_data else {
            return Err(RunnerError::NoDataLoaded);
        };
        if self.is_playing
            && self.is_paused
            && loaded_data.sequences[self.current_frame.sequence_idx].name == sequence_name
        {
            // TODO: check if applicable
            self.is_paused = false;
            animation
                .parent
                .parent
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        script_path: animation.parent.parent.path.clone(),
                        object_name: animation.parent.name.clone(),
                        event_name: "ONRESUMED".into(),
                        arguments: Vec::new(),
                    })
                });
        } else {
            self.current_frame = FrameIdentifier {
                sequence_idx: loaded_data
                    .sequences
                    .iter()
                    .position(|s| s.name == sequence_name)
                    .ok_or(RunnerError::SequenceNameNotFound {
                        name: sequence_name.to_owned(),
                    })?,
                frame_idx: 0,
            };
            self.is_playing = true;
            self.is_paused = false;
            self.is_reversed = false;
            animation
                .parent
                .parent
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        script_path: animation.parent.parent.path.clone(),
                        object_name: animation.parent.name.clone(),
                        event_name: "ONSTARTED".into(),
                        arguments: Vec::new(),
                    });
                    events.push_back(InternalEvent {
                        script_path: animation.parent.parent.path.clone(),
                        object_name: animation.parent.name.clone(),
                        event_name: "ONFIRSTFRAME".into(),
                        arguments: Vec::new(),
                    })
                });
        }
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

    pub fn remove_monitor_collision(&self) {
        // REMOVEMONITORCOLLISION
        todo!()
    }

    pub fn replace_color(&self) {
        // REPLACECOLOR
        todo!()
    }

    pub fn reset_flips(&self) {
        // RESETFLIPS
        todo!()
    }

    pub fn resume(&mut self, animation: &Animation) -> RunnerResult<()> {
        // RESUME
        self.is_paused = false;
        animation
            .parent
            .parent
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    script_path: animation.parent.parent.path.clone(),
                    object_name: animation.parent.name.clone(),
                    event_name: "ONRESUMED".into(),
                    arguments: Vec::new(),
                })
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

    pub fn set_frame(&self, _sequence_name: &str, _frame_no: usize) {
        // SETFRAME (STRING, INTEGER)
        todo!()
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

    pub fn set_position(&self) {
        // SETPOSITION
        todo!()
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

    ///

    pub fn get_max_frame_duration(&self) -> f64 {
        1f64 / (self.fps as f64)
    }

    pub fn tick(&mut self, animation: &Animation, duration: f64) -> RunnerResult<()> {
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
        self.current_frame_duration += duration;
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
                animation
                    .parent
                    .parent
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(InternalEvent {
                            script_path: animation.parent.parent.path.clone(),
                            object_name: animation.parent.name.clone(),
                            event_name: "ONFINISHED".into(),
                            arguments: Vec::new(),
                        })
                    });
            } else {
                animation
                    .parent
                    .parent
                    .runner
                    .internal_events
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(InternalEvent {
                            script_path: animation.parent.parent.path.clone(),
                            object_name: animation.parent.name.clone(),
                            event_name: "ONFRAMECHANGED".into(),
                            arguments: Vec::new(),
                        })
                    });
            }
        }
        // eprintln!("Moved animation {} to frame: {:?}", animation.parent.name, self.current_frame);
        Ok(())
    }
}
