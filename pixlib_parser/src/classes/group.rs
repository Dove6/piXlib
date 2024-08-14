use std::{any::Any, cell::RefCell};

use content::EventHandler;
use initable::Initable;
use parsers::{discard_if_empty, parse_event_handler};

use crate::{ast::ParsedScript, common::DroppableRefMut, runner::InternalEvent};

use super::*;

#[derive(Debug, Clone)]
pub struct GroupInit {
    // GROUP
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct GroupState {
    // deduced from methods
    pub objects: Vec<Arc<CnvObject>>,
    pub cursor_index: usize,
}

#[derive(Debug, Clone)]
pub struct GroupEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

impl EventHandler for GroupEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    parent: Arc<CnvObject>,

    state: RefCell<GroupState>,
    event_handlers: GroupEventHandlers,
}

impl Group {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: GroupInit) -> Self {
        Self {
            parent,
            state: RefCell::new(GroupState {
                ..Default::default()
            }),
            event_handlers: GroupEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
            },
        }
    }
}

impl CnvType for Group {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "GROUP"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        eprintln!(
            "Skipping method call {:?} for GROUP {:?}",
            name, self.parent.name
        );
        if self.get_type_id() == "GROUP" {
            return Ok(None);
        } // TODO: fill in
        match name {
            CallableIdentifier::Method("ADD") => self.state.borrow_mut().add().map(|_| None),
            CallableIdentifier::Method("ADDCLONES") => {
                self.state.borrow_mut().add_clones().map(|_| None)
            }
            CallableIdentifier::Method("CLONE") => {
                self.state.borrow_mut().clone_object().map(|_| None)
            }
            CallableIdentifier::Method("CONTAINS") => {
                self.state.borrow_mut().contains().map(|_| None)
            }
            CallableIdentifier::Method("GETCLONEINDEX") => self
                .state
                .borrow()
                .get_clone_index()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMARKERPOS") => self
                .state
                .borrow()
                .get_marker_pos()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETNAME") => self
                .state
                .borrow()
                .get_name()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETNAMEATMARKER") => self
                .state
                .borrow()
                .get_name_at_marker()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETSIZE") => self
                .state
                .borrow()
                .get_size()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("NEXT") => self.state.borrow_mut().next().map(|_| None),
            CallableIdentifier::Method("PREV") => self.state.borrow_mut().prev().map(|_| None),
            CallableIdentifier::Method("REMOVE") => self.state.borrow_mut().remove().map(|_| None),
            CallableIdentifier::Method("REMOVEALL") => {
                self.state.borrow_mut().remove_all().map(|_| None)
            }
            CallableIdentifier::Method("RESETMARKER") => {
                self.state.borrow_mut().reset_marker().map(|_| None)
            }
            CallableIdentifier::Method("SETMARKERPOS") => {
                self.state.borrow_mut().set_marker_pos().map(|_| None)
            }
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
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::Group(Self::from_initial_properties(
            parent,
            GroupInit {
                on_done,
                on_init,
                on_signal,
            },
        )))
    }
}

impl Initable for Group {
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

impl GroupState {
    pub fn add(&mut self) -> RunnerResult<()> {
        // ADD
        todo!()
    }

    pub fn add_clones(&mut self) -> RunnerResult<()> {
        // ADDCLONES
        todo!()
    }

    pub fn clone_object(&mut self) -> RunnerResult<()> {
        // CLONE
        todo!()
    }

    pub fn contains(&mut self) -> RunnerResult<()> {
        // CONTAINS
        todo!()
    }

    pub fn get_clone_index(&self) -> RunnerResult<usize> {
        // GETCLONEINDEX
        todo!()
    }

    pub fn get_marker_pos(&self) -> RunnerResult<usize> {
        // GETMARKERPOS
        todo!()
    }

    pub fn get_name(&self) -> RunnerResult<String> {
        // GETNAME
        todo!()
    }

    pub fn get_name_at_marker(&self) -> RunnerResult<String> {
        // GETNAMEATMARKER
        todo!()
    }

    pub fn get_size(&self) -> RunnerResult<usize> {
        // GETSIZE
        todo!()
    }

    pub fn next(&mut self) -> RunnerResult<()> {
        // NEXT
        todo!()
    }

    pub fn prev(&mut self) -> RunnerResult<()> {
        // PREV
        todo!()
    }

    pub fn remove(&mut self) -> RunnerResult<()> {
        // REMOVE
        todo!()
    }

    pub fn remove_all(&mut self) -> RunnerResult<()> {
        // REMOVEALL
        todo!()
    }

    pub fn reset_marker(&mut self) -> RunnerResult<()> {
        // RESETMARKER
        todo!()
    }

    pub fn set_marker_pos(&mut self) -> RunnerResult<()> {
        // SETMARKERPOS
        todo!()
    }
}
