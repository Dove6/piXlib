use std::any::Any;

use parsers::{discard_if_empty, parse_bool, parse_i32, parse_program, parse_rect, Rect};

use super::*;

#[derive(Debug, Clone)]
pub struct ButtonInit {
    // BUTTON
    pub accent: Option<bool>,            // ACCENT
    pub drag: Option<bool>,              // DRAG
    pub draggable: Option<bool>,         // DRAGGABLE
    pub enable: Option<bool>,            // ENABLE
    pub gfx_on_click: Option<ImageName>, // GFXONCLICK
    pub gfx_on_move: Option<ImageName>,  // GFXONMOVE
    pub gfx_standard: Option<ImageName>, // GFXSTANDARD
    pub priority: Option<i32>,           // PRIORITY
    pub rect: Option<Rect>,              // RECT
    pub snd_on_click: Option<SoundName>, // SNDONCLICK
    pub snd_on_move: Option<SoundName>,  // SNDONMOVE
    pub snd_standard: Option<SoundName>, // SNDSTANDARD

    pub on_action: Option<Arc<IgnorableProgram>>, // ONACTION signal
    pub on_clicked: Option<Arc<IgnorableProgram>>, // ONCLICKED signal
    pub on_done: Option<Arc<IgnorableProgram>>,   // ONDONE signal
    pub on_dragging: Option<Arc<IgnorableProgram>>, // ONDRAGGING signal
    pub on_end_dragging: Option<Arc<IgnorableProgram>>, // ONENDDRAGGING signal
    pub on_focus_off: Option<Arc<IgnorableProgram>>, // ONFOCUSOFF signal
    pub on_focus_on: Option<Arc<IgnorableProgram>>, // ONFOCUSON signal
    pub on_init: Option<Arc<IgnorableProgram>>,   // ONINIT signal
    pub on_paused: Option<Arc<IgnorableProgram>>, // ONPAUSED signal
    pub on_released: Option<Arc<IgnorableProgram>>, // ONRELEASED signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
    pub on_start_dragging: Option<Arc<IgnorableProgram>>, // ONSTARTDRAGGING signal
}

#[derive(Debug, Clone)]
pub struct Button {
    parent: Arc<CnvObject>,
    initial_properties: ButtonInit,
}

impl Button {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: ButtonInit) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn accent() {
        todo!()
    }

    pub fn disable() {
        todo!()
    }

    pub fn disable_but_visible() {
        todo!()
    }

    pub fn disable_dragging() {
        todo!()
    }

    pub fn enable() {
        todo!()
    }

    pub fn enable_dragging() {
        todo!()
    }

    pub fn get_on_click() {
        todo!()
    }

    pub fn get_on_move() {
        todo!()
    }

    pub fn get_priority() {
        todo!()
    }

    pub fn get_std() {
        todo!()
    }

    pub fn set_on_click() {
        todo!()
    }

    pub fn set_on_move() {
        todo!()
    }

    pub fn set_priority() {
        todo!()
    }

    pub fn set_rect() {
        todo!()
    }

    pub fn set_std() {
        todo!()
    }

    pub fn syn() {
        todo!()
    }
}

impl CnvType for Button {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BUTTON"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONACTION"
                | "ONCLICKED"
                | "ONDONE"
                | "ONDRAGGING"
                | "ONENDDRAGGING"
                | "ONFOCUSOFF"
                | "ONFOCUSON"
                | "ONINIT"
                | "ONPAUSED"
                | "ONRELEASED"
                | "ONSIGNAL"
                | "ONSTARTDRAGGING"
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
                    v.run(context)
                }
                Ok(None)
            }
            ident => todo!("{:?}.call_method for {:?}", self.get_type_id(), ident),
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
    ) -> Result<Self, TypeParsingError> {
        let accent = properties
            .remove("ACCENT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let drag = properties
            .remove("DRAG")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let draggable = properties
            .remove("DRAGGABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let enable = properties
            .remove("ENABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let gfx_on_click = properties.remove("GFXONCLICK").and_then(discard_if_empty);
        let gfx_on_move = properties.remove("GFXONMOVE").and_then(discard_if_empty);
        let gfx_standard = properties.remove("GFXSTANDARD").and_then(discard_if_empty);
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
        let snd_on_click = properties.remove("SNDONCLICK").and_then(discard_if_empty);
        let snd_on_move = properties.remove("SNDONMOVE").and_then(discard_if_empty);
        let snd_standard = properties.remove("SNDSTANDARD").and_then(discard_if_empty);
        let on_action = properties
            .remove("ONACTION")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_clicked = properties
            .remove("ONCLICKED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_dragging = properties
            .remove("ONDRAGGING")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_end_dragging = properties
            .remove("ONENDDRAGGING")
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
        let on_paused = properties
            .remove("ONPAUSED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_released = properties
            .remove("ONRELEASED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_start_dragging = properties
            .remove("ONSTARTDRAGGING")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Button::from_initial_properties(
            parent,
            ButtonInit {
                accent,
                drag,
                draggable,
                enable,
                gfx_on_click,
                gfx_on_move,
                gfx_standard,
                priority,
                rect,
                snd_on_click,
                snd_on_move,
                snd_standard,
                on_action,
                on_clicked,
                on_done,
                on_dragging,
                on_end_dragging,
                on_focus_off,
                on_focus_on,
                on_init,
                on_paused,
                on_released,
                on_signal,
                on_start_dragging,
            },
        ))
    }
}
