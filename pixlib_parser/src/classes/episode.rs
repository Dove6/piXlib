use std::any::Any;

use super::*;

#[derive(Debug, Clone)]
pub struct EpisodeInit {
    pub author: Option<String>,                  // AUTHOR
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub description: Option<String>,             // DESCRIPTION
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub path: Option<String>,                    // PATH
    pub scenes: Option<Vec<SceneName>>,          // SCENES
    pub start_with: Option<SceneName>,           // STARTWITH
    pub version: Option<String>,                 // VERSION
}

#[derive(Debug, Clone)]
pub struct Episode {
    // EPISODE
    initial_properties: EpisodeInit,
}

impl Episode {
    pub fn from_initial_properties(initial_properties: EpisodeInit) -> Self {
        Self {
            initial_properties,
        }
    }

    pub fn back() {
        todo!()
    }

    pub fn get_current_scene() {
        todo!()
    }

    pub fn get_latest_scene() {
        todo!()
    }

    pub fn go_to() {
        todo!()
    }

    pub fn next() {
        todo!()
    }

    pub fn prev() {
        todo!()
    }

    pub fn restart() {
        todo!()
    }
}

impl CnvType for Episode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "EPISODE"
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
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        // println!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("GOTO") => {
                context.runner.change_scene(&arguments[0].to_string());
                Ok(None)
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "PATH" => self.initial_properties.path.clone().map(|v| v.into()),
            "SCENES" => self.initial_properties.scenes.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(path: Arc<Path>, mut properties: HashMap<String, String>, filesystem: &dyn FileSystem) -> Result<Self, TypeParsingError> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let scenes = properties
            .remove("SCENES")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let start_with = properties.remove("STARTWITH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        Ok(Episode::from_initial_properties(EpisodeInit {
            author,
            creation_time,
            description,
            last_modify_time,
            path,
            scenes,
            start_with,
            version,
        }))
    }
}
