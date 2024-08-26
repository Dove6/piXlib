use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_bool, parse_event_handler, parse_i32};
use pixlib_formats::file_formats::img::parse_img;
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    common::DroppableRefMut,
    parser::ast::ParsedScript,
    runner::{InternalEvent, RunnerError},
};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ImageProperties {
    // IMAGE
    pub as_button: Option<bool>,               // ASBUTTON
    pub filename: Option<String>,              // FILENAME
    pub flush_after_played: Option<bool>,      // FLUSHAFTERPLAYED
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    pub preload: Option<bool>,                 // PRELOAD
    pub priority: Option<i32>,                 // PRIORITY
    pub release: Option<bool>,                 // RELEASE
    pub to_canvas: Option<bool>,               // TOCANVAS
    pub visible: Option<bool>,                 // VISIBLE

    pub on_click: Option<Arc<ParsedScript>>, // ONCLICK signal
    pub on_collision: Option<Arc<ParsedScript>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<ParsedScript>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<ParsedScript>>,  // ONDONE signal
    pub on_focus_off: Option<Arc<ParsedScript>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<ParsedScript>>, // ONFOCUSON signal
    pub on_init: Option<Arc<ParsedScript>>,  // ONINIT signal
    pub on_release: Option<Arc<ParsedScript>>, // ONRELEASE signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct ImageState {
    // initialized from properties
    pub is_button: bool,
    pub file_data: ImageFileData,
    pub does_monitor_collision: bool,
    pub priority: isize,
    pub is_visible: bool,

    // general graphics state
    pub position: (isize, isize),
    pub default_position: (isize, isize),
    pub opacity: usize,
    // anchor: ???,
    pub is_flipped_horizontally: bool,
    pub is_flipped_vertically: bool,
}

#[derive(Debug, Clone)]
pub struct ImageEventHandlers {
    pub on_click: Option<Arc<ParsedScript>>,     // ONCLICK signal
    pub on_collision: Option<Arc<ParsedScript>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<ParsedScript>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<ParsedScript>>,      // ONDONE signal
    pub on_focus_off: Option<Arc<ParsedScript>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<ParsedScript>>,  // ONFOCUSON signal
    pub on_init: Option<Arc<ParsedScript>>,      // ONINIT signal
    pub on_release: Option<Arc<ParsedScript>>,   // ONRELEASE signal
    pub on_signal: Option<Arc<ParsedScript>>,    // ONSIGNAL signal
}

impl EventHandler for ImageEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONCLICK" => self.on_click.as_ref(),
            "ONCOLLISION" => self.on_collision.as_ref(),
            "ONCOLLISIONFINISHED" => self.on_collision_finished.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONFOCUSOFF" => self.on_focus_off.as_ref(),
            "ONFOCUSON" => self.on_focus_on.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONRELEASE" => self.on_release.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    // IMAGE
    parent: Arc<CnvObject>,

    state: RefCell<ImageState>,
    event_handlers: ImageEventHandlers,

    should_flush_after_played: bool,
    should_collisions_respect_alpha: bool,
    should_preload: bool,
    should_release: bool,
    should_draw_to_canvas: bool,
}

