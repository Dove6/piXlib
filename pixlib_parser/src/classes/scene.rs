use std::{any::Any, cell::RefCell};

use parsers::{discard_if_empty, parse_bool, parse_comma_separated, parse_datetime, parse_program};
use pixlib_formats::file_formats::img::parse_img;
use xxhash_rust::xxh3::xxh3_64;

use crate::{ast::ParsedScript, runner::RunnerError};

use super::*;

#[derive(Debug, Clone)]
pub struct SceneProperties {
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

    pub on_activate: Option<Arc<ParsedScript>>, // ONACTIVATE signal
    pub on_deactivate: Option<Arc<ParsedScript>>, // ONDEACTIVATE signal
    pub on_do_modal: Option<Arc<ParsedScript>>, // ONDOMODAL signal
    pub on_done: Option<Arc<ParsedScript>>,     // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,     // ONINIT signal
    pub on_music_looped: Option<Arc<ParsedScript>>, // ONMUSICLOOPED signal
    pub on_restart: Option<Arc<ParsedScript>>,  // ONRESTART signal
    pub on_signal: Option<Arc<ParsedScript>>,   // ONSIGNAL signal
}

#[derive(Debug, Clone, Default)]
struct SceneState {
    pub initialized: bool, // TODO: run _INIT_

    // initialized from properties
    pub background_data: ImageFileData,
    pub music_data: SoundFileData,

    // deduced from methods
    pub min_hs_priority: isize,
    pub max_hs_priority: isize,
    pub music_frequency: usize,
    pub music_volume: f32,
    pub music_pan: f32,
    pub is_music_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SceneEventHandlers {
    pub on_activate: Option<Arc<ParsedScript>>, // ONACTIVATE signal
    pub on_deactivate: Option<Arc<ParsedScript>>, // ONDEACTIVATE signal
    pub on_do_modal: Option<Arc<ParsedScript>>, // ONDOMODAL signal
    pub on_done: Option<Arc<ParsedScript>>,     // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>,     // ONINIT signal
    pub on_music_looped: Option<Arc<ParsedScript>>, // ONMUSICLOOPED signal
    pub on_restart: Option<Arc<ParsedScript>>,  // ONRESTART signal
    pub on_signal: Option<Arc<ParsedScript>>,   // ONSIGNAL signal
}

#[derive(Debug, Clone)]
pub struct Scene {
    // SCENE
    parent: Arc<CnvObject>,

    state: RefCell<SceneState>,
    event_handlers: SceneEventHandlers,

    author: String,
    coauthors: String,
    creation_time: Option<DateTime<Utc>>,
    is_deamon: bool,
    description: String,
    dlls: Vec<String>,
    last_modify_time: Option<DateTime<Utc>>,
    path: Option<String>,
    version: String,
}

impl Scene {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: SceneProperties) -> Self {
        let scene = Self {
            parent,
            state: RefCell::new(SceneState {
                is_music_enabled: true,
                music_volume: 1f32,
                ..Default::default()
            }),
            event_handlers: SceneEventHandlers {
                on_activate: props.on_activate,
                on_deactivate: props.on_deactivate,
                on_do_modal: props.on_do_modal,
                on_done: props.on_done,
                on_init: props.on_init,
                on_music_looped: props.on_music_looped,
                on_restart: props.on_restart,
                on_signal: props.on_signal,
            },
            author: props.author.unwrap_or_default(),
            coauthors: props.coauthors.unwrap_or_default(),
            creation_time: props.creation_time,
            is_deamon: props.deamon.unwrap_or_default(),
            description: props.description.unwrap_or_default(),
            dlls: props.dlls.unwrap_or_default(),
            last_modify_time: props.last_modify_time,
            path: props.path,
            version: props.version.unwrap_or_default(),
        };
        if let Some(background_filename) = props.background {
            scene.state.borrow_mut().background_data =
                ImageFileData::NotLoaded(background_filename);
        }
        if let Some(music_filename) = props.music {
            scene.state.borrow_mut().music_data = SoundFileData::NotLoaded(music_filename);
        }
        scene
    }

    ///

    pub fn get_script_path(&self) -> Option<String> {
        self.path.clone()
    }

    pub fn get_background_path(&self) -> Option<String> {
        match &self.state.borrow().background_data {
            ImageFileData::NotLoaded(s) => Some(s.clone()),
            ImageFileData::Loaded(LoadedImage { filename, .. }) => filename.clone(),
            _ => None,
        }
    }

