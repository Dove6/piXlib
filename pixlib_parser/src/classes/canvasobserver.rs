use std::any::Any;

use parsers::{discard_if_empty, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct CanvasObserverInit {
    // CANVAS_OBSERVER
    pub on_done: Option<Arc<IgnorableProgram>>, // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>, // ONINIT signal
    pub on_initial_update: Option<Arc<IgnorableProgram>>, // ONINITIALUPDATE signal
    pub on_initial_updated: Option<Arc<IgnorableProgram>>, // ONINITIALUPDATED signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
    pub on_update: Option<Arc<IgnorableProgram>>, // ONUPDATE signal
    pub on_updated: Option<Arc<IgnorableProgram>>, // ONUPDATED signal
    pub on_window_focus_off: Option<Arc<IgnorableProgram>>, // ONWINDOWFOCUSOFF signal
    pub on_window_focus_on: Option<Arc<IgnorableProgram>>, // ONWINDOWFOCUSON signal
}

#[derive(Debug, Clone)]
pub struct CanvasObserver {
    parent: Arc<CnvObject>,
    initial_properties: CanvasObserverInit,
}

impl CanvasObserver {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: CanvasObserverInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn add() {
        // ADD
        todo!()
    }

    pub fn enable_notify() {
        // ENABLENOTIFY
        todo!()
    }

    pub fn get_bpp() {
        // GETBPP
        todo!()
    }

    pub fn get_graphics_at() {
        // GETGRAPHICSAT
        todo!()
    }

    pub fn get_graphics_at2() {
        // GETGRAPHICSAT2
        todo!()
    }

    pub fn move_bkg() {
        // MOVEBKG
        todo!()
    }

    pub fn paste() {
        // PASTE
        todo!()
    }

    pub fn redraw() {
        // REDRAW
        todo!()
    }

    pub fn refresh() {
        // REFRESH
        todo!()
    }

    pub fn remove() {
        // REMOVE
        todo!()
    }

    pub fn save() {
        // SAVE
        todo!()
    }

    pub fn set_background() {
        // SETBACKGROUND
        todo!()
    }

    pub fn set_bkg_pos() {
        // SETBKGPOS
        todo!()
    }
}

impl CnvType for CanvasObserver {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "CANVASOBSERVER"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONDONE"
                | "ONINIT"
                | "ONINITIALUPDATE"
                | "ONINITIALUPDATED"
                | "ONSIGNAL"
                | "ONUPDATE"
                | "ONUPDATED"
                | "ONWINDOWFOCUSOFF"
                | "ONWINDOWFOCUSON"
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
        let on_initial_update = properties
            .remove("ONINITIALUPDATE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_initial_updated = properties
            .remove("ONINITIALUPDATED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_update = properties
            .remove("ONUPDATE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_updated = properties
            .remove("ONUPDATED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_window_focus_off = properties
            .remove("ONWINDOWFOCUSOFF")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_window_focus_on = properties
            .remove("ONWINDOWFOCUSON")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            CanvasObserverInit {
                on_done,
                on_init,
                on_initial_update,
                on_initial_updated,
                on_signal,
                on_update,
                on_updated,
                on_window_focus_off,
                on_window_focus_on,
            },
        ))
    }
}
