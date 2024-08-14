use std::{any::Any, cell::RefCell};

use content::EventHandler;
use initable::Initable;
use parsers::{discard_if_empty, parse_bool, parse_event_handler, parse_i32};
use pixlib_formats::file_formats::img::parse_img;
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    ast::ParsedScript,
    common::DroppableRefMut,
    runner::{InternalEvent, RunnerError},
};

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
            image.state.borrow_mut().file_data = ImageFileData::NotLoaded(filename);
        }
        image
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("CLEARCLIPPING") => {
                self.state.borrow_mut().clear_clipping().map(|_| None)
            }
            CallableIdentifier::Method("DRAWONTO") => {
                self.state.borrow_mut().draw_onto().map(|_| None)
            }
            CallableIdentifier::Method("FLIPH") => self.state.borrow_mut().flip_h().map(|_| None),
            CallableIdentifier::Method("FLIPV") => self.state.borrow_mut().flip_v().map(|_| None),
            CallableIdentifier::Method("GETALPHA") => {
                self.state.borrow_mut().get_alpha().map(|_| None)
            }
            CallableIdentifier::Method("GETCENTERX") => {
                self.state.borrow_mut().get_center_x().map(|_| None)
            }
            CallableIdentifier::Method("GETCENTERY") => {
                self.state.borrow_mut().get_center_y().map(|_| None)
            }
            CallableIdentifier::Method("GETCOLORAT") => {
                self.state.borrow_mut().get_color_at().map(|_| None)
            }
            CallableIdentifier::Method("GETCOLORBAT") => {
                self.state.borrow_mut().get_color_b_at().map(|_| None)
            }
            CallableIdentifier::Method("GETCOLORGAT") => {
                self.state.borrow_mut().get_color_g_at().map(|_| None)
            }
            CallableIdentifier::Method("GETCOLORRAT") => {
                self.state.borrow_mut().get_color_r_at().map(|_| None)
            }
            CallableIdentifier::Method("GETHEIGHT") => {
                self.state.borrow_mut().get_height().map(|_| None)
            }
            CallableIdentifier::Method("GETOPACITY") => {
                self.state.borrow_mut().get_opacity().map(|_| None)
            }
            CallableIdentifier::Method("GETPIXEL") => {
                self.state.borrow_mut().get_pixel().map(|_| None)
            }
            CallableIdentifier::Method("GETPOSITIONX") => {
                self.state.borrow_mut().get_position_x().map(|_| None)
            }
            CallableIdentifier::Method("GETPOSITIONY") => {
                self.state.borrow_mut().get_position_y().map(|_| None)
            }
            CallableIdentifier::Method("GETPRIORITY") => self
                .state
                .borrow_mut()
                .get_priority()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETSLIDECOMPS") => {
                self.state.borrow_mut().get_slide_comps().map(|_| None)
            }
            CallableIdentifier::Method("GETWIDTH") => {
                self.state.borrow_mut().get_width().map(|_| None)
            }
            CallableIdentifier::Method("HIDE") => self.state.borrow_mut().hide().map(|_| None),
            CallableIdentifier::Method("INVALIDATE") => {
                self.state.borrow_mut().invalidate().map(|_| None)
            }
            CallableIdentifier::Method("ISAT") => self.state.borrow_mut().is_at().map(|_| None),
            CallableIdentifier::Method("ISINSIDE") => {
                self.state.borrow_mut().is_inside().map(|_| None)
            }
            CallableIdentifier::Method("ISNEAR") => self.state.borrow_mut().is_near().map(|_| None),
            CallableIdentifier::Method("ISVISIBLE") => self
                .state
                .borrow_mut()
                .is_visible()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("LINK") => self.state.borrow_mut().link().map(|_| None),
            CallableIdentifier::Method("LOAD") => self
                .state
                .borrow_mut()
                .load(context, &arguments[0].to_str())
                .map(|_| None),
            CallableIdentifier::Method("MERGEALPHA") => {
                self.state.borrow_mut().merge_alpha().map(|_| None)
            }
            CallableIdentifier::Method("MERGEALPHA2") => {
                self.state.borrow_mut().merge_alpha2().map(|_| None)
            }
            CallableIdentifier::Method("MONITORCOLLISION") => {
                self.state.borrow_mut().monitor_collision().map(|_| None)
            }
            CallableIdentifier::Method("MOVE") => self
                .state
                .borrow_mut()
                .move_by(
                    arguments[0].to_int() as isize,
                    arguments[1].to_int() as isize,
                )
                .map(|_| None),
            CallableIdentifier::Method("REMOVEMONITORCOLLISION") => self
                .state
                .borrow_mut()
                .remove_monitor_collision()
                .map(|_| None),
            CallableIdentifier::Method("REPLACECOLOR") => {
                self.state.borrow_mut().replace_color().map(|_| None)
            }
            CallableIdentifier::Method("RESETFLIPS") => {
                self.state.borrow_mut().reset_flips().map(|_| None)
            }
            CallableIdentifier::Method("RESETPOSITION") => {
                self.state.borrow_mut().reset_position().map(|_| None)
            }
            CallableIdentifier::Method("SAVE") => self.state.borrow_mut().save().map(|_| None),
            CallableIdentifier::Method("SETANCHOR") => {
                self.state.borrow_mut().set_anchor().map(|_| None)
            }
            CallableIdentifier::Method("SETASBUTTON") => {
                self.state.borrow_mut().set_as_button().map(|_| None)
            }
            CallableIdentifier::Method("SETCLIPPING") => {
                self.state.borrow_mut().set_clipping().map(|_| None)
            }
            CallableIdentifier::Method("SETOPACITY") => {
                self.state.borrow_mut().set_opacity().map(|_| None)
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
                self.state.borrow_mut().set_priority().map(|_| None)
            }
            CallableIdentifier::Method("SETRESETPOSITION") => {
                self.state.borrow_mut().set_reset_position().map(|_| None)
            }
            CallableIdentifier::Method("SETSCALEFACTOR") => {
                self.state.borrow_mut().set_scale_factor().map(|_| None)
            }
            CallableIdentifier::Method("SHOW") => self.state.borrow_mut().show().map(|_| None),
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
    fn initialize(&mut self, context: RunnerContext) -> RunnerResult<()> {
        let mut state = self.state.borrow_mut();
        if self.should_preload {
            if let ImageFileData::NotLoaded(filename) = &state.file_data {
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

impl ImageState {
    pub fn clear_clipping(&mut self) -> RunnerResult<()> {
        // CLEARCLIPPING
        todo!()
    }

    pub fn draw_onto(&mut self) -> RunnerResult<()> {
        // DRAWONTO
        todo!()
    }

    pub fn flip_h(&mut self) -> RunnerResult<()> {
        // FLIPH
        self.is_flipped_horizontally = !self.is_flipped_horizontally;
        Ok(())
    }

    pub fn flip_v(&mut self) -> RunnerResult<()> {
        // FLIPV
        self.is_flipped_vertically = !self.is_flipped_vertically;
        Ok(())
    }

    pub fn get_alpha(&mut self) -> RunnerResult<()> {
        // GETALPHA
        todo!()
    }

    pub fn get_center_x(&mut self) -> RunnerResult<()> {
        // GETCENTERX
        todo!()
    }

    pub fn get_center_y(&mut self) -> RunnerResult<()> {
        // GETCENTERY
        todo!()
    }

    pub fn get_color_at(&mut self) -> RunnerResult<()> {
        // GETCOLORAT
        todo!()
    }

    pub fn get_color_b_at(&mut self) -> RunnerResult<()> {
        // GETCOLORBAT
        todo!()
    }

    pub fn get_color_g_at(&mut self) -> RunnerResult<()> {
        // GETCOLORGAT
        todo!()
    }

    pub fn get_color_r_at(&mut self) -> RunnerResult<()> {
        // GETCOLORRAT
        todo!()
    }

    pub fn get_height(&mut self) -> RunnerResult<()> {
        // GETHEIGHT
        todo!()
    }

    pub fn get_opacity(&mut self) -> RunnerResult<()> {
        // GETOPACITY
        todo!()
    }

    pub fn get_pixel(&mut self) -> RunnerResult<()> {
        // GETPIXEL
        todo!()
    }

    pub fn get_position_x(&mut self) -> RunnerResult<()> {
        // GETPOSITIONX
        todo!()
    }

    pub fn get_position_y(&mut self) -> RunnerResult<()> {
        // GETPOSITIONY
        todo!()
    }

    pub fn get_priority(&self) -> RunnerResult<isize> {
        // GETPRIORITY
        Ok(self.priority)
    }

    pub fn get_slide_comps(&mut self) -> RunnerResult<()> {
        // GETSLIDECOMPS
        todo!()
    }

    pub fn get_width(&mut self) -> RunnerResult<()> {
        // GETWIDTH
        todo!()
    }

    pub fn hide(&mut self) -> RunnerResult<()> {
        // HIDE
        self.is_visible = false;
        Ok(())
    }

    pub fn invalidate(&mut self) -> RunnerResult<()> {
        // INVALIDATE
        todo!()
    }

    pub fn is_at(&mut self) -> RunnerResult<()> {
        // ISAT
        todo!()
    }

    pub fn is_inside(&mut self) -> RunnerResult<()> {
        // ISINSIDE
        todo!()
    }

    pub fn is_near(&mut self) -> RunnerResult<()> {
        // ISNEAR
        todo!()
    }

    pub fn is_visible(&self) -> RunnerResult<bool> {
        // ISVISIBLE
        Ok(self.is_visible)
    }

    pub fn link(&mut self) -> RunnerResult<()> {
        // LINK
        todo!()
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
        let data = parse_img(&data);
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

    pub fn merge_alpha(&mut self) -> RunnerResult<()> {
        // MERGEALPHA
        todo!()
    }

    pub fn merge_alpha2(&mut self) -> RunnerResult<()> {
        // MERGEALPHA2
        todo!()
    }

    pub fn monitor_collision(&mut self) -> RunnerResult<()> {
        // MONITORCOLLISION
        todo!()
    }

    pub fn move_by(&mut self, x: isize, y: isize) -> RunnerResult<()> {
        // MOVE
        self.position = (self.position.0 + x, self.position.1 + y);
        Ok(())
    }

    pub fn remove_monitor_collision(&mut self) -> RunnerResult<()> {
        // REMOVEMONITORCOLLISION
        todo!()
    }

    pub fn replace_color(&mut self) -> RunnerResult<()> {
        // REPLACECOLOR
        todo!()
    }

    pub fn reset_flips(&mut self) -> RunnerResult<()> {
        // RESETFLIPS
        todo!()
    }

    pub fn reset_position(&mut self) -> RunnerResult<()> {
        // RESETPOSITION
        todo!()
    }

    pub fn save(&mut self) -> RunnerResult<()> {
        // SAVE
        todo!()
    }

    pub fn set_anchor(&mut self) -> RunnerResult<()> {
        // SETANCHOR
        todo!()
    }

    pub fn set_as_button(&mut self) -> RunnerResult<()> {
        // SETASBUTTON
        todo!()
    }

    pub fn set_clipping(&mut self) -> RunnerResult<()> {
        // SETCLIPPING
        todo!()
    }

    pub fn set_opacity(&mut self) -> RunnerResult<()> {
        // SETOPACITY
        todo!()
    }

    pub fn set_position(&mut self, x: isize, y: isize) -> RunnerResult<()> {
        // SETPOSITION
        self.position = (x, y);
        Ok(())
    }

    pub fn set_priority(&mut self) -> RunnerResult<()> {
        // SETPRIORITY
        todo!()
    }

    pub fn set_reset_position(&mut self) -> RunnerResult<()> {
        // SETRESETPOSITION
        todo!()
    }

    pub fn set_scale_factor(&mut self) -> RunnerResult<()> {
        // SETSCALEFACTOR
        todo!()
    }

    pub fn show(&mut self) -> RunnerResult<()> {
        // SHOW
        self.is_visible = true;
        Ok(())
    }
}
