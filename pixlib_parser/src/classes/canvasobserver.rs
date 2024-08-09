use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_program};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct CanvasObserverProperties {
    // CANVAS_OBSERVER
    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_initial_update: Option<Arc<ParsedScript>>, // ONINITIALUPDATE signal
    pub on_initial_updated: Option<Arc<ParsedScript>>, // ONINITIALUPDATED signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_update: Option<Arc<ParsedScript>>, // ONUPDATE signal
    pub on_updated: Option<Arc<ParsedScript>>, // ONUPDATED signal
    pub on_window_focus_off: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSOFF signal
    pub on_window_focus_on: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSON signal
}

#[derive(Debug, Clone, Default)]
struct CanvasObserverState {
    pub initialized: bool,

    // deduced from methods
    background_data: ImageFileData,
    background_position: (isize, isize),
    graphics: Vec<Arc<CnvObject>>,
}

#[derive(Debug, Clone)]
pub struct CanvasObserverEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_initial_update: Option<Arc<ParsedScript>>, // ONINITIALUPDATE signal
    pub on_initial_updated: Option<Arc<ParsedScript>>, // ONINITIALUPDATED signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_update: Option<Arc<ParsedScript>>, // ONUPDATE signal
    pub on_updated: Option<Arc<ParsedScript>>, // ONUPDATED signal
    pub on_window_focus_off: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSOFF signal
    pub on_window_focus_on: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSON signal
}

#[derive(Debug, Clone)]
pub struct CanvasObserver {
    parent: Arc<CnvObject>,

    state: RefCell<CanvasObserverState>,
    event_handlers: CanvasObserverEventHandlers,
}

impl CanvasObserver {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        props: CanvasObserverProperties,
    ) -> Self {
        Self {
            parent,
            state: RefCell::new(CanvasObserverState {
                ..Default::default()
            }),
            event_handlers: CanvasObserverEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_initial_update: props.on_initial_update,
                on_initial_updated: props.on_initial_updated,
                on_signal: props.on_signal,
                on_update: props.on_update,
                on_updated: props.on_updated,
                on_window_focus_off: props.on_window_focus_off,
                on_window_focus_on: props.on_window_focus_on,
            },
        }
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
            CallableIdentifier::Method("ADD") => self.state.borrow_mut().add().map(|_| None),
            CallableIdentifier::Method("ENABLENOTIFY") => {
                self.state.borrow_mut().enable_notify().map(|_| None)
            }
            CallableIdentifier::Method("GETBPP") => self
                .state
                .borrow()
                .get_bpp()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETGRAPHICSAT") => {
                self.state.borrow().get_graphics_at().map(|v| {
                    v.and_then(|s| {
                        context
                            .runner
                            .get_object(&s)
                            .map(|o| CnvValue::Reference(o))
                    })
                })
            }
            CallableIdentifier::Method("GETGRAPHICSAT2") => {
                self.state.borrow().get_graphics_at2().map(|v| {
                    v.and_then(|s| {
                        context
                            .runner
                            .get_object(&s)
                            .map(|o| CnvValue::Reference(o))
                    })
                })
            }
            CallableIdentifier::Method("MOVEBKG") => {
                self.state.borrow_mut().move_bkg().map(|_| None)
            }
            CallableIdentifier::Method("PASTE") => self.state.borrow_mut().paste().map(|_| None),
            CallableIdentifier::Method("REDRAW") => self.state.borrow_mut().redraw().map(|_| None),
            CallableIdentifier::Method("REFRESH") => {
                self.state.borrow_mut().refresh().map(|_| None)
            }
            CallableIdentifier::Method("REMOVE") => self.state.borrow_mut().remove().map(|_| None),
            CallableIdentifier::Method("SAVE") => self.state.borrow_mut().save().map(|_| None),
            CallableIdentifier::Method("SETBACKGROUND") => {
                self.state.borrow_mut().set_background().map(|_| None)
            }
            CallableIdentifier::Method("SETBKGPOS") => {
                self.state.borrow_mut().set_bkg_pos().map(|_| None)
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.event_handlers.on_done.as_ref() {
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
            CallableIdentifier::Event("ONINITIALUPDATE") => {
                if let Some(v) = self.event_handlers.on_initial_update.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONINITIALUPDATED") => {
                if let Some(v) = self.event_handlers.on_initial_updated.as_ref() {
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
            CallableIdentifier::Event("ONUPDATE") => {
                if let Some(v) = self.event_handlers.on_update.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONUPDATED") => {
                if let Some(v) = self.event_handlers.on_updated.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONWINDOWFOCUSOFF") => {
                if let Some(v) = self.event_handlers.on_window_focus_off.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONWINDOWFOCUSON") => {
                if let Some(v) = self.event_handlers.on_window_focus_on.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
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
        Ok(CnvContent::CanvasObserver(Self::from_initial_properties(
            parent,
            CanvasObserverProperties {
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
        )))
    }
}

impl CanvasObserverState {
    pub fn add(&mut self) -> RunnerResult<()> {
        // ADD
        todo!()
    }

    pub fn enable_notify(&mut self) -> RunnerResult<()> {
        // ENABLENOTIFY
        todo!()
    }

    pub fn get_bpp(&self) -> RunnerResult<usize> {
        // GETBPP
        Ok(32)
    }

    pub fn get_graphics_at(&self) -> RunnerResult<Option<String>> {
        // GETGRAPHICSAT
        todo!()
    }

    pub fn get_graphics_at2(&self) -> RunnerResult<Option<String>> {
        // GETGRAPHICSAT2
        todo!()
    }

    pub fn move_bkg(&mut self) -> RunnerResult<()> {
        // MOVEBKG
        todo!()
    }

    pub fn paste(&mut self) -> RunnerResult<()> {
        // PASTE
        todo!()
    }

    pub fn redraw(&mut self) -> RunnerResult<()> {
        // REDRAW
        todo!()
    }

    pub fn refresh(&mut self) -> RunnerResult<()> {
        // REFRESH
        todo!()
    }

    pub fn remove(&mut self) -> RunnerResult<()> {
        // REMOVE
        todo!()
    }

    pub fn save(&mut self) -> RunnerResult<()> {
        // SAVE
        todo!()
    }

    pub fn set_background(&mut self) -> RunnerResult<()> {
        // SETBACKGROUND
        todo!()
    }

    pub fn set_bkg_pos(&mut self) -> RunnerResult<()> {
        // SETBKGPOS
        todo!()
    }
}
