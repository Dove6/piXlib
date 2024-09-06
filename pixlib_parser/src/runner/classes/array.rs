use std::{any::Any, cell::RefCell};

use pixlib_formats::file_formats::{
    arr::{parse_arr, serialize_arr, ElementData},
    DecodedStr,
};

use super::super::{
    content::EventHandler,
    initable::Initable,
    parsers::{discard_if_empty, parse_bool, parse_event_handler},
};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
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

impl EventHandler for ArrayEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONCHANGE" => self.on_change.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("ADD") => self
                .state
                .borrow_mut()
                .add(arguments)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("ADDAT") => {
                self.state.borrow_mut().add_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ADDCLONES") => {
                self.state.borrow_mut().add_clones().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("CHANGEAT") => {
                self.state.borrow_mut().change_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("CLAMPAT") => {
                self.state.borrow_mut().clamp_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("COMPARE") => {
                self.state.borrow().compare().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("CONTAINS") => {
                self.state.borrow().contains().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("COPYTO") => {
                self.state.borrow_mut().copy_to().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DIR") => {
                self.state.borrow_mut().dir().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DIV") => {
                self.state.borrow_mut().div().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DIVA") => {
                self.state.borrow_mut().div_a().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DIVAT") => {
                self.state.borrow_mut().div_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("FILL") => {
                self.state.borrow_mut().fill().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("FIND") => {
                self.state.borrow().find().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("FINDALL") => {
                self.state.borrow().find_all().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GET") => {
                self.state.borrow().get(arguments[0].to_int() as usize)
            }
            CallableIdentifier::Method("GETMARKERPOS") => {
                self.state.borrow().get_marker_pos().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETSIZE") => {
                self.state.borrow().get_size().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETSUMVALUE") => {
                self.state.borrow().get_sum_value().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("INSERTAT") => {
                self.state.borrow_mut().insert_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("LOAD") => self
                .state
                .borrow_mut()
                .load(context, &arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("LOADINI") => {
                self.state.borrow_mut().load_ini().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MAX") => {
                self.state.borrow_mut().max().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MAXD") => {
                self.state.borrow_mut().max_d().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MIN") => {
                self.state.borrow_mut().min().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MIND") => {
                self.state.borrow_mut().min_d().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MODAT") => {
                self.state.borrow_mut().mod_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MUL") => {
                self.state.borrow_mut().mul().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MULA") => {
                self.state.borrow_mut().mul_a().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("MULAT") => {
                self.state.borrow_mut().mul_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("NEXT") => {
                self.state.borrow_mut().next().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("PREV") => {
                self.state.borrow_mut().prev().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RANDOMFILL") => self
                .state
                .borrow_mut()
                .random_fill()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("REMOVE") => {
                self.state.borrow_mut().remove().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("REMOVEALL") => {
                self.state.borrow_mut().remove_all().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("REMOVEAT") => {
                self.state.borrow_mut().remove_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RESETMARKER") => self
                .state
                .borrow_mut()
                .reset_marker()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("REVERSEFIND") => {
                self.state.borrow().reverse_find().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ROTATELEFT") => self
                .state
                .borrow_mut()
                .rotate_left()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("ROTATERIGHT") => self
                .state
                .borrow_mut()
                .rotate_right()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SAVE") => self
                .state
                .borrow_mut()
                .save(context, &arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SAVEINI") => {
                self.state.borrow_mut().save_ini().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SENDONCHANGE") => self
                .state
                .borrow_mut()
                .send_on_change()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETMARKERPOS") => self
                .state
                .borrow_mut()
                .set_marker_pos()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SHIFTLEFT") => {
                self.state.borrow_mut().shift_left().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SHIFTRIGHT") => self
                .state
                .borrow_mut()
                .shift_right()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SORT") => {
                self.state.borrow_mut().sort().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SORTMANY") => {
                self.state.borrow_mut().sort_many().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SUB") => {
                self.state.borrow_mut().sub().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SUBA") => {
                self.state.borrow_mut().sub_a().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SUBAT") => {
                self.state.borrow_mut().sub_at().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SUM") => {
                self.state.borrow_mut().sum().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SUMA") => {
                self.state.borrow_mut().sum_a().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SWAP") => {
                self.state.borrow_mut().swap().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Event(event_name) => {
                if let Some(code) = self
                    .event_handlers
                    .get(event_name, arguments.first().map(|v| v.to_str()).as_deref())
                {
                    code.run(context).map(|_| CnvValue::Null)
                } else {
                    Ok(CnvValue::Null)
                }
            }
            ident => Err(RunnerError::InvalidCallable {
                object_name: self.parent.name.clone(),
                callable: ident.to_owned(),
            }
            .into()),
        }
    }

    fn new_content(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        let send_on_change = properties
            .remove("SENDONCHANGE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        // TODO: error when there are superfluous properties
        let on_change = properties
            .remove("ONCHANGE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
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

impl Initable for Array {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(Vec::new()),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                })
            });
        Ok(())
    }
}

impl ArrayState {
    pub fn add(&mut self, values: &[CnvValue]) -> anyhow::Result<()> {
        // ADD
        self.values.extend(values.iter().cloned());
        Ok(())
    }

    pub fn add_at(&mut self) -> anyhow::Result<()> {
        // ADDAT
        todo!()
    }

    pub fn add_clones(&mut self) -> anyhow::Result<()> {
        // ADDCLONES
        todo!()
    }

    pub fn change_at(&mut self) -> anyhow::Result<()> {
        // CHANGEAT
        todo!()
    }

    pub fn clamp_at(&mut self) -> anyhow::Result<()> {
        // CLAMPAT
        todo!()
    }

    pub fn compare(&self) -> anyhow::Result<()> {
        // COMPARE
        todo!()
    }

    pub fn contains(&self) -> anyhow::Result<bool> {
        // CONTAINS
        todo!()
    }

    pub fn copy_to(&mut self) -> anyhow::Result<()> {
        // COPYTO
        todo!()
    }

    pub fn dir(&mut self) -> anyhow::Result<()> {
        // DIR
        todo!()
    }

    pub fn div(&mut self) -> anyhow::Result<()> {
        // DIV
        todo!()
    }

    pub fn div_a(&mut self) -> anyhow::Result<()> {
        // DIVA
        todo!()
    }

    pub fn div_at(&mut self) -> anyhow::Result<()> {
        // DIVAT
        todo!()
    }

    pub fn fill(&mut self) -> anyhow::Result<()> {
        // FILL
        todo!()
    }

    pub fn find(&self) -> anyhow::Result<CnvValue> {
        // FIND
        todo!()
    }

    pub fn find_all(&self) -> anyhow::Result<Vec<CnvValue>> {
        // FINDALL
        todo!()
    }

    pub fn get(&self, index: usize) -> anyhow::Result<CnvValue> {
        // GET
        Ok(self.values.get(index).cloned().unwrap_or(CnvValue::Null))
    }

    pub fn get_marker_pos(&self) -> anyhow::Result<usize> {
        // GETMARKERPOS
        todo!()
    }

    pub fn get_size(&self) -> anyhow::Result<usize> {
        // GETSIZE
        todo!()
    }

    pub fn get_sum_value(&self) -> anyhow::Result<CnvValue> {
        // GETSUMVALUE
        todo!()
    }

    pub fn insert_at(&mut self) -> anyhow::Result<()> {
        // INSERTAT
        todo!()
    }

    pub fn load(&mut self, context: RunnerContext, filename: &str) -> anyhow::Result<()> {
        // LOAD
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .write()
            .unwrap()
            .read_scene_asset(
                Arc::clone(&script.runner.game_paths),
                &script.path.with_file_path(filename),
            )
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_arr(&data);
        self.values = data
            .into_iter()
            .map(|e| match e {
                ElementData::Integer(i) => CnvValue::Integer(i),
                ElementData::String(s) => CnvValue::String(s.0),
                ElementData::Boolean(b) => CnvValue::Bool(b),
                ElementData::FixedPoint(d) => CnvValue::Double(d),
            })
            .collect();
        Ok(())
    }

    pub fn load_ini(&mut self) -> anyhow::Result<()> {
        // LOADINI
        todo!()
    }

    pub fn max(&mut self) -> anyhow::Result<()> {
        // MAX
        todo!()
    }

    pub fn max_d(&mut self) -> anyhow::Result<()> {
        // MAXD
        todo!()
    }

    pub fn min(&mut self) -> anyhow::Result<()> {
        // MIN
        todo!()
    }

    pub fn min_d(&mut self) -> anyhow::Result<()> {
        // MIND
        todo!()
    }

    pub fn mod_at(&mut self) -> anyhow::Result<()> {
        // MODAT
        todo!()
    }

    pub fn mul(&mut self) -> anyhow::Result<()> {
        // MUL
        todo!()
    }

    pub fn mul_a(&mut self) -> anyhow::Result<()> {
        // MULA
        todo!()
    }

    pub fn mul_at(&mut self) -> anyhow::Result<()> {
        // MULAT
        todo!()
    }

    pub fn next(&mut self) -> anyhow::Result<()> {
        // NEXT
        todo!()
    }

    pub fn prev(&mut self) -> anyhow::Result<()> {
        // PREV
        todo!()
    }

    pub fn random_fill(&mut self) -> anyhow::Result<()> {
        // RANDOMFILL
        todo!()
    }

    pub fn remove(&mut self) -> anyhow::Result<()> {
        // REMOVE
        todo!()
    }

    pub fn remove_all(&mut self) -> anyhow::Result<()> {
        // REMOVEALL
        self.values.clear();
        Ok(())
    }

    pub fn remove_at(&mut self) -> anyhow::Result<()> {
        // REMOVEAT
        todo!()
    }

    pub fn reset_marker(&mut self) -> anyhow::Result<()> {
        // RESETMARKER
        todo!()
    }

    pub fn reverse_find(&self) -> anyhow::Result<CnvValue> {
        // REVERSEFIND
        todo!()
    }

    pub fn rotate_left(&mut self) -> anyhow::Result<()> {
        // ROTATELEFT
        todo!()
    }

    pub fn rotate_right(&mut self) -> anyhow::Result<()> {
        // ROTATERIGHT
        todo!()
    }

    pub fn save(&mut self, context: RunnerContext, filename: &str) -> anyhow::Result<()> {
        // SAVE
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = serialize_arr(
            &self
                .values
                .iter()
                .map(|v| match v {
                    CnvValue::Integer(i) => ElementData::Integer(*i),
                    CnvValue::Double(d) => ElementData::FixedPoint(*d),
                    CnvValue::Bool(b) => ElementData::Boolean(*b),
                    CnvValue::String(s) => ElementData::String(DecodedStr(s.clone(), None)),
                    CnvValue::Null => ElementData::String(DecodedStr("NULL".to_owned(), None)),
                })
                .collect::<Vec<_>>(),
        )
        .map_err(|e| RunnerError::IoError { source: e })?;
        return Ok(filesystem
            .write()
            .unwrap()
            .write_scene_asset(
                Arc::clone(&script.runner.game_paths),
                &script.path.with_file_path(filename),
                &data,
            )
            .map_err(|e| RunnerError::IoError { source: e })?);
    }

    pub fn save_ini(&mut self) -> anyhow::Result<()> {
        // SAVEINI
        todo!()
    }

    pub fn send_on_change(&mut self) -> anyhow::Result<()> {
        // SENDONCHANGE
        todo!()
    }

    pub fn set_marker_pos(&mut self) -> anyhow::Result<()> {
        // SETMARKERPOS
        todo!()
    }

    pub fn shift_left(&mut self) -> anyhow::Result<()> {
        // SHIFTLEFT
        todo!()
    }

    pub fn shift_right(&mut self) -> anyhow::Result<()> {
        // SHIFTRIGHT
        todo!()
    }

    pub fn sort(&mut self) -> anyhow::Result<()> {
        // SORT
        todo!()
    }

    pub fn sort_many(&mut self) -> anyhow::Result<()> {
        // SORTMANY
        todo!()
    }

    pub fn sub(&mut self) -> anyhow::Result<()> {
        // SUB
        todo!()
    }

    pub fn sub_a(&mut self) -> anyhow::Result<()> {
        // SUBA
        todo!()
    }

    pub fn sub_at(&mut self) -> anyhow::Result<()> {
        // SUBAT
        todo!()
    }

    pub fn sum(&mut self) -> anyhow::Result<()> {
        // SUM
        todo!()
    }

    pub fn sum_a(&mut self) -> anyhow::Result<()> {
        // SUMA
        todo!()
    }

    pub fn swap(&mut self) -> anyhow::Result<()> {
        // SWAP
        todo!()
    }
}