    pub fn get_background_to_show(&self) -> RunnerResult<Option<(ImageDefinition, ImageData)>> {
        let mut state = self.state.borrow_mut();
        state.load_background(self).or_else(|e| {
            if matches!(e, RunnerError::MissingFilenameToLoad) {
                Ok(())
            } else {
                Err(e)
            }
        })?;
        let ImageFileData::Loaded(loaded_background) = &state.background_data else {
            return Ok(None);
        };
        let image = &loaded_background.image;
        Ok(Some((image.0.clone(), image.1.clone())))
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
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            CallableIdentifier::Method("CREATEOBJECT") => {
                self.state.borrow_mut().create_object().map(|_| None)
            }
            CallableIdentifier::Method("GETDRAGGEDNAME") => {
                self.state.borrow().get_dragged_name().map(|_| None)
            }
            CallableIdentifier::Method("GETELEMENTSNO") => self
                .state
                .borrow()
                .get_elements_no()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMAXHSPRIORITY") => self
                .state
                .borrow()
                .get_max_hs_priority()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMINHSPRIORITY") => self
                .state
                .borrow()
                .get_min_hs_priority()
                .map(|v| Some(CnvValue::Integer(v as i32))),
            CallableIdentifier::Method("GETMUSICVOLUME") => self
                .state
                .borrow()
                .get_music_volume()
                .map(|v| Some(CnvValue::Double(v as f64))),
            CallableIdentifier::Method("GETOBJECTS") => {
                self.state.borrow().get_objects().map(|_| None)
            }
            CallableIdentifier::Method("GETPLAYINGANIMO") => {
                self.state.borrow().get_playing_animo().map(|_| None)
            }
            CallableIdentifier::Method("GETPLAYINGSEQ") => {
                self.state.borrow().get_playing_seq().map(|_| None)
            }
            CallableIdentifier::Method("GETRUNNINGTIMER") => {
                self.state.borrow().get_running_timer().map(|_| None)
            }
            CallableIdentifier::Method("ISPAUSED") => self
                .state
                .borrow()
                .is_paused()
                .map(|v| Some(CnvValue::Boolean(v))),
            CallableIdentifier::Method("PAUSE") => self.state.borrow_mut().pause().map(|_| None),
            CallableIdentifier::Method("REMOVE") => self.state.borrow_mut().remove().map(|_| None),
            CallableIdentifier::Method("REMOVECLONES") => {
                self.state.borrow_mut().remove_clones().map(|_| None)
            }
            CallableIdentifier::Method("RESUME") => self.state.borrow_mut().resume().map(|_| None),
            CallableIdentifier::Method("RESUMEONLY") => {
                self.state.borrow_mut().resume_only().map(|_| None)
            }
            CallableIdentifier::Method("RESUMESEQONLY") => {
                self.state.borrow_mut().resume_seq_only().map(|_| None)
            }
            CallableIdentifier::Method("RUN") => self.state.borrow_mut().run().map(|_| None),
            CallableIdentifier::Method("RUNCLONES") => {
                self.state.borrow_mut().run_clones().map(|_| None)
            }
            CallableIdentifier::Method("SETMAXHSPRIORITY") => {
                self.state.borrow_mut().set_max_hs_priority().map(|_| None)
            }
            CallableIdentifier::Method("SETMINHSPRIORITY") => {
                self.state.borrow_mut().set_min_hs_priority().map(|_| None)
            }
            CallableIdentifier::Method("SETMUSICFREQ") => {
                self.state.borrow_mut().set_music_freq().map(|_| None)
            }
            CallableIdentifier::Method("SETMUSICPAN") => {
                self.state.borrow_mut().set_music_pan().map(|_| None)
            }
            CallableIdentifier::Method("SETMUSICVOLUME") => {
                self.state.borrow_mut().set_music_volume().map(|_| None)
            }
            CallableIdentifier::Method("STARTMUSIC") => {
                self.state.borrow_mut().start_music().map(|_| None)
            }
            CallableIdentifier::Method("STOPMUSIC") => {
                self.state.borrow_mut().stop_music().map(|_| None)
            }
            CallableIdentifier::Method("TOTIME") => self.state.borrow_mut().to_time().map(|_| None),
            CallableIdentifier::Event("ONACTIVATE") => {
                if let Some(v) = self.event_handlers.on_activate.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONDEACTIVATE") => {
                if let Some(v) = self.event_handlers.on_deactivate.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONDOMODAL") => {
                if let Some(v) = self.event_handlers.on_do_modal.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONDONE") => {
                if let Some(v) = self.event_handlers.on_done.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONINIT") => {
                if let Some(v) = self.event_handlers.on_init.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONMUSICLOOPED") => {
                if let Some(v) = self.event_handlers.on_music_looped.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONRESTART") => {
                if let Some(v) = self.event_handlers.on_restart.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            CallableIdentifier::Event("ONSIGNAL") => {
                if let Some(v) = self.event_handlers.on_signal.as_ref() {
                    v.run(context).map(|_| None)
                } else {
                    Ok(None)
                }
            }
            ident => todo!("{:?} {:?}", self.get_type_id(), ident),
        }
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        todo!()
    }

