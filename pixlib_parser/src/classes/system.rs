use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct SystemInit {
    // SYSTEM
    pub system: Option<String>, // SYSTEM
}

#[derive(Debug, Clone)]
pub struct System {
    parent: Arc<CnvObject>,
    initial_properties: SystemInit,
}

impl System {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        initial_properties: SystemInit,
    ) -> Self {
        Self {
            parent,
            initial_properties,
        }
    }

    pub fn copy_file() {
        // COPYFILE
        todo!()
    }

    pub fn create_dir() {
        // CREATEDIR
        todo!()
    }

    pub fn delay() {
        // DELAY
        todo!()
    }

    pub fn get_cmd_line_parameter() {
        // GETCMDLINEPARAMETER
        todo!()
    }

    pub fn get_command_line() {
        // GETCOMMANDLINE
        todo!()
    }

    pub fn get_date() {
        // GETDATE
        todo!()
    }

    pub fn get_date_string() {
        // GETDATESTRING
        todo!()
    }

    pub fn get_day() {
        // GETDAY
        todo!()
    }

    pub fn get_day_of_week() {
        // GETDAYOFWEEK
        todo!()
    }

    pub fn get_day_of_week_string() {
        // GETDAYOFWEEKSTRING
        todo!()
    }

    pub fn get_folder_location() {
        // GETFOLDERLOCATION
        todo!()
    }

    pub fn get_hour() {
        // GETHOUR
        todo!()
    }

    pub fn get_mhz() {
        // GETMHZ
        todo!()
    }

    pub fn get_minutes() {
        // GETMINUTES
        todo!()
    }

    pub fn get_month() {
        // GETMONTH
        todo!()
    }

    pub fn get_month_string() {
        // GETMONTHSTRING
        todo!()
    }

    pub fn get_seconds() {
        // GETSECONDS
        todo!()
    }

    pub fn get_system_time() {
        // GETSYSTEMTIME
        todo!()
    }

    pub fn get_time_string() {
        // GETTIMESTRING
        todo!()
    }

    pub fn get_user_name() {
        // GETUSERNAME
        todo!()
    }

    pub fn get_year() {
        // GETYEAR
        todo!()
    }

    pub fn install() {
        // INSTALL
        todo!()
    }

    pub fn is_cmd_line_parameter() {
        // ISCMDLINEPARAMETER
        todo!()
    }

    pub fn is_file_exist() {
        // ISFILEEXIST
        todo!()
    }

    pub fn minimize() {
        // MINIMIZE
        todo!()
    }

    pub fn uninstall() {
        // UNINSTALL
        todo!()
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
        let system = properties.remove("SYSTEM").and_then(discard_if_empty);
        Ok(Self::from_initial_properties(parent, SystemInit { system }))
    }
}
