use std::{any::Any, cell::RefCell};

use content::EventHandler;
use initable::Initable;
use parsers::{discard_if_empty, parse_event_handler};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

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

impl EventHandler for CanvasObserverEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONINITIALUPDATE" => self.on_initial_update.as_ref(),
            "ONINITIALUPDATED" => self.on_initial_updated.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            "ONUPDATE" => self.on_update.as_ref(),
            "ONUPDATED" => self.on_updated.as_ref(),
            "ONWINDOWFOCUSOFF" => self.on_window_focus_off.as_ref(),
            "ONWINDOWFOCUSON" => self.on_window_focus_on.as_ref(),
            _ => None,
        }
    }
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
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
            CallableIdentifier::Method("GETGRAPHICSAT") => self
                .state
                .borrow()
                .get_graphics_at()
                .map(|v| v.and_then(|s| Some(CnvValue::String(s)))),
            CallableIdentifier::Method("GETGRAPHICSAT2") => self
                .state
                .borrow()
                .get_graphics_at2()
                .map(|v| v.and_then(|s| Some(CnvValue::String(s)))),
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
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self.event_handlers.get(
                    event_name,
                    arguments.get(0).map(|v| v.to_string()).as_deref(),
                ) {
                    code.run(context)?;
                }
                Ok(None)
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_initial_update = properties
            .remove("ONINITIALUPDATE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_initial_updated = properties
            .remove("ONINITIALUPDATED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_update = properties
            .remove("ONUPDATE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_updated = properties
            .remove("ONUPDATED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_window_focus_off = properties
            .remove("ONWINDOWFOCUSOFF")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_window_focus_on = properties
            .remove("ONWINDOWFOCUSON")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
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

impl Initable for CanvasObserver {
    fn initialize(&mut self, context: RunnerContext) -> RunnerResult<()> {
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