    fn new(
        parent: Arc<CnvObject>,
        mut properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
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
        Ok(CnvContent::Scene(Self::from_initial_properties(
            parent,
            SceneProperties {
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
        )))
    }
}

impl SceneState {
    pub fn create_object(&mut self) -> RunnerResult<()> {
        // CREATEOBJECT
        todo!()
    }

    pub fn get_dragged_name(&self) -> RunnerResult<String> {
        // GETDRAGGEDNAME
        todo!()
    }

    pub fn get_elements_no(&self) -> RunnerResult<usize> {
        // GETELEMENTSNO
        todo!()
    }

    pub fn get_max_hs_priority(&self) -> RunnerResult<isize> {
        // GETMAXHSPRIORITY
        todo!()
    }

    pub fn get_min_hs_priority(&self) -> RunnerResult<isize> {
        // GETMINHSPRIORITY
        todo!()
    }

    pub fn get_music_volume(&self) -> RunnerResult<f32> {
        // GETMUSICVOLUME
        todo!()
    }

    pub fn get_objects(&self) -> RunnerResult<()> {
        // GETOBJECTS
        todo!()
    }

    pub fn get_playing_animo(&self) -> RunnerResult<()> {
        // GETPLAYINGANIMO
        todo!()
    }

    pub fn get_playing_seq(&self) -> RunnerResult<()> {
        // GETPLAYINGSEQ
        todo!()
    }

    pub fn get_running_timer(&self) -> RunnerResult<()> {
        // GETRUNNINGTIMER
        todo!()
    }

    pub fn is_paused(&self) -> RunnerResult<bool> {
        // ISPAUSED
        todo!()
    }

    pub fn pause(&mut self) -> RunnerResult<()> {
        // PAUSE
        todo!()
    }

    pub fn remove(&mut self) -> RunnerResult<()> {
        // REMOVE
        todo!()
    }

    pub fn remove_clones(&mut self) -> RunnerResult<()> {
        // REMOVECLONES
        todo!()
    }

    pub fn resume(&mut self) -> RunnerResult<()> {
        // RESUME
        todo!()
    }

    pub fn resume_only(&mut self) -> RunnerResult<()> {
        // RESUMEONLY
        todo!()
    }

    pub fn resume_seq_only(&mut self) -> RunnerResult<()> {
        // RESUMESEQONLY
        todo!()
    }

    pub fn run(&mut self) -> RunnerResult<()> {
        // RUN
        todo!()
    }

    pub fn run_clones(&mut self) -> RunnerResult<()> {
        // RUNCLONES
        todo!()
    }

    pub fn set_max_hs_priority(&mut self) -> RunnerResult<()> {
        // SETMAXHSPRIORITY
        todo!()
    }

    pub fn set_min_hs_priority(&mut self) -> RunnerResult<()> {
        // SETMINHSPRIORITY
        todo!()
    }

    pub fn set_music_freq(&mut self) -> RunnerResult<()> {
        // SETMUSICFREQ
        todo!()
    }

    pub fn set_music_pan(&mut self) -> RunnerResult<()> {
        // SETMUSICPAN
        todo!()
    }

    pub fn set_music_volume(&mut self) -> RunnerResult<()> {
        // SETMUSICVOLUME
        todo!()
    }

    pub fn start_music(&mut self) -> RunnerResult<()> {
        // STARTMUSIC
        todo!()
    }

    pub fn stop_music(&mut self) -> RunnerResult<()> {
        // STOPMUSIC
        todo!()
    }

    pub fn to_time(&mut self) -> RunnerResult<()> {
        // TOTIME
        todo!()
    }

    ///

    pub fn load_background(&mut self, scene: &Scene) -> RunnerResult<()> {
        let filename = match &self.background_data {
            ImageFileData::Empty => return Err(RunnerError::MissingFilenameToLoad),
            ImageFileData::Loaded(_) => return Ok(()),
            ImageFileData::NotLoaded(filename) => filename,
        };
        let script = scene.parent.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .borrow()
            .read_scene_file(
                Arc::clone(&script.runner.game_paths),
                Some(scene.path.as_ref().unwrap()),
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
        self.background_data = ImageFileData::Loaded(LoadedImage {
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
}
