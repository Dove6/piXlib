use std::any::Any;

use parsers::{discard_if_empty, parse_bool, parse_i32, parse_program};
use pixlib_formats::file_formats::img::parse_img;

use crate::runner::{DummyFileSystem, RunnerError};

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

    pub on_click: Option<Arc<IgnorableProgram>>, // ONCLICK signal
    pub on_collision: Option<Arc<IgnorableProgram>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<IgnorableProgram>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<IgnorableProgram>>,  // ONDONE signal
    pub on_focus_off: Option<Arc<IgnorableProgram>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<IgnorableProgram>>, // ONFOCUSON signal
    pub on_init: Option<Arc<IgnorableProgram>>,  // ONINIT signal
    pub on_release: Option<Arc<IgnorableProgram>>, // ONRELEASE signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageDefinition {
    pub size_px: (u32, u32),
    pub offset_px: (i32, i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageData {
    pub data: Vec<u8>, // RGBA8888
}

#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub filename: Option<String>,
    pub image: (ImageDefinition, ImageData),
}

#[derive(Debug, Clone)]
pub struct Image {
    // IMAGE
    parent: Arc<CnvObject>,
    initial_properties: ImageInit,

    is_flipped_horizontally: bool,
    is_flipped_vertically: bool,
    is_visible: bool,
    loaded_data: Option<LoadedImage>,
    // anchor: ,
    is_button: bool,
    opacity: usize,
    priority: i32,
    position: (i32, i32),
}

