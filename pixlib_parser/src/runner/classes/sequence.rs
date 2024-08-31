use std::collections::HashSet;
use std::{any::Any, cell::RefCell};

use ::rand::{seq::SliceRandom, thread_rng};
use xxhash_rust::xxh3::xxh3_64;

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler};

use crate::common::Position;
use crate::parser::seq_parser::{SeqBuilder, SeqEntry, SeqMode, SeqParser, SeqType};
use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct SequenceProperties {
    // SEQUENCE
    pub filename: Option<String>, // FILENAME

    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_finished: HashMap<String, Arc<ParsedScript>>, // ONFINISHED signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_started: HashMap<String, Arc<ParsedScript>>, // ONSTARTED signal
}

#[derive(Debug, Clone, Default)]
struct SequenceState {
    pub initialized: bool,

    // initialized from properties
    pub file_data: SequenceFileData,

    // deduced from methods
    pub is_paused: bool,
    pub music_frequency: usize,
    pub music_volume: f32,
    pub music_pan: f32,

    pub currently_playing: Option<SequenceQueue>,
    pub animation_mapping: HashMap<String, Arc<CnvObject>>,
    pub current_sound: SoundFileData,
}

#[derive(Debug, Clone)]
struct SequenceQueue {
    pub parameter: String,
    pub queue: VecDeque<SeqInstruction>,
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct SequenceEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_finished: HashMap<String, Arc<ParsedScript>>, // ONFINISHED signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_signal: HashMap<String, Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_started: HashMap<String, Arc<ParsedScript>>, // ONSTARTED signal
}

impl EventHandler for SequenceEventHandlers {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONFINISHED" => argument
                .and_then(|a| self.on_finished.get(a))
                .or(self.on_finished.get("")),
            "ONINIT" => self.on_init.as_ref(),
            "ONSIGNAL" => argument
                .and_then(|a| self.on_signal.get(a))
                .or(self.on_signal.get("")),
            "ONSTARTED" => argument
                .and_then(|a| self.on_started.get(a))
                .or(self.on_started.get("")),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    // SEQUENCE
    parent: Arc<CnvObject>,

    state: RefCell<SequenceState>,
    event_handlers: SequenceEventHandlers,
}

impl Sequence {
    pub fn from_initial_properties(parent: Arc<CnvObject>, props: SequenceProperties) -> Self {
        let sequence = Self {
            parent,
            state: RefCell::new(SequenceState {
                music_volume: 1f32,
                ..Default::default()
            }),
            event_handlers: SequenceEventHandlers {
                on_done: props.on_done,
                on_finished: props.on_finished,
                on_init: props.on_init,
                on_signal: props.on_signal,
                on_started: props.on_started,
            },
        };
        if let Some(filename) = props.filename {
            sequence.state.borrow_mut().file_data = SequenceFileData::NotLoaded(filename);
        }
        sequence
    }

    // custom

    pub fn get_currently_played_animation(&self) -> anyhow::Result<Option<Arc<CnvObject>>> {
        self.state.borrow().get_currently_played_animation()
    }

    pub fn is_currently_playing_sound(&self) -> anyhow::Result<bool> {
        self.state.borrow_mut().is_currently_playing_sound()
    }

    pub fn handle_animation_finished(&self) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().handle_animation_finished(context)
    }

    pub fn handle_sound_finished(&self) -> anyhow::Result<()> {
        let context = RunnerContext::new_minimal(&self.parent.parent.runner, &self.parent);
        self.state.borrow_mut().handle_sound_finished(context)
    }
}

