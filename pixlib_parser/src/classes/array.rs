use std::any::Any;

use parsers::{discard_if_empty, parse_bool, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct ArrayInit {
    // ARRAY
    pub send_on_change: Option<bool>, // SENDONCHANGE

    pub on_change: Option<Arc<IgnorableProgram>>, // ONCHANGE signal
    pub on_done: Option<Arc<IgnorableProgram>>,   // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>,   // ONINIT signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Array {
    parent: Arc<CnvObject>,
    initial_properties: ArrayInit,
}

impl Array {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: ArrayInit) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn add() {
        // ADD
        todo!()
    }

    pub fn add_at() {
        // ADDAT
        todo!()
    }

    pub fn add_clones() {
        // ADDCLONES
        todo!()
    }

    pub fn change_at() {
        // CHANGEAT
        todo!()
    }

    pub fn clamp_at() {
        // CLAMPAT
        todo!()
    }

    pub fn compare() {
        // COMPARE
        todo!()
    }

    pub fn contains() {
        // CONTAINS
        todo!()
    }

    pub fn copy_to() {
        // COPYTO
        todo!()
    }

    pub fn dir() {
        // DIR
        todo!()
    }

    pub fn div() {
        // DIV
        todo!()
    }

    pub fn div_a() {
        // DIVA
        todo!()
    }

    pub fn div_at() {
        // DIVAT
        todo!()
    }

    pub fn fill() {
        // FILL
        todo!()
    }

    pub fn find() {
        // FIND
        todo!()
    }

    pub fn find_all() {
        // FINDALL
        todo!()
    }

    pub fn get() {
        // GET
        todo!()
    }

    pub fn get_marker_pos() {
        // GETMARKERPOS
        todo!()
    }

    pub fn get_size() {
        // GETSIZE
        todo!()
    }

    pub fn get_sum_value() {
        // GETSUMVALUE
        todo!()
    }

    pub fn insert_at() {
        // INSERTAT
        todo!()
    }

    pub fn load() {
        // LOAD
        todo!()
    }

    pub fn load_ini() {
        // LOADINI
        todo!()
    }

    pub fn max() {
        // MAX
        todo!()
    }

    pub fn max_d() {
        // MAXD
        todo!()
    }

    pub fn min() {
        // MIN
        todo!()
    }

    pub fn min_d() {
        // MIND
        todo!()
    }

    pub fn mod_at() {
        // MODAT
        todo!()
    }

    pub fn mul() {
        // MUL
        todo!()
    }

    pub fn mul_a() {
        // MULA
        todo!()
    }

    pub fn mul_at() {
        // MULAT
        todo!()
    }

    pub fn next() {
        // NEXT
        todo!()
    }

    pub fn prev() {
        // PREV
        todo!()
    }

    pub fn random_fill() {
        // RANDOMFILL
        todo!()
    }

    pub fn remove() {
        // REMOVE
        todo!()
    }

    pub fn remove_all() {
        // REMOVEALL
        todo!()
    }

    pub fn remove_at() {
        // REMOVEAT
        todo!()
    }

    pub fn reset_marker() {
        // RESETMARKER
        todo!()
    }

    pub fn reverse_find() {
        // REVERSEFIND
        todo!()
    }

    pub fn rotate_left() {
        // ROTATELEFT
        todo!()
    }

    pub fn rotate_right() {
        // ROTATERIGHT
        todo!()
    }

    pub fn save() {
        // SAVE
        todo!()
    }

    pub fn save_ini() {
        // SAVEINI
        todo!()
    }

    pub fn send_on_change() {
        // SENDONCHANGE
        todo!()
    }

    pub fn set_marker_pos() {
        // SETMARKERPOS
        todo!()
    }

    pub fn shift_left() {
        // SHIFTLEFT
        todo!()
    }

    pub fn shift_right() {
        // SHIFTRIGHT
        todo!()
    }

    pub fn sort() {
        // SORT
        todo!()
    }

    pub fn sort_many() {
        // SORTMANY
        todo!()
    }

    pub fn sub() {
        // SUB
        todo!()
    }

    pub fn sub_a() {
        // SUBA
        todo!()
    }

    pub fn sub_at() {
        // SUBAT
        todo!()
    }

    pub fn sum() {
        // SUM
        todo!()
    }

    pub fn sum_a() {
        // SUMA
        todo!()
    }

    pub fn swap() {
        // SWAP
        todo!()
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
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            ident => todo!("{:?}.call_method for {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
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
        Ok(Self::from_initial_properties(
            parent,
            ArrayInit {
                send_on_change,
                on_change,
                on_done,
                on_init,
                on_signal,
            },
        ))
    }
}
