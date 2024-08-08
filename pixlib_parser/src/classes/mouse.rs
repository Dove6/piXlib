use std::any::Any;

use parsers::{discard_if_empty, parse_i32, parse_program};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct MouseInit {
    // MOUSE
    pub mouse: Option<String>, // MOUSE
    pub raw: Option<i32>,      // RAW

    pub on_click: Option<Arc<ParsedScript>>, // ONCLICK signal
    pub on_dbl_click: Option<Arc<ParsedScript>>, // ONDBLCLICK signal
    pub on_done: Option<Arc<ParsedScript>>,  // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,  // ONINIT signal
    pub on_move: Option<Arc<ParsedScript>>,  // ONMOVE signal
    pub on_release: Option<Arc<ParsedScript>>, // ONRELEASE signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Mouse {
    parent: Arc<CnvObject>,
    initial_properties: MouseInit,
}

impl Mouse {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: MouseInit) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn click() {
        // CLICK
        todo!()
    }

    pub fn disable() {
        // DISABLE
        todo!()
    }

    pub fn disable_signal() {
        // DISABLESIGNAL
        todo!()
    }

    pub fn enable() {
        // ENABLE
        todo!()
    }

    pub fn enable_signal() {
        // ENABLESIGNAL
        todo!()
    }

    pub fn get_last_click_pos_x() {
        // GETLASTCLICKPOSX
        todo!()
    }

    pub fn get_last_click_pos_y() {
        // GETLASTCLICKPOSY
        todo!()
    }

    pub fn get_pos_x() {
        // GETPOSX
        todo!()
    }

    pub fn get_pos_y() {
        // GETPOSY
        todo!()
    }

    pub fn hide() {
        // HIDE
        todo!()
    }

    pub fn is_l_button_down() {
        // ISLBUTTONDOWN
        todo!()
    }

    pub fn is_r_button_down() {
        // ISRBUTTONDOWN
        todo!()
    }

    pub fn lock_active_cursor() {
        // LOCKACTIVECURSOR
        todo!()
    }

    pub fn mouse_release() {
        // MOUSERELEASE
        todo!()
    }

    pub fn move_to() {
        // MOVE
        todo!()
    }

    pub fn set() {
        // SET
        todo!()
    }

    pub fn set_active_rect() {
        // SETACTIVERECT
        todo!()
    }

    pub fn set_clip_rect() {
        // SETCLIPRECT
        todo!()
    }

    pub fn set_position() {
        // SETPOSITION
        todo!()
    }

    pub fn show() {
        // SHOW
        todo!()
    }
}

impl CnvType for Mouse {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "MOUSE"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONCLICK" | "ONDBLCLICK" | "ONDONE" | "ONINIT" | "ONMOVE" | "ONRELEASE" | "ONSIGNAL"
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
        _arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
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
            "ONINIT" => self.initial_properties.on_init.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let mouse = properties.remove("MOUSE").and_then(discard_if_empty);
        let raw = properties
            .remove("RAW")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let on_click = properties
            .remove("ONCLICK")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_dbl_click = properties
            .remove("ONDBLCLICK")
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
        let on_move = properties
            .remove("ONMOVE")
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
        Ok(CnvContent::Mouse(Self::from_initial_properties(
            parent,
            MouseInit {
                mouse,
                raw,
                on_click,
                on_dbl_click,
                on_done,
                on_init,
                on_move,
                on_release,
                on_signal,
            },
        )))
    }
}
