use std::any::Any;

use parsers::{discard_if_empty, parse_bool, parse_comma_separated, parse_datetime, parse_program};
use pixlib_formats::file_formats::img::parse_img;
use xxhash_rust::xxh3::xxh3_64;

use crate::runner::RunnerError;

use super::*;

#[derive(Debug, Clone)]
pub struct SceneInit {
    // SCENE
    pub author: Option<String>,                  // AUTHOR
    pub background: Option<String>,              // BACKGROUND
    pub coauthors: Option<String>,               // COAUTHORS
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub deamon: Option<bool>,                    // DEAMON
    pub description: Option<String>,             // DESCRIPTION
    pub dlls: Option<Vec<String>>,               // DLLS
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub music: Option<String>,                   // MUSIC
    pub path: Option<String>,                    // PATH
    pub version: Option<String>,                 // VERSION

    pub on_activate: Option<Arc<IgnorableProgram>>, // ONACTIVATE signal
    pub on_deactivate: Option<Arc<IgnorableProgram>>, // ONDEACTIVATE signal
    pub on_do_modal: Option<Arc<IgnorableProgram>>, // ONDOMODAL signal
    pub on_done: Option<Arc<IgnorableProgram>>,     // ONDONE signal
    pub on_init: Option<Arc<IgnorableProgram>>,     // ONINIT signal
    pub on_music_looped: Option<Arc<IgnorableProgram>>, // ONMUSICLOOPED signal
    pub on_restart: Option<Arc<IgnorableProgram>>,  // ONRESTART signal
    pub on_signal: Option<Arc<IgnorableProgram>>,   // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Scene {
    // SCENE
    parent: Arc<CnvObject>,
    initial_properties: SceneInit,

    loaded_background: Option<LoadedImage>,
}

impl Scene {
    pub fn from_initial_properties(parent: Arc<CnvObject>, initial_properties: SceneInit) -> Self {
        // let background_path = initial_properties.background.clone();
        Self {
            parent,
            initial_properties,
            loaded_background: None,
        }
    }

    pub fn create_object() {
        // CREATEOBJECT
        todo!()
    }

    pub fn get_dragged_name() {
        // GETDRAGGEDNAME
        todo!()
    }

    pub fn get_elements_no() {
        // GETELEMENTSNO
        todo!()
    }

    pub fn get_max_hs_priority() {
        // GETMAXHSPRIORITY
        todo!()
    }

    pub fn get_min_hs_priority() {
        // GETMINHSPRIORITY
        todo!()
    }

    pub fn get_music_volume() {
        // GETMUSICVOLUME
        todo!()
    }

    pub fn get_objects() {
        // GETOBJECTS
        todo!()
    }

    pub fn get_playing_animo() {
        // GETPLAYINGANIMO
        todo!()
    }

    pub fn get_playing_seq() {
        // GETPLAYINGSEQ
        todo!()
    }

    pub fn get_running_timer() {
        // GETRUNNINGTIMER
        todo!()
    }

    pub fn is_paused() {
        // ISPAUSED
        todo!()
    }

    pub fn pause() {
        // PAUSE
        todo!()
    }

    pub fn remove() {
        // REMOVE
        todo!()
    }

    pub fn remove_clones() {
        // REMOVECLONES
        todo!()
    }

    pub fn resume() {
        // RESUME
        todo!()
    }

    pub fn resume_only() {
        // RESUMEONLY
        todo!()
    }

    pub fn resume_seq_only() {
        // RESUMESEQONLY
        todo!()
    }

    pub fn run() {
        // RUN
        todo!()
    }

    pub fn run_clones() {
        // RUNCLONES
        todo!()
    }

    pub fn set_max_hs_priority() {
        // SETMAXHSPRIORITY
        todo!()
    }

    pub fn set_min_hs_priority() {
        // SETMINHSPRIORITY
        todo!()
    }