impl Image {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: ImageProperties) -> Self {
        let image = Self {
            parent: parent.clone(),
            state: RefCell::new(ImageState {
                is_button: props.as_button.unwrap_or_default(),
                does_monitor_collision: props.monitor_collision.unwrap_or_default(),
                priority: props.priority.unwrap_or_default() as isize,
                is_visible: props.visible.unwrap_or(true),
                ..ImageState::default()
            }),
            event_handlers: ImageEventHandlers {
                on_click: props.on_click,
                on_collision: props.on_collision,
                on_collision_finished: props.on_collision_finished,
                on_done: props.on_done,
                on_focus_off: props.on_focus_off,
                on_focus_on: props.on_focus_on,
                on_init: props.on_init,
                on_release: props.on_release,
                on_signal: props.on_signal,
            },
            should_flush_after_played: props.flush_after_played.unwrap_or_default(),
            should_collisions_respect_alpha: props.monitor_collision_alpha.unwrap_or_default(),
            should_preload: props.preload.unwrap_or_default(),
            should_release: props.release.unwrap_or(true),
            should_draw_to_canvas: props.to_canvas.unwrap_or(true),
        };
        let filename = props.filename;
        if let Some(filename) = filename {
            image.state.borrow_mut().file_data = ImageFileData::NotLoaded(filename);
        }
        image
    }

    pub fn is_visible(&self) -> anyhow::Result<bool> {
        self.state.borrow().is_visible()
    }

    pub fn get_priority(&self) -> anyhow::Result<isize> {
        self.state.borrow().get_priority()
    }

    // custom

    pub fn get_position(&self) -> anyhow::Result<(isize, isize)> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context))?;
        self.state.borrow().get_position()
    }

    pub fn get_size(&self) -> anyhow::Result<(usize, usize)> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().get_size(context)
    }

    pub fn get_rect(&self) -> anyhow::Result<Rect> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().get_rect(context)
    }

    pub fn get_center_position(&self) -> anyhow::Result<(isize, isize)> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context.clone()))?;
        self.state.borrow().get_center_position(context)
    }

    pub fn does_monitor_collision(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().does_monitor_collision)
    }

    pub fn does_monitor_collision_pixel_perfect(&self) -> anyhow::Result<bool> {
        Ok(self.state.borrow().does_monitor_collision && self.should_collisions_respect_alpha)
    }

    pub fn get_image_to_show(&self) -> anyhow::Result<Option<(ImageDefinition, ImageData)>> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state
            .borrow_mut()
            .use_and_drop_mut(|s| s.load_if_needed(context))?;
        let state = self.state.borrow();
        if !state.is_visible {
            return Ok(None);
        }
        let ImageFileData::Loaded(loaded_data) = &state.file_data else {
            return Ok(None);
        };
        let image = &loaded_data.image;
        Ok(Some((image.0.clone(), image.1.clone())))
    }
}

impl GeneralGraphics for Image {
    fn hide(&self) -> anyhow::Result<()> {
        self.state.borrow_mut().hide()
    }

    fn show(&self) -> anyhow::Result<()> {
        self.state.borrow_mut().show()
    }
}

