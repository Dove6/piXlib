use std::any::Any;

use parsers::{discard_if_empty, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct GroupInit {
    // GROUP
    pub on_done: Option<Arc<IgnorableProgram>>, // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>, // ONINIT signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Group {
    parent: Arc<CnvObject>,
    initial_properties: GroupInit,
}

impl Group {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: GroupInit) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn add() {
        // ADD
        todo!()
    }

    pub fn add_clones() {
        // ADDCLONES
        todo!()
    }

    pub fn clone() {
        // CLONE
        todo!()
    }

    pub fn contains() {
        // CONTAINS
        todo!()
    }

    pub fn get_clone_index() {
        // GETCLONEINDEX
        todo!()
    }

    pub fn get_marker_pos() {
        // GETMARKERPOS
        todo!()
    }

    pub fn get_name() {
        // GETNAME
        todo!()
    }

    pub fn get_name_at_marker() {
        // GETNAMEATMARKER
        todo!()
    }

    pub fn get_size() {
        // GETSIZE
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

    pub fn remove() {
        // REMOVE
        todo!()
    }

    pub fn remove_all() {
        // REMOVEALL
        todo!()
    }

    pub fn reset_marker() {
        // RESETMARKER
        todo!()
    }

    pub fn set_marker_pos() {
        // SETMARKERPOS
        todo!()
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

    fn has_event(&self, name: &str) -> bool {
        matches!(name, "ONDONE" | "ONINIT" | "ONSIGNAL")
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        eprintln!("Skipping method call {:?} for GROUP {:?}", name, self.parent.name); // TODO: fill in
        Ok(None)
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
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
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        Ok(Self::from_initial_properties(
            parent,
            GroupInit {
                on_done,
                on_init,
                on_signal,
            },
        ))
    }
}
