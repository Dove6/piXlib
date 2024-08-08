use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_i32, parse_program};
use pixlib_formats::file_formats::img::parse_img;
use xxhash_rust::xxh3::xxh3_64;

use crate::{ast::ParsedScript, runner::RunnerError};

use super::*;

#[derive(Debug, Clone)]
pub struct ImageInit {
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
    pub initialized: bool,

    // initialized from properties
    pub is_button: bool,
    pub file_data: ImageFileData,
    pub does_monitor_collision: bool,
    pub priority: isize,
    pub is_visible: bool,

    // general graphics state
    pub position: (isize, isize),
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
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: ImageInit) -> Self {
        let filename = props.filename;
        let image = Self {
            parent: Arc::clone(&parent),
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
        if let Some(filename) = filename {
            if image.should_preload {
                image.state.borrow_mut().load(&image, &filename).unwrap();
            } else {
                image.state.borrow_mut().file_data = ImageFileData::NotLoaded(filename);
            }
        }
        image
    }

    pub fn is_visible(&self) -> RunnerResult<bool> {
        self.state.borrow().is_visible()
    }

    pub fn get_priority(&self) -> RunnerResult<isize> {
        self.state.borrow().get_priority()
    }

    ///

    pub fn get_position(&self) -> RunnerResult<(isize, isize)> {
        Ok(self.state.borrow().position)
    }

    pub fn get_image_to_show(&self) -> RunnerResult<Option<(ImageDefinition, ImageData)>> {
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

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONCLICK"
                | "ONCOLLISION"
                | "ONCOLLISIONFINISHED"
                | "ONDONE"
                | "ONFOCUSOFF"
                | "ONFOCUSON"
                | "ONINIT"
                | "ONRELEASE"
                | "ONSIGNAL"
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
                self.state.borrow_mut().get_alpha();
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
            CallableIdentifier::Method("GETCOLORAT") => {
                self.state.borrow_mut().get_color_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETCOLORBAT") => {
                self.state.borrow_mut().get_color_b_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETCOLORGAT") => {
                self.state.borrow_mut().get_color_g_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETCOLORRAT") => {
                self.state.borrow_mut().get_color_r_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETHEIGHT") => {
                self.state.borrow_mut().get_height();
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
            CallableIdentifier::Method("GETPRIORITY") => self
                .state
                .borrow_mut()
                .get_priority()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETSLIDECOMPS") => {
                self.state.borrow_mut().get_slide_comps();
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
            CallableIdentifier::Method("ISVISIBLE") => self
                .state
                .borrow_mut()
                .is_visible()
                .map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Method("LINK") => {
                self.state.borrow_mut().link();
                Ok(None)
            }
            CallableIdentifier::Method("LOAD") => {
                self.state
                    .borrow_mut()
                    .load(self, &arguments[0].to_string())?;
                Ok(None)
            }
            CallableIdentifier::Method("MERGEALPHA") => {
                self.state.borrow_mut().merge_alpha();
                Ok(None)
            }
            CallableIdentifier::Method("MERGEALPHA2") => {
                self.state.borrow_mut().merge_alpha2();
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
            CallableIdentifier::Method("RESETPOSITION") => {
                self.state.borrow_mut().reset_position();
                Ok(None)
            }
            CallableIdentifier::Method("SAVE") => {
                self.state.borrow_mut().save();
                Ok(None)
            }
            CallableIdentifier::Method("SETANCHOR") => {
                self.state.borrow_mut().set_anchor();
                Ok(None)
            }
            CallableIdentifier::Method("SETASBUTTON") => {
                self.state.borrow_mut().set_as_button();
                Ok(None)
            }
            CallableIdentifier::Method("SETCLIPPING") => {
                self.state.borrow_mut().set_clipping();
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
            CallableIdentifier::Method("SETRESETPOSITION") => {
                self.state.borrow_mut().set_reset_position();
                Ok(None)
            }
            CallableIdentifier::Method("SETSCALEFACTOR") => {
                self.state.borrow_mut().set_scale_factor();
                Ok(None)
            }
            CallableIdentifier::Method("SHOW") => {
                self.state.borrow_mut().show();
                Ok(None)
            }
            CallableIdentifier::Event("ONCLICK") => {
                if let Some(v) = self.event_handlers.on_click.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONCOLLISION") => {
                if let Some(v) = self.event_handlers.on_collision.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONCOLLISIONFINISHED") => {
                if let Some(v) = self.event_handlers.on_collision_finished.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.event_handlers.on_done.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONFOCUSOFF") => {
                if let Some(v) = self.event_handlers.on_focus_off.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONFOCUSON") => {
                if let Some(v) = self.event_handlers.on_focus_on.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.event_handlers.on_init.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONRELEASE") => {
                if let Some(v) = self.event_handlers.on_release.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONSIGNAL") => {
                if let Some(v) = self.event_handlers.on_signal.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
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
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_release = properties
            .remove("ONRELEASE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(CnvContent::Image(Image::from_initial_properties(
            parent,
            ImageInit {
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

impl ImageState {
    pub fn clear_clipping(&self) {
        todo!()
    }

    pub fn draw_onto(&self) {
        todo!()
    }

    pub fn flip_h(&mut self) {
        self.is_flipped_horizontally = !self.is_flipped_horizontally;
    }

    pub fn flip_v(&mut self) {
        self.is_flipped_vertically = !self.is_flipped_vertically;
    }

    pub fn get_alpha(&self) {
        todo!()
    }

    pub fn get_center_x(&self) {
        todo!()
    }

    pub fn get_center_y(&self) {
        todo!()
    }

    pub fn get_color_at(&self) {
        todo!()
    }

    pub fn get_color_b_at(&self) {
        todo!()
    }

    pub fn get_color_g_at(&self) {
        todo!()
    }

    pub fn get_color_r_at(&self) {
        todo!()
    }

    pub fn get_height(&self) {
        todo!()
    }

    pub fn get_opacity(&self) {
        todo!()
    }

    pub fn get_pixel(&self) {
        todo!()
    }

    pub fn get_position_x(&self) {
        todo!()
    }

    pub fn get_position_y(&self) {
        todo!()
    }

    pub fn get_priority(&self) -> RunnerResult<isize> {
        Ok(self.priority)
    }

    pub fn get_slide_comps(&self) {
        todo!()
    }

    pub fn get_width(&self) {
        todo!()
    }

    pub fn hide(&mut self) {
        self.is_visible = false;
    }

    pub fn invalidate(&self) {
        todo!()
    }

    pub fn is_at(&self) {
        todo!()
    }

    pub fn is_inside(&self) {
        todo!()
    }

    pub fn is_near(&self) {
        todo!()
    }

    pub fn is_visible(&self) -> RunnerResult<bool> {
        Ok(self.is_visible)
    }

    pub fn link(&self) {
        todo!()
    }

    pub fn load(&mut self, image: &Image, filename: &str) -> RunnerResult<()> {
        let script = image.parent.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .borrow()
            .read_scene_file(
                Arc::clone(&script.runner.game_paths),
                Some(script.path.with_file_name("").to_str().unwrap()),
                filename,
                Some("IMG"),
            )
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_img(&data.0);
        let converted_data = data
            .image_data
            .to_rgba8888(data.header.color_format, data.header.compression_type);
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

    pub fn merge_alpha(&self) {
        todo!()
    }

    pub fn merge_alpha2(&self) {
        todo!()
    }

    pub fn monitor_collision(&self) {
        todo!()
    }

    pub fn move_to(&self) {
        todo!()
    }

    pub fn remove_monitor_collision(&self) {
        todo!()
    }

    pub fn replace_color(&self) {
        todo!()
    }

    pub fn reset_flips(&self) {
        todo!()
    }

    pub fn reset_position(&self) {
        todo!()
    }

    pub fn save(&self) {
        todo!()
    }

    pub fn set_anchor(&self) {
        todo!()
    }

    pub fn set_as_button(&self) {
        todo!()
    }

    pub fn set_clipping(&self) {
        todo!()
    }

    pub fn set_opacity(&self) {
        todo!()
    }

    pub fn set_position(&self) {
        todo!()
    }

    pub fn set_priority(&self) {
        todo!()
    }

    pub fn set_reset_position(&self) {
        todo!()
    }

    pub fn set_scale_factor(&self) {
        todo!()
    }

    pub fn show(&mut self) {
        self.is_visible = true;
    }
}