impl CnvType for Sequence {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "SEQUENCE"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        // log::trace!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("GETEVENTNAME") => {
                self.state.borrow().get_event_name().map(CnvValue::String)
            }
            CallableIdentifier::Method("GETPLAYING") => {
                self.state.borrow().get_playing().map(CnvValue::String)
            }
            CallableIdentifier::Method("HIDE") => {
                self.state.borrow_mut().hide().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ISPLAYING") => {
                self.state.borrow().is_playing().map(CnvValue::Bool)
            }
            CallableIdentifier::Method("PAUSE") => {
                self.state.borrow_mut().pause().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("PLAY") => self
                .state
                .borrow_mut()
                .play(context, &arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("RESUME") => {
                self.state.borrow_mut().resume().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETFREQ") => {
                self.state.borrow_mut().set_freq().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETPAN") => {
                self.state.borrow_mut().set_pan().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETVOLUME") => {
                self.state.borrow_mut().set_volume().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SHOW") => {
                self.state.borrow_mut().show().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("STOP") => self
                .state
                .borrow_mut()
                .stop(
                    context,
                    arguments.first().map(|v| v.to_bool()).unwrap_or_default(), // TODO: check
                )
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
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let on_done = properties
            .remove("ONDONE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_finished = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONFINISHED" {
                on_finished.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONFINISHED^") {
                on_finished.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let on_init = properties
            .remove("ONINIT")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let mut on_signal = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSIGNAL" {
                on_signal.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONSIGNAL^") {
                on_signal.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        let mut on_started = HashMap::new();
        for (k, v) in properties.iter() {
            if k == "ONSTARTED" {
                on_started.insert(String::from(""), parse_event_handler(v.to_owned())?);
            } else if let Some(argument) = k.strip_prefix("ONSTARTED^") {
                on_started.insert(String::from(argument), parse_event_handler(v.to_owned())?);
            }
        }
        Ok(CnvContent::Sequence(Self::from_initial_properties(
            parent,
            SequenceProperties {
                filename,
                on_done,
                on_finished,
                on_init,
                on_signal,
                on_started,
            },
        )))
    }
}

impl Initable for Sequence {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
        let root_seq =
            self.state
                .borrow_mut()
                .use_and_drop_mut(|state| -> anyhow::Result<Arc<SeqEntry>> {
                    state.load_if_needed(context.clone())?;
                    let SequenceFileData::Loaded(loaded_seq) = &state.file_data else {
                        return Err(RunnerError::NoSequenceDataLoaded(
                            context.current_object.name.clone(),
                        )
                        .into());
                    };
                    Ok(loaded_seq.sequence.clone())
                })?;
        let mut animations_used = HashSet::new();
        root_seq.append_animations_used(&mut animations_used)?;
        let mapping: HashMap<String, Arc<CnvObject>> = animations_used
            .into_iter()
            .map(|filename| {
                let filename_without_extension = filename
                    .to_uppercase()
                    .strip_suffix(".ANN")
                    .map(|s| s.to_owned())
                    .unwrap_or(filename.clone());
                let filename_with_extension = filename_without_extension.clone() + ".ANN";
                (
                    filename_without_extension,
                    filename,
                    filename_with_extension,
                )
            })
            .map(|(filename_no_ext, filename, filename_with_ext)| -> (String, anyhow::Result<Arc<CnvObject>>) {
                if let Some(object) = context.runner.find_object(|o| {
                    if let CnvContent::Animation(animation) = &o.content {
                        animation.get_filename().is_ok_and(|r| {
                            r.is_some_and(|f| {
                                f.eq_ignore_ascii_case(&filename_with_ext)
                            })
                        })
                    } else {
                        false
                    }
                }) {
                    (filename.clone(), Ok(object))
                } else {
                    let name = context.current_object.name.clone()
                        + "_"
                        + &filename_no_ext;
                    let mut created_object = create_object(
                        &context.current_object.parent,
                        &name,
                        &[
                            ("TYPE", "ANIMO"),
                            (
                                "FILENAME",
                                &filename_with_ext,
                            ),
                        ],
                    );
                    if let Ok(ok_object) = &created_object {
                        if let Err(e) = context.current_object.parent.add_object(ok_object.clone()) {
                            created_object = Err(e);
                        };
                    }
                    (filename.clone(), created_object)
                }
            })
            .filter_map(|(filename, object)| object.ok_or_error().map(|o| (filename, o)))
            .collect();
        self.state
            .borrow_mut()
            .use_and_drop_mut(|state| state.animation_mapping = mapping);
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

impl SequenceState {
    pub fn get_event_name(&self) -> anyhow::Result<String> {
        // GETEVENTNAME
        todo!()
    }

    pub fn get_playing(&self) -> anyhow::Result<String> {
        // GETPLAYING
        todo!()
    }

    pub fn hide(&mut self) -> anyhow::Result<()> {
        // HIDE
        for animation_obj in self.animation_mapping.values() {
            let CnvContent::Animation(animation) = &animation_obj.content else {
                unreachable!()
            };
            animation.hide()?;
        }
        Ok(())
    }

    pub fn is_playing(&self) -> anyhow::Result<bool> {
        // ISPLAYING
        Ok(self.currently_playing.is_some())
    }

    pub fn pause(&mut self) -> anyhow::Result<()> {
        // PAUSE
        todo!()
    }

    pub fn play(&mut self, context: RunnerContext, parameter: &str) -> anyhow::Result<()> {
        // PLAY
        if !*context.current_object.initialized.read().unwrap() {
            return Err(RunnerError::NotInitialized(context.current_object.name.clone()).into());
        }
        self.stop(context.clone(), false)?;
        let SequenceFileData::Loaded(LoadedSequence { sequence, .. }) = &self.file_data else {
            return Err(
                RunnerError::NoSequenceDataLoaded(context.current_object.name.clone()).into(),
            );
        };
        let mut queue = VecDeque::new();
        sequence.append_instruction(parameter, &self.animation_mapping, &mut queue)?;
        self.currently_playing = Some(SequenceQueue {
            parameter: parameter.to_owned(),
            queue,
            current_index: None,
        });
        self.step(context)
    }

    pub fn resume(&mut self) -> anyhow::Result<()> {
        // RESUME
        todo!()
    }

    pub fn set_freq(&mut self) -> anyhow::Result<()> {
        // SETFREQ
        todo!()
    }

    pub fn set_pan(&mut self) -> anyhow::Result<()> {
        // SETPAN
        todo!()
    }

    pub fn set_volume(&mut self) -> anyhow::Result<()> {
        // SETVOLUME
        todo!()
    }

    pub fn show(&mut self) -> anyhow::Result<()> {
        // SHOW
        todo!()
    }

    pub fn stop(&mut self, context: RunnerContext, emit_on_finished: bool) -> anyhow::Result<()> {
        // STOP
        self.is_paused = false;
        let Some(mut currently_playing) = self.currently_playing.take() else {
            return Ok(());
        };
        let Some(current_instruction) = currently_playing.queue.pop_front() else {
            return Ok(());
        };
        let CnvContent::Animation(animation) = &current_instruction.animation_object.content else {
            unreachable!();
        };
        animation.stop(false)?;
        if current_instruction.loop_while_spoken.is_some() {
            context
                .runner
                .events_out
                .sound
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(SoundEvent::SoundStopped(SoundSource::Sequence {
                        script_path: context.current_object.parent.path.clone(),
                        object_name: context.current_object.name.clone(),
                    }))
                });
        }
        if emit_on_finished {
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        context: context
                            .clone()
                            .with_arguments(vec![CnvValue::String(currently_playing.parameter)]),
                        callable: CallableIdentifier::Event("ONFINISHED").to_owned(),
                    })
                });
        }
        Ok(())
    }

    // custom

    pub fn load(&mut self, context: RunnerContext, path: &ScenePath) -> anyhow::Result<()> {
        let script = context.current_object.parent.as_ref();
        let filesystem = Arc::clone(&script.runner.filesystem);
        let data = filesystem
            .write()
            .unwrap()
            .read_scene_asset(Arc::clone(&script.runner.game_paths), path)
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let seq_parser = SeqParser::new(
            data.iter().enumerate().map(|(i, b)| {
                Ok((
                    Position {
                        line: 1,
                        column: 1 + i,
                        character: i,
                    },
                    *b as char,
                    Position {
                        line: 1,
                        column: 2 + i,
                        character: i + 1,
                    },
                ))
            }),
            Default::default(),
        )
        .peekable();
        let mut root_seq_name = path.file_path.to_str();
        root_seq_name.drain(
            ..root_seq_name
                .chars()
                .position(|c| c == '/')
                .unwrap_or_default(),
        );
        if root_seq_name.ends_with(".SEQ") {
            root_seq_name.drain((root_seq_name.len() - 4)..);
        }
        let builder = SeqBuilder::new(root_seq_name);
        self.file_data = SequenceFileData::Loaded(LoadedSequence {
            filename: Some(path.file_path.to_str()),
            sequence: builder.build(seq_parser)?,
        });
        Ok(())
    }

    fn load_sound(&mut self, context: RunnerContext, path: &ScenePath) -> anyhow::Result<()> {
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
        self.current_sound = SoundFileData::Loaded(LoadedSound {
            filename: Some(path.file_path.to_str()),
            sound: sound_data.clone(),
        });
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundLoaded {
                    source: SoundSource::Sequence {
                        script_path: context.current_object.parent.path.clone(),
                        object_name: context.current_object.name.clone(),
                    },
                    sound_data,
                })
            });
        Ok(())
    }

    fn play_sound(&mut self, context: RunnerContext, path: &str) -> anyhow::Result<()> {
        if !matches!(self.current_sound, SoundFileData::Loaded(ref loaded) if loaded.filename.as_deref() == Some(path))
        {
            self.load_sound(
                context.clone(),
                &context.current_object.parent.path.with_file_path(path),
            )?;
        }
        context
            .runner
            .events_out
            .sound
            .borrow_mut()
            .use_and_drop_mut(|events| {
                events.push_back(SoundEvent::SoundStopped(SoundSource::Sequence {
                    script_path: context.current_object.parent.path.clone(),
                    object_name: context.current_object.name.clone(),
                }));
                events.push_back(SoundEvent::SoundStarted(SoundSource::Sequence {
                    script_path: context.current_object.parent.path.clone(),
                    object_name: context.current_object.name.clone(),
                }))
            });
        Ok(())
    }

    fn load_if_needed(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        if let SequenceFileData::NotLoaded(ref filename) = self.file_data {
            let path = context.current_object.parent.path.with_file_path(filename);
            self.load(context, &path)?;
        };
        Ok(())
    }

    pub fn get_currently_played_animation(&self) -> anyhow::Result<Option<Arc<CnvObject>>> {
        let Some(currently_playing) = &self.currently_playing else {
            return Ok(None);
        };
        let Some(current_instruction) = currently_playing.queue.front() else {
            return Ok(None);
        };
        Ok(Some(current_instruction.animation_object.clone()))
    }

    pub fn is_currently_playing_sound(&self) -> anyhow::Result<bool> {
        let Some(currently_playing) = &self.currently_playing else {
            return Ok(false);
        };
        let Some(current_instruction) = currently_playing.queue.front() else {
            return Ok(false);
        };
        Ok(current_instruction.loop_while_spoken.is_some())
    }

    pub fn handle_animation_finished(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // log::trace!(
        //     "{}.handle_animation_finished: {:#?}",
        //     context.current_object.name, self.currently_playing
        // );
        let Some(currently_playing) = &mut self.currently_playing else {
            return Err(RunnerError::SeqNotPlaying(context.current_object.name.clone()).into());
        };
        let Some(current_instruction) = currently_playing.queue.front() else {
            return Err(RunnerError::SeqNotPlaying(context.current_object.name.clone()).into());
        };
        let Some(current_index) = currently_playing.current_index else {
            return Err(RunnerError::SeqNotPlaying(context.current_object.name.clone()).into());
        };
        let mut current_index = current_index + 1;
        let seq_len = current_instruction.sequence_names.len();
        if current_instruction.loop_while_spoken.is_some() {
            while current_index >= seq_len {
                current_index -= seq_len;
            }
        }
        currently_playing.current_index = if current_index >= seq_len {
            let _ = currently_playing.queue.pop_front();
            None
        } else {
            Some(current_index)
        };
        self.step(context)
    }

    pub fn handle_sound_finished(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // log::trace!(
        //     "{}.handle_sound_finished: {:#?}",
        //     context.current_object.name, self.currently_playing
        // );
        let Some(currently_playing) = &mut self.currently_playing else {
            return Err(
                RunnerError::SeqNotPlayingSound(context.current_object.name.clone()).into(),
            );
        };
        let Some(current_instruction) = currently_playing.queue.front() else {
            return Err(
                RunnerError::SeqNotPlayingSound(context.current_object.name.clone()).into(),
            );
        };
        if current_instruction.loop_while_spoken.is_none() {
            return Err(
                RunnerError::SeqNotPlayingSound(context.current_object.name.clone()).into(),
            );
        }
        let object = currently_playing
            .queue
            .pop_front()
            .unwrap()
            .animation_object;
        currently_playing.current_index = None;
        let CnvContent::Animation(animation) = &object.content else {
            unreachable!()
        };
        animation.stop(false)?;
        self.step(context)
    }

    fn step(&mut self, context: RunnerContext) -> anyhow::Result<()> {
        // log::trace!(
        //     "{}.step: {:#?}",
        //     context.current_object.name, self.currently_playing
        // );
        if self.currently_playing.is_none() {
            return Err(RunnerError::SeqNotPlaying(context.current_object.name.clone()).into());
        };
        let Some(current_instruction) = self
            .currently_playing
            .as_ref()
            .unwrap()
            .queue
            .front()
            .cloned()
        else {
            let currently_playing = self.currently_playing.take().unwrap();
            // log::trace!(
            //     "Sequence '{}' finished with parameter '{}'",
            //     context.current_object.name, currently_playing.parameter
            // );
            context
                .runner
                .internal_events
                .borrow_mut()
                .use_and_drop_mut(|events| {
                    events.push_back(InternalEvent {
                        context: context
                            .clone()
                            .with_arguments(vec![CnvValue::String(currently_playing.parameter)]),
                        callable: CallableIdentifier::Event("ONFINISHED").to_owned(),
                    })
                });
            self.currently_playing = None;
            self.is_paused = false;
            return Ok(());
        };
        if self
            .currently_playing
            .as_ref()
            .unwrap()
            .current_index
            .is_none()
        {
            if let Some(sound_filename) = &current_instruction.loop_while_spoken {
                self.play_sound(context, sound_filename)?;
            }
            self.currently_playing.as_mut().unwrap().current_index = Some(0);
        }
        let CnvContent::Animation(animation) = &current_instruction.animation_object.content else {
            unreachable!()
        };
        animation.play(
            &current_instruction.sequence_names[self
                .currently_playing
                .as_ref()
                .unwrap()
                .current_index
                .unwrap()],
        )
    }
}