    pub fn set_music_freq() {
        // SETMUSICFREQ
        todo!()
    }

    pub fn set_music_pan() {
        // SETMUSICPAN
        todo!()
    }

    pub fn set_music_volume() {
        // SETMUSICVOLUME
        todo!()
    }

    pub fn start_music() {
        // STARTMUSIC
        todo!()
    }

    pub fn stop_music() {
        // STOPMUSIC
        todo!()
    }

    pub fn to_time() {
        // TOTIME
        todo!()
    }

    ///

    pub fn get_script_path(&self) -> Option<String> {
        self.initial_properties.path.clone()
    }

    pub fn get_background_path(&self) -> Option<String> {
        self.initial_properties.background.clone()
    }

    fn load_background(&mut self, filename: &str) -> RunnerResult<()> {
        let script = self.parent.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .borrow()
            .read_scene_file(
                Arc::clone(&script.runner.game_paths),
                Some(self.initial_properties.path.as_ref().unwrap()),
                filename,
                Some("IMG"),
            )
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_img(&data.0);
        let converted_data = data
            .image_data
            .to_rgba8888(data.header.color_format, data.header.compression_type);
        self.loaded_background = Some(LoadedImage {
            filename: Some(filename.to_owned()),
            image: (
                ImageDefinition {
                    size_px: (data.header.width_px, data.header.height_px),
                    offset_px: (data.header.x_position_px, data.header.y_position_px),
                },
                ImageData {
                    hash: xxh3_64(&converted_data),
                    data: converted_data,
                },
            ),
        });
        Ok(())
    }

    pub fn get_background_to_show(
        &mut self,
    ) -> RunnerResult<Option<(&ImageDefinition, &ImageData)>> {
        if self.loaded_background.is_none() {
            if let Some(ref background_path) = self.initial_properties.background.clone() {
                self.load_background(&background_path)?;
            }
        }
        let Some(loaded_background) = &self.loaded_background else {
            return Ok(None);
        };
        let image = &loaded_background.image;
        Ok(Some((&image.0, &image.1)))
    }
}

impl CnvType for Scene {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SCENE"
    }

    fn has_event(&self, name: &str) -> bool {
        matches!(
            name,
            "ONACTIVATE"
                | "ONDEACTIVATE"
                | "ONDOMODAL"
                | "ONDONE"
                | "ONINIT"
                | "ONMUSICLOOPED"
                | "ONRESTART"
                | "ONSIGNAL"
        )
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
        context: &mut RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.initial_properties.on_init.as_ref() {
                    v.run(context)
                }
                Ok(None)
            }
            _ => todo!(),
        }
    }

    fn get_property(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "BACKGROUND" => self.initial_properties.background.clone().map(|v| v.into()),
            "PATH" => self.initial_properties.path.clone().map(|v| v.into()),
            _ => todo!(),
        }
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let background = properties
            .remove("BACKGROUND")
            .and_then(discard_if_empty)
            .and_then(|s| if s.is_empty() { None } else { Some(s) });
        let coauthors = properties.remove("COAUTHORS").and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let deamon = properties
            .remove("DEAMON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let dlls = properties
            .remove("DLLS")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let music = properties.remove("MUSIC").and_then(discard_if_empty);
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        let on_activate = properties
            .remove("ONACTIVATE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_deactivate = properties
            .remove("ONDEACTIVATE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_do_modal = properties
            .remove("ONDOMODAL")
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
        let on_music_looped = properties
            .remove("ONMUSICLOOPED")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let on_restart = properties
            .remove("ONRESTART")
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
            SceneInit {
                author,
                background,
                coauthors,
                creation_time,
                deamon,
                description,
                dlls,
                last_modify_time,
                music,
                path,
                version,
                on_activate,
                on_deactivate,
                on_do_modal,
                on_done,
                on_init,
                on_music_looped,
                on_restart,
                on_signal,
            },
        ))
    }
}
