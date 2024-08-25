use std::{any::Any, cell::RefCell};

use chrono::Local;

use super::super::content::EventHandler;
use super::super::parsers::discard_if_empty;

use crate::parser::ast::ParsedScript;

use super::super::common::*;
use super::super::*;
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
            state: RefCell::new(SystemState {}),
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
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("COPYFILE") => {
                self.state.borrow_mut().copy_file().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("CREATEDIR") => {
                self.state.borrow_mut().create_dir().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("DELAY") => {
                self.state.borrow_mut().delay().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETCMDLINEPARAMETER") => self
                .state
                .borrow()
                .get_cmd_line_parameter()
                .map(CnvValue::String),
            CallableIdentifier::Method("GETCOMMANDLINE") => {
                self.state.borrow().get_command_line().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETDATE") => {
                self.state.borrow().get_date().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETDATESTRING") => {
                self.state.borrow().get_date_string().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETDAY") => self
                .state
                .borrow()
                .get_day()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETDAYOFWEEK") => self
                .state
                .borrow()
                .get_day_of_week()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETDAYOFWEEKSTRING") => self
                .state
                .borrow()
                .get_day_of_week_string()
                .map(CnvValue::String),
            CallableIdentifier::Method("GETFOLDERLOCATION") => self
                .state
                .borrow()
                .get_folder_location()
                .map(CnvValue::String),
            CallableIdentifier::Method("GETHOUR") => self
                .state
                .borrow()
                .get_hour()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETMHZ") => self
                .state
                .borrow()
                .get_mhz()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETMINUTES") => self
                .state
                .borrow()
                .get_minutes()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETMONTH") => self
                .state
                .borrow()
                .get_month()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETMONTHSTRING") => {
                self.state.borrow().get_month_string().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETSECONDS") => self
                .state
                .borrow()
                .get_seconds()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETSYSTEMTIME") => {
                self.state.borrow().get_system_time().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETTIMESTRING") => {
                self.state.borrow().get_time_string().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETUSERNAME") => {
                self.state.borrow().get_user_name().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETYEAR") => self
                .state
                .borrow()
                .get_year()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("INSTALL") => {
                self.state.borrow_mut().install().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISCMDLINEPARAMETER") => self
                .state
                .borrow()
                .is_cmd_line_parameter()
                .map(CnvValue::Bool),
            CallableIdentifier::Method("ISFILEEXIST") => {
                self.state.borrow().is_file_exist().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("MINIMIZE") => {
                self.state.borrow_mut().minimize().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("UNINSTALL") => {
                self.state.borrow_mut().uninstall().map(|_| CnvValue::Null)
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
        let system = properties.remove("SYSTEM").and_then(discard_if_empty);
        Ok(CnvContent::System(System::from_initial_properties(
            parent,
            SystemProperties { system },
        )))
    }
}

impl SystemState {
    pub fn copy_file(&mut self) -> anyhow::Result<()> {
        // COPYFILE
        todo!()
    }

    pub fn create_dir(&mut self) -> anyhow::Result<()> {
        // CREATEDIR
        todo!()
    }

    pub fn delay(&mut self) -> anyhow::Result<()> {
        // DELAY
        todo!()
    }

    pub fn get_cmd_line_parameter(&self) -> anyhow::Result<String> {
        // GETCMDLINEPARAMETER
        todo!()
    }

    pub fn get_command_line(&self) -> anyhow::Result<String> {
        // GETCOMMANDLINE
        todo!()
    }

    pub fn get_date(&self) -> anyhow::Result<String> {
        // GETDATE
        Ok(Local::now().format("%y%m%d").to_string())
    }

    pub fn get_date_string(&self) -> anyhow::Result<String> {
        // GETDATESTRING
        todo!()
    }

    pub fn get_day(&self) -> anyhow::Result<usize> {
        // GETDAY
        todo!()
    }

    pub fn get_day_of_week(&self) -> anyhow::Result<usize> {
        // GETDAYOFWEEK
        todo!()
    }

    pub fn get_day_of_week_string(&self) -> anyhow::Result<String> {
        // GETDAYOFWEEKSTRING
        todo!()
    }

    pub fn get_folder_location(&self) -> anyhow::Result<String> {
        // GETFOLDERLOCATION
        todo!()
    }

    pub fn get_hour(&self) -> anyhow::Result<usize> {
        // GETHOUR
        todo!()
    }

    pub fn get_mhz(&self) -> anyhow::Result<usize> {
        // GETMHZ
        todo!()
    }

    pub fn get_minutes(&self) -> anyhow::Result<usize> {
        // GETMINUTES
        todo!()
    }

    pub fn get_month(&self) -> anyhow::Result<usize> {
        // GETMONTH
        todo!()
    }

    pub fn get_month_string(&self) -> anyhow::Result<String> {
        // GETMONTHSTRING
        todo!()
    }

    pub fn get_seconds(&self) -> anyhow::Result<usize> {
        // GETSECONDS
        todo!()
    }

    pub fn get_system_time(&self) -> anyhow::Result<String> {
        // GETSYSTEMTIME
        todo!()
    }

    pub fn get_time_string(&self) -> anyhow::Result<String> {
        // GETTIMESTRING
        todo!()
    }

    pub fn get_user_name(&self) -> anyhow::Result<String> {
        // GETUSERNAME
        todo!()
    }

    pub fn get_year(&self) -> anyhow::Result<isize> {
        // GETYEAR
        todo!()
    }

    pub fn install(&mut self) -> anyhow::Result<()> {
        // INSTALL
        todo!()
    }

    pub fn is_cmd_line_parameter(&self) -> anyhow::Result<bool> {
        // ISCMDLINEPARAMETER
        todo!()
    }

    pub fn is_file_exist(&self) -> anyhow::Result<bool> {
        // ISFILEEXIST
        todo!()
    }

    pub fn minimize(&mut self) -> anyhow::Result<()> {
        // MINIMIZE
        todo!()
    }

    pub fn uninstall(&mut self) -> anyhow::Result<()> {
        // UNINSTALL
        todo!()
    }
}
