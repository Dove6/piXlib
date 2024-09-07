use std::{any::Any, cell::RefCell};

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{
    discard_if_empty, parse_bool, parse_comma_separated, parse_datetime, parse_event_handler,
};
use events::SoundSource;
use pixlib_formats::file_formats::img::parse_img;
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    common::DroppableRefMut,
    parser::ast::ParsedScript,
    runner::{InternalEvent, RunnerError, ScenePath},
};

use super::super::common::*;
use super::super::*;
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
    // initialized from properties
    pub background_data: ImageFileData,
    pub music_data: SoundFileData,

    // deduced from methods
    pub min_hs_priority: isize,
    pub max_hs_priority: isize,
    pub music_frequency: usize,
    pub music_volume_permilles: usize,
    pub music_pan: i32,
    pub is_music_playing: bool,
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

impl EventHandler for SceneEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONACTIVATE" => self.on_activate.as_ref(),
            "ONDEACTIVATE" => self.on_deactivate.as_ref(),
            "ONDOMODAL" => self.on_do_modal.as_ref(),
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONMUSICLOOPED" => self.on_music_looped.as_ref(),
            "ONRESTART" => self.on_restart.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            _ => None,
        }
    }
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
                is_music_playing: true,
                music_volume_permilles: 1000usize,
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

    // custom

    pub fn get_script_path(&self) -> Option<String> {
        self.path.clone()
    }

    pub fn has_background_image(&self) -> bool {
        !matches!(&self.state.borrow().background_data, ImageFileData::Empty)
    }

    pub fn has_background_music(&self) -> bool {
        !matches!(&self.state.borrow().music_data, SoundFileData::Empty)
    }

    pub fn get_music_volume_pan_freq(&self) -> anyhow::Result<(f32, i32, usize)> {
        Ok(self.state.borrow().use_and_drop(|state| {
            (
                state.music_volume_permilles as f32 / 1000f32,
                state.music_pan,
                state.music_frequency,
            )
        }))
    }

    pub fn handle_music_finished(&self) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        if !self.state.borrow().use_and_drop(|s| s.is_music_playing) {
            return Ok(());
        }
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundStarted(SoundSource::BackgroundMusic))
            });
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(Vec::new()),
                    callable: CallableIdentifier::Event("ONMUSICLOOPED").to_owned(),
                })
            });
        Ok(())
    }

    pub fn handle_scene_loaded(&self) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().use_and_drop_mut(|s| {
            s.load_music_if_not_loaded(context.clone())?;
            s.load_background_if_not_loaded(context.clone())
        })?;
        let canvas_observer = context
            .runner
            .find_object(|o| matches!(&o.content, CnvContent::CanvasObserver(_)))
            .unwrap();
        let CnvContent::CanvasObserver(canvas_observer) = &canvas_observer.content else {
            unreachable!();
        };
        canvas_observer.set_background_data(self.state.borrow().background_data.clone())?;
        if self.state.borrow().use_and_drop(|s| s.is_music_playing) {
            if let SoundFileData::Loaded(sound_data) =
                self.state.borrow().use_and_drop(|s| s.music_data.clone())
            {
                context
                    .runner
                    .events_out
                    .sound
                    .borrow_mut()
                    .use_and_drop_mut(|events| {
                        events.push_back(SoundEvent::SoundLoaded {
                            source: SoundSource::BackgroundMusic,
                            sound_data: sound_data.sound,
                        });
                        events.push_back(SoundEvent::SoundStarted(SoundSource::BackgroundMusic));
                    });
            }
        }
        Ok(())
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

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        match name {
            CallableIdentifier::Method("CREATEOBJECT") => self
                .state
                .borrow_mut()
                .create_object()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETDRAGGEDNAME") => self
                .state
                .borrow()
                .get_dragged_name()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETELEMENTSNO") => self
                .state
                .borrow()
                .get_elements_no()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETMAXHSPRIORITY") => self
                .state
                .borrow()
                .get_max_hs_priority()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETMINHSPRIORITY") => self
                .state
                .borrow()
                .get_min_hs_priority()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETMUSICVOLUME") => self
                .state
                .borrow()
                .get_music_volume()
                .map(|_| CnvValue::Integer(-10000)), // EDGE CASE: this seems to be broken
            CallableIdentifier::Method("GETOBJECTS") => {
                self.state.borrow().get_objects().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("GETPLAYINGANIMO") => self
                .state
                .borrow()
                .get_playing_animo()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETPLAYINGSEQ") => self
                .state
                .borrow()
                .get_playing_seq()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETRUNNINGTIMER") => self
                .state
                .borrow()
                .get_running_timer()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("ISPAUSED") => {
                self.state.borrow().is_paused().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("PAUSE") => {
                self.state.borrow_mut().pause().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("REMOVE") => {
                self.state.borrow_mut().remove().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("REMOVECLONES") => self
                .state
                .borrow_mut()
                .remove_clones()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("RESUME") => {
                self.state.borrow_mut().resume().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("RESUMEONLY") => self
                .state
                .borrow_mut()
                .resume_only()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("RESUMESEQONLY") => self
                .state
                .borrow_mut()
                .resume_seq_only()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("RUN") => self
                .state
                .borrow_mut()
                .run(
                    context,
                    arguments[0].to_str(),
                    arguments[1].to_str(),
                    arguments.iter().skip(2).map(|v| v.to_owned()).collect(),
                )
                .map(|_| CnvValue::Null), // TODO: return something
            CallableIdentifier::Method("RUNCLONES") => {
                self.state.borrow_mut().run_clones().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETMAXHSPRIORITY") => self
                .state
                .borrow_mut()
                .set_max_hs_priority()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETMINHSPRIORITY") => self
                .state
                .borrow_mut()
                .set_min_hs_priority()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETMUSICFREQ") => self
                .state
                .borrow_mut()
                .set_music_freq()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETMUSICPAN") => self
                .state
                .borrow_mut()
                .set_music_pan()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETMUSICVOLUME") => self
                .state
                .borrow_mut()
                .set_music_volume(arguments[0].to_int() as usize)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("STARTMUSIC") => self
                .state
                .borrow_mut()
                .start_music(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("STOPMUSIC") => self
                .state
                .borrow_mut()
                .stop_music(context)
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("TOTIME") => self
                .state
                .borrow_mut()
                .convert_to_time()
                .map(|_| CnvValue::Null),
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
            .map(parse_event_handler)
            .transpose()?;
        let on_deactivate = properties
            .remove("ONDEACTIVATE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_do_modal = properties
            .remove("ONDOMODAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_music_looped = properties
            .remove("ONMUSICLOOPED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_restart = properties
            .remove("ONRESTART")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
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

impl Initable for Scene {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        let mut state = self.state.borrow_mut();
        if let ImageFileData::NotLoaded(filename) = &state.background_data {
            let path = ScenePath::new(self.path.as_ref().unwrap(), filename);
            state.load_background(context.clone(), &path)?;
        };
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(InternalEvent {
                    context: context.clone().with_arguments(Vec::new()),
                    callable: CallableIdentifier::Event("ONINIT").to_owned(),
                })
            });
        Ok(())
    }
}

impl SceneState {
    pub fn create_object(&mut self) -> anyhow::Result<()> {
        // CREATEOBJECT
        todo!()
    }

    pub fn get_dragged_name(&self) -> anyhow::Result<String> {
        // GETDRAGGEDNAME
        todo!()
    }

    pub fn get_elements_no(&self) -> anyhow::Result<usize> {
        // GETELEMENTSNO
        todo!()
    }

    pub fn get_max_hs_priority(&self) -> anyhow::Result<isize> {
        // GETMAXHSPRIORITY
        todo!()
    }

    pub fn get_min_hs_priority(&self) -> anyhow::Result<isize> {
        // GETMINHSPRIORITY
        todo!()
    }

    pub fn get_music_volume(&self) -> anyhow::Result<usize> {
        // GETMUSICVOLUME
        Ok(self.music_volume_permilles)
    }

    pub fn get_objects(&self) -> anyhow::Result<()> {
        // GETOBJECTS
        todo!()
    }

    pub fn get_playing_animo(&self) -> anyhow::Result<()> {
        // GETPLAYINGANIMO
        todo!()
    }

    pub fn get_playing_seq(&self) -> anyhow::Result<()> {
        // GETPLAYINGSEQ
        todo!()
    }

    pub fn get_running_timer(&self) -> anyhow::Result<()> {
        // GETRUNNINGTIMER
        todo!()
    }

    pub fn is_paused(&self) -> anyhow::Result<bool> {
        // ISPAUSED
        todo!()
    }

    pub fn pause(&mut self) -> anyhow::Result<()> {
        // PAUSE
        todo!()
    }

    pub fn remove(&mut self) -> anyhow::Result<()> {
        // REMOVE
        todo!()
    }

    pub fn remove_clones(&mut self) -> anyhow::Result<()> {
        // REMOVECLONES
        todo!()
    }

    pub fn resume(&mut self) -> anyhow::Result<()> {
        // RESUME
        todo!()
    }

    pub fn resume_only(&mut self) -> anyhow::Result<()> {
        // RESUMEONLY
        todo!()
    }

    pub fn resume_seq_only(&mut self) -> anyhow::Result<()> {
        // RESUMESEQONLY
        todo!()
    }

    pub fn run(
        &mut self,
        context: RunnerContext,
        object_name: String,
        method_name: String,
        arguments: Vec<CnvValue>,
    ) -> anyhow::Result<()> {
        // RUN
        let Some(object) = context.runner.get_object(&object_name) else {
            return Err(RunnerError::ObjectNotFound { name: object_name }.into());
        };
        let evt_context = RunnerContext::new(&context.runner, &object, &object, &arguments);
        context
            .runner
            .internal_events
            .borrow_mut()
            .use_and_drop_mut(move |events| {
                events.push_back(InternalEvent {
                    context: evt_context,
                    callable: CallableIdentifierOwned::Method(method_name),
                })
            });
        Ok(())
    }

    pub fn run_clones(&mut self) -> anyhow::Result<()> {
        // RUNCLONES
        todo!()
    }

    pub fn set_max_hs_priority(&mut self) -> anyhow::Result<()> {
        // SETMAXHSPRIORITY
        todo!()
    }

    pub fn set_min_hs_priority(&mut self) -> anyhow::Result<()> {
        // SETMINHSPRIORITY
        todo!()
    }

    pub fn set_music_freq(&mut self) -> anyhow::Result<()> {
        // SETMUSICFREQ
        todo!()
    }

    pub fn set_music_pan(&mut self) -> anyhow::Result<()> {
        // SETMUSICPAN
        todo!()
    }

    pub fn set_music_volume(&mut self, volume_permilles: usize) -> anyhow::Result<()> {
        // SETMUSICVOLUME
        self.music_volume_permilles = volume_permilles;
        Ok(())
    }

    pub fn start_music(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // STARTMUSIC
        if self.is_music_playing {
            return Ok(());
        }
        self.is_music_playing = true;
        if context
            .runner
            .get_current_scene()
            .is_some_and(|o| context.current_object == o)
        {
            context
                .runner
                .events_out
                .sound
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(SoundEvent::SoundStarted(SoundSource::BackgroundMusic))
                });
        }
        Ok(())
    }

    pub fn stop_music(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // STOPMUSIC
        if !self.is_music_playing {
            return Ok(());
        }
        self.is_music_playing = false;
        if context
            .runner
            .get_current_scene()
            .is_some_and(|o| context.current_object == o)
        {
            context
                .runner
                .events_out
                .sound
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(SoundEvent::SoundStopped(SoundSource::BackgroundMusic))
                });
        }
        Ok(())
    }

    pub fn convert_to_time(&mut self) -> anyhow::Result<()> {
        // TOTIME
        todo!()
    }

    // custom

    pub fn load_background(
        &mut self,
        context: RunnerContext,
        path: &ScenePath,
    ) -> anyhow::Result<()> {
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .write()
            .unwrap()
            .read_scene_asset(Arc::clone(&script.runner.game_paths), path)
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_img(&data)
            .ok_or_error()
            .ok_or(RunnerError::CouldNotLoadFile(path.to_str()))?;
        let converted_data = data
            .image_data
            .to_rgba8888(data.header.color_format, data.header.compression_type);
        self.background_data = ImageFileData::Loaded(LoadedImage {
            filename: Some(path.file_path.to_str()),
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

    pub fn load_background_if_not_loaded(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        if let ImageFileData::NotLoaded(filename) = &self.background_data {
            let CnvContent::Scene(scene) = &context.current_object.content else {
                unreachable!();
            };
            let path = ScenePath::new(scene.path.as_deref().unwrap_or_default(), filename);
            self.load_background(context, &path)
        } else {
            Ok(())
        }
    }

    pub fn load_music(&mut self, context: RunnerContext, path: &ScenePath) -> anyhow::Result<()> {
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .write()
            .unwrap()
            .read_sound(Arc::clone(&script.runner.game_paths), path)
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let sound_data = SoundData {
            hash: xxh3_64(&data),
            data,
        };
        self.music_data = SoundFileData::Loaded(LoadedSound {
            filename: Some(path.file_path.to_str()),
            sound: sound_data.clone(),
        });
        Ok(())
    }

    pub fn load_music_if_not_loaded(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        if let SoundFileData::NotLoaded(filename) = &self.music_data {
            let CnvContent::Scene(scene) = &context.current_object.content else {
                unreachable!();
            };
            let path = ScenePath::new(scene.path.as_deref().unwrap_or_default(), filename);
            self.load_music(context, &path)
        } else {
            Ok(())
        }
    }
}