impl CnvType for Image {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "IMAGE"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
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
                self.state.borrow_mut().get_alpha().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETCENTERX") => self
                .state
                .borrow_mut()
                .get_center_x()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETCENTERY") => self
                .state
                .borrow_mut()
                .get_center_y()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETCOLORAT") => self
                .state
                .borrow_mut()
                .get_color_at()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETCOLORBAT") => self
                .state
                .borrow_mut()
                .get_color_b_at()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETCOLORGAT") => self
                .state
                .borrow_mut()
                .get_color_g_at()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETCOLORRAT") => self
                .state
                .borrow_mut()
                .get_color_r_at()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETHEIGHT") => {
                self.state.borrow_mut().get_height().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETOPACITY") => self
                .state
                .borrow_mut()
                .get_opacity()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETPIXEL") => {
                self.state.borrow_mut().get_pixel().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETPOSITIONX") => self
                .state
                .borrow()
                .get_position_x()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETPOSITIONY") => self
                .state
                .borrow()
                .get_position_y()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETPRIORITY") => self
                .state
                .borrow_mut()
                .get_priority()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETSLIDECOMPS") => self
                .state
                .borrow_mut()
                .get_slide_comps()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETWIDTH") => {
                self.state.borrow_mut().get_width().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("HIDE") => {
                self.state.borrow_mut().hide().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("INVALIDATE") => {
                self.state.borrow_mut().invalidate().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISAT") => {
                self.state.borrow_mut().is_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISINSIDE") => {
                self.state.borrow_mut().is_inside().map(|_| CnvValue::Null)
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
            CallableIdentifier::Method("ISVISIBLE") => {
                self.state.borrow_mut().is_visible().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("LINK") => {
                self.state.borrow_mut().link().map(|_| CnvValue::Null)
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
            CallableIdentifier::Method("MERGEALPHA2") => self
                .state
                .borrow_mut()
                .merge_alpha2()
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
                    context,
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| CnvValue::Null),
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
            CallableIdentifier::Method("RESETPOSITION") => self
                .state
                .borrow_mut()
                .reset_position()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SAVE") => {
                self.state.borrow_mut().save().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETANCHOR") => {
                self.state.borrow_mut().set_anchor().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETASBUTTON") => self
                .state
                .borrow_mut()
                .set_as_button()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETCLIPPING") => self
                .state
                .borrow_mut()
                .set_clipping()
                .map(|_| CnvValue::Null),
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
                .set_priority()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETRESETPOSITION") => self
                .state
                .borrow_mut()
                .set_reset_position(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETSCALEFACTOR") => self
                .state
                .borrow_mut()
                .set_scale_factor()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SHOW") => {
                self.state.borrow_mut().show().map(|_| CnvValue::Null)
            }
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
        let on_collision = properties
            .remove("ONCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_collision_finished = properties
            .remove("ONCOLLISIONFINISHED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
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
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_release = properties
            .remove("ONRELEASE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::Image(Image::from_initial_properties(
            parent,
            ImageProperties {
                as_button,
                filename,
                flush_after_played,
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
                on_focus_off,
                on_focus_on,
                on_init,
                on_release,
                on_signal,
            },
        )))
    }
}

impl Initable for Image {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        let mut state = self.state.borrow_mut();
        if self.should_preload {
            state.load_if_needed(context.clone())?;
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

impl ImageState {
    pub fn clear_clipping(&mut self) -> anyhow::Result<()> {
        // CLEARCLIPPING
        todo!()
    }

    pub fn draw_onto(&mut self) -> anyhow::Result<()> {
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

    pub fn get_alpha(&mut self) -> anyhow::Result<()> {
        // GETALPHA
        todo!()
    }

    pub fn get_center_x(&mut self) -> anyhow::Result<()> {
        // GETCENTERX
        todo!()
    }

    pub fn get_center_y(&mut self) -> anyhow::Result<()> {
        // GETCENTERY
        todo!()
    }

    pub fn get_color_at(&mut self) -> anyhow::Result<()> {
        // GETCOLORAT
        todo!()
    }

    pub fn get_color_b_at(&mut self) -> anyhow::Result<()> {
        // GETCOLORBAT
        todo!()
    }

    pub fn get_color_g_at(&mut self) -> anyhow::Result<()> {
        // GETCOLORGAT
        todo!()
    }

    pub fn get_color_r_at(&mut self) -> anyhow::Result<()> {
        // GETCOLORRAT
        todo!()
    }

    pub fn get_height(&mut self) -> anyhow::Result<()> {
        // GETHEIGHT
        todo!()
    }

    pub fn get_opacity(&mut self) -> anyhow::Result<()> {
        // GETOPACITY
        todo!()
    }

    pub fn get_pixel(&mut self) -> anyhow::Result<()> {
        // GETPIXEL
        todo!()
    }

    pub fn get_position_x(&self) -> anyhow::Result<isize> {
        // GETPOSITIONX
        Ok(self.position.0)
    }

    pub fn get_position_y(&self) -> anyhow::Result<isize> {
        // GETPOSITIONY
        Ok(self.position.1)
    }

    pub fn get_priority(&self) -> anyhow::Result<isize> {
        // GETPRIORITY
        Ok(self.priority)
    }

    pub fn get_slide_comps(&mut self) -> anyhow::Result<()> {
        // GETSLIDECOMPS
        todo!()
    }

    pub fn get_width(&mut self) -> anyhow::Result<()> {
        // GETWIDTH
        todo!()
    }

    pub fn hide(&mut self) -> anyhow::Result<()> {
        // HIDE
        self.is_visible = false;
        Ok(())
    }

    pub fn invalidate(&mut self) -> anyhow::Result<()> {
        // INVALIDATE
        todo!()
    }

    pub fn is_at(&mut self) -> anyhow::Result<()> {
        // ISAT
        todo!()
    }

    pub fn is_inside(&mut self) -> anyhow::Result<()> {
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
        let current_position = self.get_position()?;
        let current_size = self.get_size(context.clone())?;
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

    pub fn is_visible(&self) -> anyhow::Result<bool> {
        // ISVISIBLE
        Ok(self.is_visible)
    }

    pub fn link(&mut self) -> anyhow::Result<()> {
        // LINK
        todo!()
    }

    pub fn load(&mut self, context: RunnerContext, filename: &str) -> anyhow::Result<()> {
        // LOAD
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .borrow_mut()
            .read_scene_asset(
                Arc::clone(&script.runner.game_paths),
                &script.path.with_file_path(filename),
            )
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_img(&data);
        let converted_data = data
            .image_data
            .to_rgba8888(data.header.color_format, data.header.compression_type);
        self.default_position = (
            data.header.x_position_px as isize,
            data.header.y_position_px as isize,
        );
        self.position = self.default_position;
        self.file_data = ImageFileData::Loaded(LoadedImage {
            filename: Some(filename.to_owned()),
            image: (
                ImageDefinition {
                    size_px: (data.header.width_px, data.header.height_px),
                    offset_px: (data.header.x_position_px, data.header.y_position_px),
                },
                ImageData {
                    hash: xxh3_64(&converted_data),
                    data: converted_data,
                },
            ),
        });
        Ok(())
    }

    pub fn merge_alpha(&mut self) -> anyhow::Result<()> {
        // MERGEALPHA
        todo!()
    }

    pub fn merge_alpha2(&mut self) -> anyhow::Result<()> {
        // MERGEALPHA2
        todo!()
    }

    pub fn monitor_collision(&mut self) -> anyhow::Result<()> {
        // MONITORCOLLISION
        todo!()
    }

    pub fn move_by(&mut self, context: RunnerContext, x: isize, y: isize) -> anyhow::Result<()> {
        // MOVE
        self.load_if_needed(context)?;
        self.position = (self.position.0 + x, self.position.1 + y);
        Ok(())
    }

    pub fn remove_monitor_collision(&mut self) -> anyhow::Result<()> {
        // REMOVEMONITORCOLLISION
        todo!()
    }

    pub fn replace_color(&mut self) -> anyhow::Result<()> {
        // REPLACECOLOR
        todo!()
    }

    pub fn reset_flips(&mut self) -> anyhow::Result<()> {
        // RESETFLIPS
        todo!()
    }

    pub fn reset_position(&mut self) -> anyhow::Result<()> {
        // RESETPOSITION
        self.position = self.default_position;
        Ok(())
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        // SAVE
        todo!()
    }

    pub fn set_anchor(&mut self) -> anyhow::Result<()> {
        // SETANCHOR
        todo!()
    }

    pub fn set_as_button(&mut self) -> anyhow::Result<()> {
        // SETASBUTTON
        todo!()
    }

    pub fn set_clipping(&mut self) -> anyhow::Result<()> {
        // SETCLIPPING
        todo!()
    }

    pub fn set_opacity(&mut self) -> anyhow::Result<()> {
        // SETOPACITY
        todo!()
    }

    pub fn set_position(&mut self, x: isize, y: isize) -> anyhow::Result<()> {
        // SETPOSITION
        self.position = (x, y);
        Ok(())
    }

    pub fn set_priority(&mut self) -> anyhow::Result<()> {
        // SETPRIORITY
        todo!()
    }

    pub fn set_reset_position(&mut self, x: isize, y: isize) -> anyhow::Result<()> {
        // SETRESETPOSITION
        self.default_position = (x, y);
        Ok(())
    }

    pub fn set_scale_factor(&mut self) -> anyhow::Result<()> {
        // SETSCALEFACTOR
        todo!()
    }

    pub fn show(&mut self) -> anyhow::Result<()> {
        // SHOW
        self.is_visible = true;
        Ok(())
    }

    // custom

    pub fn get_position(&self) -> anyhow::Result<(isize, isize)> {
        Ok(self.position)
    }

    pub fn get_size(&self, context: RunnerContext) -> anyhow::Result<(usize, usize)> {
        let ImageFileData::Loaded(loaded_data) = &self.file_data else {
            return Err(RunnerError::NoImageDataLoaded(context.current_object.name.clone()).into());
        };
        let size = loaded_data.image.0.size_px;
        Ok((size.0 as usize, size.1 as usize))
    }

    pub fn get_rect(&self, context: RunnerContext) -> anyhow::Result<Rect> {
        let ImageFileData::Loaded(loaded_data) = &self.file_data else {
            return Err(RunnerError::NoImageDataLoaded(context.current_object.name.clone()).into());
        };
        let position = self.position;
        let size = (
            loaded_data.image.0.size_px.0 as isize,
            loaded_data.image.0.size_px.1 as isize,
        );
        Ok(Rect {
            top_left_x: position.0,
            top_left_y: position.1,
            bottom_right_x: position.0 + size.0,
            bottom_right_y: position.1 + size.1,
        })
    }

    pub fn get_center_position(&self, context: RunnerContext) -> anyhow::Result<(isize, isize)> {
        let ImageFileData::Loaded(loaded_data) = &self.file_data else {
            return Err(RunnerError::NoImageDataLoaded(context.current_object.name.clone()).into());
        };
        let position = self.position;
        let size = loaded_data.image.0.size_px;
        Ok((
            position.0 + (size.0 / 2) as isize,
            position.1 + (size.1 / 2) as isize,
        ))
    }

    pub fn load_if_needed(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        if let ImageFileData::NotLoaded(filename) = &self.file_data {
            let filename = filename.clone();
            self.load(context, &filename)?;
        };
        Ok(())
    }
}
