use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct TextInit {
    // TEXT
    pub font: Option<FontName>,                // FONT
    pub horizontal_justify: Option<bool>,      // HJUSTIFY
    pub hypertext: Option<bool>,               // HYPERTEXT
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    pub priority: Option<i32>,                 // PRIORITY
    pub rect: Option<Rect>,                    // RECT
    pub text: Option<String>,                  // TEXT
    pub to_canvas: Option<bool>,               // TOCANVAS
    pub visible: Option<bool>,                 // VISIBLE
    pub vertical_justify: Option<bool>,        // VJUSTIFY

    pub on_collision: Option<Arc<IgnorableProgram>>, // ONCOLLISION signal
    pub on_collision_finished: Option<Arc<IgnorableProgram>>, // ONCOLLISIONFINISHED signal
    pub on_done: Option<Arc<IgnorableProgram>>,      // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>,      // ONINIT signal
    pub on_signal: Option<Arc<IgnorableProgram>>,    // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Text {
    parent: Arc<RwLock<CnvObject>>,
    initial_properties: TextInit,
}

impl Text {
    pub fn from_initial_properties(
        parent: Arc<RwLock<CnvObject>>,
        initial_properties: TextInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn clear_clipping() {
        // CLEARCLIPPING
        todo!()
    }

    pub fn draw_onto() {
        // DRAWONTO
        todo!()
    }

    pub fn get_height() {
        // GETHEIGHT
        todo!()
    }

    pub fn get_num_words() {
        // GETNUMWORDS
        todo!()
    }

    pub fn get_position_x() {
        // GETPOSITIONX
        todo!()
    }

    pub fn get_position_y() {
        // GETPOSITIONY
        todo!()
    }

    pub fn get_width() {
        // GETWIDTH
        todo!()
    }

    pub fn get_word_at() {
        // GETWORDAT
        todo!()
    }

    pub fn get_word_at_xy() {
        // GETWORDATXY
        todo!()
    }

    pub fn get_word_pos_x() {
        // GETWORDPOSX
        todo!()
    }

    pub fn get_word_pos_y() {
        // GETWORDPOSY
        todo!()
    }

    pub fn get_word_width() {
        // GETWORDWIDTH
        todo!()
    }

    pub fn hide() {
        // HIDE
        todo!()
    }

    pub fn invalidate() {
        // INVALIDATE
        todo!()
    }

    pub fn is_near() {
        // ISNEAR
        todo!()
    }

    pub fn load() {
        // LOAD
        todo!()
    }

    pub fn move_to() {
        // MOVE
        todo!()
    }

    pub fn search() {
        // SEARCH
        todo!()
    }

    pub fn set_clipping() {
        // SETCLIPPING
        todo!()
    }

    pub fn set_color() {
        // SETCOLOR
        todo!()
    }

    pub fn set_font() {
        // SETFONT
        todo!()
    }

    pub fn set_justify() {
        // SETJUSTIFY
        todo!()
    }

    pub fn set_opacity() {
        // SETOPACITY
        todo!()
    }

    pub fn set_position() {
        // SETPOSITION
        todo!()
    }

    pub fn set_priority() {
        // SETPRIORITY
        todo!()
    }

    pub fn set_rect() {
        // SETRECT
        todo!()
    }

    pub fn set_text() {
        // SETTEXT
        todo!()
    }

    pub fn set_text_double() {
        // SETTEXTDOUBLE
        todo!()
    }

    pub fn set_word_color() {
        // SETWORDCOLOR
        todo!()
    }

    pub fn show() {
        // SHOW
        todo!()
    }
}

impl CnvType for Text {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "TEXT"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        todo!()
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<RwLock<CnvObject>>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let font = properties.remove("FONT").and_then(discard_if_empty);
        let horizontal_justify = properties
            .remove("HJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let hypertext = properties
            .remove("HYPERTEXT")
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
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let rect = properties
            .remove("RECT")
            .and_then(discard_if_empty)
            .map(parse_rect)
            .transpose()?;
        let text = properties.remove("TEXT").and_then(discard_if_empty);
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
        let vertical_justify = properties
            .remove("VJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
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
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            TextInit {
                font,
                horizontal_justify,
                hypertext,
                monitor_collision,
                monitor_collision_alpha,
                priority,
                rect,
                text,
                to_canvas,
                visible,
                vertical_justify,
                on_collision,
                on_collision_finished,
                on_done,
                on_init,
                on_signal,
            },
        ))
    }
}