#[derive(Debug, Clone)]
struct SeqInstruction {
    pub animation_object: Arc<CnvObject>,
    pub sequence_names: Vec<String>,
    pub loop_while_spoken: Option<String>,
}

trait CnvSequence {
    fn append_animations_used(&self, buffer: &mut HashSet<String>) -> anyhow::Result<()>;
    fn append_instruction(
        &self,
        parameter: &str,
        animation_mapping: &HashMap<String, Arc<CnvObject>>,
        buffer: &mut VecDeque<SeqInstruction>,
    ) -> anyhow::Result<()>;
}

impl CnvSequence for SeqEntry {
    fn append_animations_used(&self, buffer: &mut HashSet<String>) -> anyhow::Result<()> {
        match &self.r#type {
            SeqType::Simple { filename, .. } => {
                buffer.insert(filename.clone());
            }
            SeqType::Speaking {
                animation_filename, ..
            } => {
                buffer.insert(animation_filename.clone());
            }
            SeqType::Sequence { children, .. } => {
                for child in children.iter() {
                    child.append_animations_used(buffer)?;
                }
            }
        };
        Ok(())
    }

    fn append_instruction(
        &self,
        parameter: &str,
        animation_mapping: &HashMap<String, Arc<CnvObject>>,
        buffer: &mut VecDeque<SeqInstruction>,
    ) -> anyhow::Result<()> {
        match &self.r#type {
            SeqType::Simple { filename, event } => {
                buffer.push_back(SeqInstruction {
                    animation_object: animation_mapping
                        .get(filename)
                        .ok_or(RunnerError::MissingFilenameToLoad)?
                        .clone(),
                    sequence_names: vec![event.to_owned()],
                    loop_while_spoken: None,
                });
            }
            SeqType::Speaking {
                animation_filename,
                sound_filename,
                prefix,
                starting,
                ending,
            } => {
                let object = animation_mapping
                    .get(animation_filename)
                    .ok_or(RunnerError::MissingFilenameToLoad)?;
                let CnvContent::Animation(animation) = &object.content else {
                    unreachable!()
                };
                if *starting {
                    let starting_sequence_name = prefix.to_owned() + "_START";
                    if animation.has_sequence(&starting_sequence_name)? {
                        buffer.push_back(SeqInstruction {
                            animation_object: object.clone(),
                            sequence_names: vec![starting_sequence_name],
                            loop_while_spoken: None,
                        });
                    }
                }
                let speaking_sequences: Vec<String> = (1usize..)
                    .map(|i| prefix.to_owned() + "_" + &i.to_string())
                    .take_while(|name| animation.has_sequence(name).unwrap())
                    .collect();
                if !speaking_sequences.is_empty() {
                    buffer.push_back(SeqInstruction {
                        animation_object: object.clone(),
                        sequence_names: speaking_sequences,
                        loop_while_spoken: Some(sound_filename.clone()),
                    });
                }
                if *ending {
                    let ending_sequence_name = prefix.to_owned() + "_STOP";
                    if animation.has_sequence(&ending_sequence_name)? {
                        buffer.push_back(SeqInstruction {
                            animation_object: object.clone(),
                            sequence_names: vec![ending_sequence_name],
                            loop_while_spoken: None,
                        });
                    }
                }
            }
            SeqType::Sequence { children, mode } => match mode {
                SeqMode::Parameter(parameter_map) => {
                    if let Some(parametrized_child) =
                        parameter_map.get(parameter).and_then(|i| children.get(*i))
                    {
                        parametrized_child.append_instruction(
                            parameter,
                            animation_mapping,
                            buffer,
                        )?;
                    }
                }
                SeqMode::Random => {
                    if let Some(random_child) = children.choose(&mut thread_rng()) {
                        random_child.append_instruction(parameter, animation_mapping, buffer)?;
                    }
                }
                SeqMode::Sequence => {
                    for child in children.iter() {
                        child.append_instruction(parameter, animation_mapping, buffer)?;
                    }
                }
            },
        };
        Ok(())
    }
}
