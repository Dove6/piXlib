use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_program};

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct ArrayProperties {
    // ARRAY
    pub send_on_change: Option<bool>, // SENDONCHANGE

    pub on_change: Option<Arc<ParsedScript>>, // ONCHANGE signal
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct ArrayState {
    pub initialized: bool,

    // initialized from properties
    pub should_send_on_change_event: bool,

    // deduced from methods
    pub cursor_index: usize,

    pub values: Vec<CnvValue>,
}

#[derive(Debug, Clone)]
pub struct ArrayEventHandlers {
    pub on_change: Option<Arc<ParsedScript>>, // ONCHANGE signal
    pub on_done: Option<Arc<ParsedScript>>,   // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,   // ONINIT signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Array {
    parent: Arc<CnvObject>,

    state: RefCell<ArrayState>,
    event_handlers: ArrayEventHandlers,
}

impl Array {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: ArrayProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(ArrayState {
                should_send_on_change_event: props.send_on_change.unwrap_or_default(),
                ..Default::default()
            }),
            event_handlers: ArrayEventHandlers {
                on_change: props.on_change,
                on_done: props.on_done,
                on_init: props.on_init,
                on_signal: props.on_signal,
            },
        }
    }
}

impl CnvType for Array {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "ARRAY"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(name, "ONCHANGE" | "ONDONE" | "ONINIT" | "ONSIGNAL")
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
        match name {
            CallableIdentifier::Method("ADD") => self.state.borrow_mut().add().map(|_| None),
            CallableIdentifier::Method("ADDAT") => self.state.borrow_mut().add_at().map(|_| None),
            CallableIdentifier::Method("ADDCLONES") => {
                self.state.borrow_mut().add_clones().map(|_| None)
            }
            CallableIdentifier::Method("CHANGEAT") => {
                self.state.borrow_mut().change_at().map(|_| None)
            }
            CallableIdentifier::Method("CLAMPAT") => {
                self.state.borrow_mut().clamp_at().map(|_| None)
            }
            CallableIdentifier::Method("COMPARE") => self.state.borrow().compare().map(|_| None),
            CallableIdentifier::Method("CONTAINS") => self.state.borrow().contains().map(|_| None),
            CallableIdentifier::Method("COPYTO") => self.state.borrow_mut().copy_to().map(|_| None),
            CallableIdentifier::Method("DIR") => self.state.borrow_mut().dir().map(|_| None),
            CallableIdentifier::Method("DIV") => self.state.borrow_mut().div().map(|_| None),
            CallableIdentifier::Method("DIVA") => self.state.borrow_mut().div_a().map(|_| None),
            CallableIdentifier::Method("DIVAT") => self.state.borrow_mut().div_at().map(|_| None),
            CallableIdentifier::Method("FILL") => self.state.borrow_mut().fill().map(|_| None),
            CallableIdentifier::Method("FIND") => self.state.borrow().find().map(|_| None),
            CallableIdentifier::Method("FINDALL") => self.state.borrow().find_all().map(|_| None),
            CallableIdentifier::Method("GET") => self.state.borrow().get().map(|_| None),
            CallableIdentifier::Method("GETMARKERPOS") => {
                self.state.borrow().get_marker_pos().map(|_| None)
            }
            CallableIdentifier::Method("GETSIZE") => self.state.borrow().get_size().map(|_| None),
            CallableIdentifier::Method("GETSUMVALUE") => {
                self.state.borrow().get_sum_value().map(|_| None)
            }
            CallableIdentifier::Method("INSERTAT") => {
                self.state.borrow_mut().insert_at().map(|_| None)
            }
            CallableIdentifier::Method("LOAD") => self.state.borrow_mut().load().map(|_| None),
            CallableIdentifier::Method("LOADINI") => {
                self.state.borrow_mut().load_ini().map(|_| None)
            }
            CallableIdentifier::Method("MAX") => self.state.borrow_mut().max().map(|_| None),
            CallableIdentifier::Method("MAXD") => self.state.borrow_mut().max_d().map(|_| None),
            CallableIdentifier::Method("MIN") => self.state.borrow_mut().min().map(|_| None),
            CallableIdentifier::Method("MIND") => self.state.borrow_mut().min_d().map(|_| None),
            CallableIdentifier::Method("MODAT") => self.state.borrow_mut().mod_at().map(|_| None),
            CallableIdentifier::Method("MUL") => self.state.borrow_mut().mul().map(|_| None),
            CallableIdentifier::Method("MULA") => self.state.borrow_mut().mul_a().map(|_| None),
            CallableIdentifier::Method("MULAT") => self.state.borrow_mut().mul_at().map(|_| None),
            CallableIdentifier::Method("NEXT") => self.state.borrow_mut().next().map(|_| None),
            CallableIdentifier::Method("PREV") => self.state.borrow_mut().prev().map(|_| None),
            CallableIdentifier::Method("RANDOMFILL") => {
                self.state.borrow_mut().random_fill().map(|_| None)
            }
            CallableIdentifier::Method("REMOVE") => self.state.borrow_mut().remove().map(|_| None),
            CallableIdentifier::Method("REMOVEALL") => {
                self.state.borrow_mut().remove_all().map(|_| None)
            }
            CallableIdentifier::Method("REMOVEAT") => {
                self.state.borrow_mut().remove_at().map(|_| None)
            }
            CallableIdentifier::Method("RESETMARKER") => {
                self.state.borrow_mut().reset_marker().map(|_| None)
            }
            CallableIdentifier::Method("REVERSEFIND") => {
                self.state.borrow().reverse_find().map(|_| None)
            }
            CallableIdentifier::Method("ROTATELEFT") => {
                self.state.borrow_mut().rotate_left().map(|_| None)
            }
            CallableIdentifier::Method("ROTATERIGHT") => {
                self.state.borrow_mut().rotate_right().map(|_| None)
            }
            CallableIdentifier::Method("SAVE") => self.state.borrow_mut().save().map(|_| None),
            CallableIdentifier::Method("SAVEINI") => {
                self.state.borrow_mut().save_ini().map(|_| None)
            }
            CallableIdentifier::Method("SENDONCHANGE") => {
                self.state.borrow_mut().send_on_change().map(|_| None)
            }
            CallableIdentifier::Method("SETMARKERPOS") => {
                self.state.borrow_mut().set_marker_pos().map(|_| None)
            }
            CallableIdentifier::Method("SHIFTLEFT") => {
                self.state.borrow_mut().shift_left().map(|_| None)
            }
            CallableIdentifier::Method("SHIFTRIGHT") => {
                self.state.borrow_mut().shift_right().map(|_| None)
            }
            CallableIdentifier::Method("SORT") => self.state.borrow_mut().sort().map(|_| None),
            CallableIdentifier::Method("SORTMANY") => {
                self.state.borrow_mut().sort_many().map(|_| None)
            }
            CallableIdentifier::Method("SUB") => self.state.borrow_mut().sub().map(|_| None),
            CallableIdentifier::Method("SUBA") => self.state.borrow_mut().sub_a().map(|_| None),
            CallableIdentifier::Method("SUBAT") => self.state.borrow_mut().sub_at().map(|_| None),
            CallableIdentifier::Method("SUM") => self.state.borrow_mut().sum().map(|_| None),
            CallableIdentifier::Method("SUMA") => self.state.borrow_mut().sum_a().map(|_| None),
            CallableIdentifier::Method("SWAP") => self.state.borrow_mut().swap().map(|_| None),
            CallableIdentifier::Event("ONCHANGE") => {
                if let Some(v) = self.event_handlers.on_change.as_ref() {
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
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.event_handlers.on_init.as_ref() {
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

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let send_on_change = properties
            .remove("SENDONCHANGE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        // TODO: too many properties
        let on_change = properties
            .remove("ONCHANGE")
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
        Ok(CnvContent::Array(Self::from_initial_properties(
            parent,
            ArrayProperties {
                send_on_change,
                on_change,
                on_done,
                on_init,
                on_signal,
            },
        )))
    }
}

impl ArrayState {
    pub fn add(&mut self) -> RunnerResult<()> {
        // ADD
        todo!()
    }

    pub fn add_at(&mut self) -> RunnerResult<()> {
        // ADDAT
        todo!()
    }

    pub fn add_clones(&mut self) -> RunnerResult<()> {
        // ADDCLONES
        todo!()
    }

    pub fn change_at(&mut self) -> RunnerResult<()> {
        // CHANGEAT
        todo!()
    }

    pub fn clamp_at(&mut self) -> RunnerResult<()> {
        // CLAMPAT
        todo!()
    }

    pub fn compare(&self) -> RunnerResult<()> {
        // COMPARE
        todo!()
    }

    pub fn contains(&self) -> RunnerResult<bool> {
        // CONTAINS
        todo!()
    }

    pub fn copy_to(&mut self) -> RunnerResult<()> {
        // COPYTO
        todo!()
    }

    pub fn dir(&mut self) -> RunnerResult<()> {
        // DIR
        todo!()
    }

    pub fn div(&mut self) -> RunnerResult<()> {
        // DIV
        todo!()
    }

    pub fn div_a(&mut self) -> RunnerResult<()> {
        // DIVA
        todo!()
    }

    pub fn div_at(&mut self) -> RunnerResult<()> {
        // DIVAT
        todo!()
    }

    pub fn fill(&mut self) -> RunnerResult<()> {
        // FILL
        todo!()
    }

    pub fn find(&self) -> RunnerResult<Option<CnvValue>> {
        // FIND
        todo!()
    }

    pub fn find_all(&self) -> RunnerResult<Vec<CnvValue>> {
        // FINDALL
        todo!()
    }

    pub fn get(&self) -> RunnerResult<Option<CnvValue>> {
        // GET
        todo!()
    }

    pub fn get_marker_pos(&self) -> RunnerResult<usize> {
        // GETMARKERPOS
        todo!()
    }

    pub fn get_size(&self) -> RunnerResult<usize> {
        // GETSIZE
        todo!()
    }

    pub fn get_sum_value(&self) -> RunnerResult<CnvValue> {
        // GETSUMVALUE
        todo!()
    }

    pub fn insert_at(&mut self) -> RunnerResult<()> {
        // INSERTAT
        todo!()
    }

    pub fn load(&mut self) -> RunnerResult<()> {
        // LOAD
        todo!()
    }

    pub fn load_ini(&mut self) -> RunnerResult<()> {
        // LOADINI
        todo!()
    }

    pub fn max(&mut self) -> RunnerResult<()> {
        // MAX
        todo!()
    }

    pub fn max_d(&mut self) -> RunnerResult<()> {
        // MAXD
        todo!()
    }

    pub fn min(&mut self) -> RunnerResult<()> {
        // MIN
        todo!()
    }

    pub fn min_d(&mut self) -> RunnerResult<()> {
        // MIND
        todo!()
    }

    pub fn mod_at(&mut self) -> RunnerResult<()> {
        // MODAT
        todo!()
    }

    pub fn mul(&mut self) -> RunnerResult<()> {
        // MUL
        todo!()
    }

    pub fn mul_a(&mut self) -> RunnerResult<()> {
        // MULA
        todo!()
    }

    pub fn mul_at(&mut self) -> RunnerResult<()> {
        // MULAT
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

    pub fn random_fill(&mut self) -> RunnerResult<()> {
        // RANDOMFILL
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

    pub fn remove_at(&mut self) -> RunnerResult<()> {
        // REMOVEAT
        todo!()
    }

    pub fn reset_marker(&mut self) -> RunnerResult<()> {
        // RESETMARKER
        todo!()
    }

    pub fn reverse_find(&self) -> RunnerResult<Option<CnvValue>> {
        // REVERSEFIND
        todo!()
    }

    pub fn rotate_left(&mut self) -> RunnerResult<()> {
        // ROTATELEFT
        todo!()
    }

    pub fn rotate_right(&mut self) -> RunnerResult<()> {
        // ROTATERIGHT
        todo!()
    }

    pub fn save(&mut self) -> RunnerResult<()> {
        // SAVE
        todo!()
    }

    pub fn save_ini(&mut self) -> RunnerResult<()> {
        // SAVEINI
        todo!()
    }

    pub fn send_on_change(&mut self) -> RunnerResult<()> {
        // SENDONCHANGE
        todo!()
    }

    pub fn set_marker_pos(&mut self) -> RunnerResult<()> {
        // SETMARKERPOS
        todo!()
    }

    pub fn shift_left(&mut self) -> RunnerResult<()> {
        // SHIFTLEFT
        todo!()
    }

    pub fn shift_right(&mut self) -> RunnerResult<()> {
        // SHIFTRIGHT
        todo!()
    }

    pub fn sort(&mut self) -> RunnerResult<()> {
        // SORT
        todo!()
    }

    pub fn sort_many(&mut self) -> RunnerResult<()> {
        // SORTMANY
        todo!()
    }

    pub fn sub(&mut self) -> RunnerResult<()> {
        // SUB
        todo!()
    }

    pub fn sub_a(&mut self) -> RunnerResult<()> {
        // SUBA
        todo!()
    }

    pub fn sub_at(&mut self) -> RunnerResult<()> {
        // SUBAT
        todo!()
    }

    pub fn sum(&mut self) -> RunnerResult<()> {
        // SUM
        todo!()
    }

    pub fn sum_a(&mut self) -> RunnerResult<()> {
        // SUMA
        todo!()
    }

    pub fn swap(&mut self) -> RunnerResult<()> {
        // SWAP
        todo!()
    }
}
