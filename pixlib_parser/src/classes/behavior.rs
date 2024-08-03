use std::any::Any;

use parsers::{discard_if_empty, parse_program};

use super::*;

#[derive(Debug, Clone)]
pub struct BehaviorInit {
    // BEHAVIOUR
    pub code: Option<Arc<IgnorableProgram>>, // CODE
    pub condition: Option<ConditionName>,    // CONDITION

    pub on_done: Option<Arc<IgnorableProgram>>, // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>, // ONINIT signal
    pub on_signal: HashMap<String, Arc<IgnorableProgram>>, // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Behavior {
    // BEHAVIOUR
    parent: Arc<CnvObject>,
    initial_properties: BehaviorInit,

    is_enabled: bool,
}

impl Behavior {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: BehaviorInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
            is_enabled: true,
        }
    }

    pub fn break_running(&self) {
        todo!()
    }

    pub fn disable(&mut self) {
        self.is_enabled = false;
    }

    pub fn run(&self, context: &mut RunnerContext) -> RunnerResult<()> {
        if let Some(condition) = self.initial_properties.condition.as_ref() {
            let condition = context.runner.get_object(condition).unwrap();
            match condition.call_method(
                CallableIdentifier::Method("CHECK"),
                &Vec::new(),
                context,
            )? {
                Some(CnvValue::Boolean(false)) => {
                    condition.call_method(
                        CallableIdentifier::Event("ONRUNTIMEFAILED"),
                        &Vec::new(),
                        context,
                    )?;
                    return Ok(());
                }
                Some(CnvValue::Boolean(true)) => {
                    condition.call_method(
                        CallableIdentifier::Event("ONRUNTIMESUCCESS"),
                        &Vec::new(),
                        context,
                    )?;
                }
                _ => todo!(),
            };
        }
        if let Some(v) = self.initial_properties.code.as_ref() {
            v.run(context)
        }
        Ok(())
    }

    pub fn runc(&self) {
        todo!()
    }

    pub fn runlooped(&self) {
        todo!()
    }
}

impl CnvType for Behavior {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "BEHAVIOUR"
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
        arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("BREAK") => {
                self.break_running();
                Ok(None)
            }
            CallableIdentifier::Method("DISABLE") => {
                self.disable();
                Ok(None)
            }
            CallableIdentifier::Method("RUN") => {
                self.run(context);
                Ok(None)
            }
            CallableIdentifier::Method("RUNC") => {
                self.runc();
                Ok(None)
            }
            CallableIdentifier::Method("RUNLOOPED") => {
                self.runlooped();
                Ok(None)
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.initial_properties.on_done.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            CallableIdentifier::Event("ONSIGNAL") => {
                if let Some(v) = self
                    .initial_properties
                    .on_signal
                    .get(&arguments[0].to_string())
                    .as_ref()
                {
                    v.run(context)
                }
                Ok(None)
            }
            _ => todo!(),
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
        let code = properties
            .remove("CODE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let condition = properties.remove("CONDITION").and_then(discard_if_empty);
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
        let mut on_signal = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSIGNAL" {
                on_signal.insert(String::from(""), parse_program(v.to_owned())?);
            } else if k.starts_with("ONSIGNAL^") {
                on_signal.insert(String::from(&k[9..]), parse_program(v.to_owned())?);
            }
        }
        properties.retain(|k, _| k != "ONSIGNAL" && !k.starts_with("ONSIGNAL^"));
        Ok(Behavior::from_initial_properties(
            parent,
            BehaviorInit {
                code,
                condition,
                on_done,
                on_init,
                on_signal,
            },
        ))
    }
}
