use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct FontInit {
    // FONT
    pub defs: HashMap<FontDef, Option<String>>,

    pub on_done: Option<Arc<IgnorableProgram>>, // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>, // ONINIT signal
    pub on_signal: Option<Arc<IgnorableProgram>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Font {
    parent: Arc<CnvObject>,
    initial_properties: FontInit,
}

impl Font {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: FontInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn get_height() {
        // GETHEIGHT
        todo!()
    }

    pub fn set_color() {
        // SETCOLOR
        todo!()
    }

    pub fn set_family() {
        // SETFAMILY
        todo!()
    }

    pub fn set_size() {
        // SETSIZE
        todo!()
    }

    pub fn set_style() {
        // SETSTYLE
        todo!()
    }
}

lazy_static! {
    static ref FONT_DEF_REGEX: Regex = Regex::new(r"^DEF_(\w+)_(\w+)_(\d+)$").unwrap();
}

impl CnvType for Font {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "FONT"
    }

    fn has_event(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_property(&self, _name: &str) -> bool {
        todo!()
    }

    fn has_method(&self, _name: &str) -> bool {
        todo!()
    }

    fn call_method(
        &mut self,
        _name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        todo!()
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
        let defs: HashMap<FontDef, Option<String>> = properties
            .into_iter()
            .filter_map(|(k, v)| {
                FONT_DEF_REGEX.captures(k.as_ref()).map(|m| {
                    (
                        FontDef {
                            family: m[1].to_owned(),
                            style: m[2].to_owned(),
                            size: m[3].parse().unwrap(),
                        },
                        Some(v),
                    )
                })
            })
            .collect();
        Ok(Self::from_initial_properties(
            parent,
            FontInit {
                defs,
                on_done,
                on_init,
                on_signal,
            },
        ))
    }
}