impl Image {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: ImageInit) -> Self {
        let preload = initial_properties.preload.is_some_and(|v| v);
        let filename = initial_properties.filename.clone().unwrap_or_default();
        let is_visible = initial_properties.visible.unwrap_or(true);
        let is_button = initial_properties.as_button.unwrap_or(false);
        let priority = initial_properties.priority.unwrap_or(0);
        let mut image = Self {
            parent: Arc::clone(&parent),
            initial_properties,
            is_flipped_horizontally: false,
            is_flipped_vertically: false,
            is_visible,
            loaded_data: None,
            is_button,
            opacity: 1,
            priority,
            position: (0, 0),
        };
        if preload {
            let script = parent.parent.as_ref();
            let filesystem = Arc::clone(&script.runner.filesystem);
            let path = Arc::clone(&script.path);
            image.load(
                &*filesystem.as_ref().borrow(),
                path.with_file_name(&filename).to_str().unwrap(),
            );
        }
        image
    }

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

    pub fn get_priority(&self) {
        todo!()
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

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn link(&self) {
        todo!()
    }

    pub fn load(&mut self, filesystem: &dyn FileSystem, filename: &str) -> RunnerResult<()> {
        let data = filesystem
            .read_file(filename)
            .map_err(|e| RunnerError::IoError { source: e })?;
        let data = parse_img(&data);
        self.loaded_data = Some(LoadedImage {
            filename: Some(filename.to_owned()),
            image: (
                ImageDefinition {
                    size_px: (data.header.width_px, data.header.height_px),
                    offset_px: (data.header.x_position_px, data.header.y_position_px),
                },
                ImageData {
                    data: data
                        .image_data
                        .to_rgba8888(data.header.color_format, data.header.compression_type),
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

    pub fn get_image_to_show(&self) -> RunnerResult<Option<(&ImageDefinition, &ImageData)>> {
        if !self.is_visible {
            return Ok(None);
        }
        let Some(loaded_data) = &self.loaded_data else {
            return Ok(None);
        };
        let image = &loaded_data.image;
        Ok(Some((&image.0, &image.1)))
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
        &mut self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("CLEARCLIPPING") => {
                self.clear_clipping();
                Ok(None)
            }
            CallableIdentifier::Method("DRAWONTO") => {
                self.draw_onto();
                Ok(None)
            }
            CallableIdentifier::Method("FLIPH") => {
                self.flip_h();
                Ok(None)
            }
            CallableIdentifier::Method("FLIPV") => {
                self.flip_v();
                Ok(None)
            }
            CallableIdentifier::Method("GETALPHA") => {
                self.get_alpha();
                Ok(None)
            }
            CallableIdentifier::Method("GETCENTERX") => {
                self.get_center_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETCENTERY") => {
                self.get_center_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETCOLORAT") => {
                self.get_color_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETCOLORBAT") => {
                self.get_color_b_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETCOLORGAT") => {
                self.get_color_g_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETCOLORRAT") => {
                self.get_color_r_at();
                Ok(None)
            }
            CallableIdentifier::Method("GETHEIGHT") => {
                self.get_height();
                Ok(None)
            }
            CallableIdentifier::Method("GETOPACITY") => {
                self.get_opacity();
                Ok(None)
            }
            CallableIdentifier::Method("GETPIXEL") => {
                self.get_pixel();
                Ok(None)
            }
            CallableIdentifier::Method("GETPOSITIONX") => {
                self.get_position_x();
                Ok(None)
            }
            CallableIdentifier::Method("GETPOSITIONY") => {
                self.get_position_y();
                Ok(None)
            }
            CallableIdentifier::Method("GETPRIORITY") => {
                self.get_priority();
                Ok(None)
            }
            CallableIdentifier::Method("GETSLIDECOMPS") => {
                self.get_slide_comps();
                Ok(None)
            }
            CallableIdentifier::Method("GETWIDTH") => {
                self.get_width();
                Ok(None)
            }
            CallableIdentifier::Method("HIDE") => {
                self.hide();
                Ok(None)
            }
            CallableIdentifier::Method("INVALIDATE") => {
                self.invalidate();
                Ok(None)
            }
            CallableIdentifier::Method("ISAT") => {
                self.is_at();
                Ok(None)
            }
            CallableIdentifier::Method("ISINSIDE") => {
                self.is_inside();
                Ok(None)
            }
            CallableIdentifier::Method("ISNEAR") => {
                self.is_near();
                Ok(None)
            }
            CallableIdentifier::Method("ISVISIBLE") => {
                self.is_visible();
                Ok(None)
            }
            CallableIdentifier::Method("LINK") => {
                self.link();
                Ok(None)
            }
            CallableIdentifier::Method("LOAD") => {
                self.load(&DummyFileSystem, &arguments[0].to_string());
                Ok(None)
            }
            CallableIdentifier::Method("MERGEALPHA") => {
                self.merge_alpha();
                Ok(None)
            }
            CallableIdentifier::Method("MERGEALPHA2") => {
                self.merge_alpha2();
                Ok(None)
            }
            CallableIdentifier::Method("MONITORCOLLISION") => {
                self.monitor_collision();
                Ok(None)
            }
            CallableIdentifier::Method("MOVE") => {
                self.move_to();
                Ok(None)
            }
            CallableIdentifier::Method("REMOVEMONITORCOLLISION") => {
                self.remove_monitor_collision();
                Ok(None)
            }
            CallableIdentifier::Method("REPLACECOLOR") => {
                self.replace_color();
                Ok(None)
            }
            CallableIdentifier::Method("RESETFLIPS") => {
                self.reset_flips();
                Ok(None)
            }
            CallableIdentifier::Method("RESETPOSITION") => {
                self.reset_position();
                Ok(None)
            }
            CallableIdentifier::Method("SAVE") => {
                self.save();
                Ok(None)
            }
            CallableIdentifier::Method("SETANCHOR") => {
                self.set_anchor();
                Ok(None)
            }
            CallableIdentifier::Method("SETASBUTTON") => {
                self.set_as_button();
                Ok(None)
            }
            CallableIdentifier::Method("SETCLIPPING") => {
                self.set_clipping();
                Ok(None)
            }
            CallableIdentifier::Method("SETOPACITY") => {
                self.set_opacity();
                Ok(None)
            }
            CallableIdentifier::Method("SETPOSITION") => {
                self.set_position();
                Ok(None)
            }
            CallableIdentifier::Method("SETPRIORITY") => {
                self.set_priority();
                Ok(None)
            }
            CallableIdentifier::Method("SETRESETPOSITION") => {
                self.set_reset_position();
                Ok(None)
            }
            CallableIdentifier::Method("SETSCALEFACTOR") => {
                self.set_scale_factor();
                Ok(None)
            }
            CallableIdentifier::Method("SHOW") => {
                self.show();
                Ok(None)
            }
            CallableIdentifier::Event("ONCLICK") => {
                if let Some(v) = self.initial_properties.on_click.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONCOLLISION") => {
                if let Some(v) = self.initial_properties.on_collision.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONCOLLISIONFINISHED") => {
                if let Some(v) = self.initial_properties.on_collision_finished.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.initial_properties.on_done.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONFOCUSOFF") => {
                if let Some(v) = self.initial_properties.on_focus_off.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONFOCUSON") => {
                if let Some(v) = self.initial_properties.on_focus_on.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONRELEASE") => {
                if let Some(v) = self.initial_properties.on_release.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONSIGNAL") => {
                if let Some(v) = self.initial_properties.on_signal.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "FILENAME" => self.initial_properties.filename.clone().map(|v| v.into()),
            "PRIORITY" => Some(self.priority.into()),
            "ONINIT" => self.initial_properties.on_init.clone().map(|v| v.into()),
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
        Ok(Image::from_initial_properties(
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
        ))
    }
}
