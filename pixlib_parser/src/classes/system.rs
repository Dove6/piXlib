use std::{any::Any, cell::RefCell};

use content::EventHandler;
use parsers::discard_if_empty;

use crate::ast::ParsedScript;

use super::*;

#[derive(Debug, Clone)]
pub struct SystemProperties {
    // SYSTEM
    pub system: Option<String>, // SYSTEM
}

#[derive(Debug, Clone, Default)]
struct SystemState {}

#[derive(Debug, Clone)]
pub struct SystemEventHandlers {}

impl EventHandler for SystemEventHandlers {
    fn get(&self, _name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        None
    }
}

#[derive(Debug, Clone)]
pub struct System {
    parent: Arc<CnvObject>,

    state: RefCell<SystemState>,
    event_handlers: SystemEventHandlers,

    system: String,
}

impl System {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: SystemProperties) -> Self {
        Self {
            parent,
            state: RefCell::new(SystemState {
                ..Default::default()
            }),
            event_handlers: SystemEventHandlers {},
            system: props.system.unwrap_or_default(),
        }
    }
}

impl CnvType for System {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SYSTEM"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file().map(|_| None)
            }
            CallableIdentifier::Method("CREATEDIR") => {
                self.state.borrow_mut().create_dir().map(|_| None)
            }
            CallableIdentifier::Method("DELAY") => self.state.borrow_mut().delay().map(|_| None),
            CallableIdentifier::Method("GETCMDLINEPARAMETER") => self
                .state
                .borrow()
                .get_cmd_line_parameter()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETCOMMANDLINE") => self
                .state
                .borrow()
                .get_command_line()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETDATE") => self
                .state
                .borrow()
                .get_date()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETDATESTRING") => self
                .state
                .borrow()
                .get_date_string()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETDAY") => self
                .state
                .borrow()
                .get_day()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETDAYOFWEEK") => self
                .state
                .borrow()
                .get_day_of_week()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETDAYOFWEEKSTRING") => self
                .state
                .borrow()
                .get_day_of_week_string()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETFOLDERLOCATION") => self
                .state
                .borrow()
                .get_folder_location()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETHOUR") => self
                .state
                .borrow()
                .get_hour()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMHZ") => self
                .state
                .borrow()
                .get_mhz()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMINUTES") => self
                .state
                .borrow()
                .get_minutes()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMONTH") => self
                .state
                .borrow()
                .get_month()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMONTHSTRING") => self
                .state
                .borrow()
                .get_month_string()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETSECONDS") => self
                .state
                .borrow()
                .get_seconds()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETSYSTEMTIME") => self
                .state
                .borrow()
                .get_system_time()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETTIMESTRING") => self
                .state
                .borrow()
                .get_time_string()
                .map(|v| Some(CnvValue::String(v))),
            CallableIdentifier::Method("GETUSERNAME") => {
                self.state.borrow().get_user_name().map(|_| None)
            }
            CallableIdentifier::Method("GETYEAR") => self
                .state
                .borrow()
                .get_year()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("INSTALL") => {
                self.state.borrow_mut().install().map(|_| None)
            }
            CallableIdentifier::Method("ISCMDLINEPARAMETER") => self
                .state
                .borrow()
                .is_cmd_line_parameter()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("ISFILEEXIST") => self
                .state
                .borrow()
                .is_file_exist()
                .map(|v| Some(CnvValue::Bool(v))),
            CallableIdentifier::Method("MINIMIZE") => {
                self.state.borrow_mut().minimize().map(|_| None)
            }
            CallableIdentifier::Method("UNINSTALL") => {
                self.state.borrow_mut().uninstall().map(|_| None)
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
        let system = properties.remove("SYSTEM").and_then(discard_if_empty);
        Ok(CnvContent::System(System::from_initial_properties(
            parent,
            SystemProperties { system },
        )))
    }
}

impl SystemState {
    pub fn copy_file(&mut self) -> RunnerResult<()> {
        // COPYFILE
        todo!()
    }

    pub fn create_dir(&mut self) -> RunnerResult<()> {
        // CREATEDIR
        todo!()
    }

    pub fn delay(&mut self) -> RunnerResult<()> {
        // DELAY
        todo!()
    }

    pub fn get_cmd_line_parameter(&self) -> RunnerResult<String> {
        // GETCMDLINEPARAMETER
        todo!()
    }

    pub fn get_command_line(&self) -> RunnerResult<String> {
        // GETCOMMANDLINE
        todo!()
    }

    pub fn get_date(&self) -> RunnerResult<String> {
        // GETDATE
        todo!()
    }

    pub fn get_date_string(&self) -> RunnerResult<String> {
        // GETDATESTRING
        todo!()
    }

    pub fn get_day(&self) -> RunnerResult<usize> {
        // GETDAY
        todo!()
    }

    pub fn get_day_of_week(&self) -> RunnerResult<usize> {
        // GETDAYOFWEEK
        todo!()
    }

    pub fn get_day_of_week_string(&self) -> RunnerResult<String> {
        // GETDAYOFWEEKSTRING
        todo!()
    }

    pub fn get_folder_location(&self) -> RunnerResult<String> {
        // GETFOLDERLOCATION
        todo!()
    }

    pub fn get_hour(&self) -> RunnerResult<usize> {
        // GETHOUR
        todo!()
    }

    pub fn get_mhz(&self) -> RunnerResult<usize> {
        // GETMHZ
        todo!()
    }

    pub fn get_minutes(&self) -> RunnerResult<usize> {
        // GETMINUTES
        todo!()
    }

    pub fn get_month(&self) -> RunnerResult<usize> {
        // GETMONTH
        todo!()
    }

    pub fn get_month_string(&self) -> RunnerResult<String> {
        // GETMONTHSTRING
        todo!()
    }

    pub fn get_seconds(&self) -> RunnerResult<usize> {
        // GETSECONDS
        todo!()
    }

    pub fn get_system_time(&self) -> RunnerResult<String> {
        // GETSYSTEMTIME
        todo!()
    }

    pub fn get_time_string(&self) -> RunnerResult<String> {
        // GETTIMESTRING
        todo!()
    }

    pub fn get_user_name(&self) -> RunnerResult<String> {
        // GETUSERNAME
        todo!()
    }

    pub fn get_year(&self) -> RunnerResult<isize> {
        // GETYEAR
        todo!()
    }

    pub fn install(&mut self) -> RunnerResult<()> {
        // INSTALL
        todo!()
    }

    pub fn is_cmd_line_parameter(&self) -> RunnerResult<bool> {
        // ISCMDLINEPARAMETER
        todo!()
    }

    pub fn is_file_exist(&self) -> RunnerResult<bool> {
        // ISFILEEXIST
        todo!()
    }

    pub fn minimize(&mut self) -> RunnerResult<()> {
        // MINIMIZE
        todo!()
    }

    pub fn uninstall(&mut self) -> RunnerResult<()> {
        // UNINSTALL
        todo!()
    }
}
