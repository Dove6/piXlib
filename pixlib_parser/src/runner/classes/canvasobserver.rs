use lazy_static::lazy_static;
use pixlib_formats::file_formats::img::parse_img;
use std::any::Any;
use std::sync::RwLock;
use xxhash_rust::xxh3::xxh3_64;

use super::super::content::EventHandler;
use super::super::initable::Initable;
use super::super::parsers::{discard_if_empty, parse_event_handler};

use crate::{common::DroppableRefMut, parser::ast::ParsedScript, runner::InternalEvent};

use super::super::common::*;
use super::super::*;
use super::*;

#[derive(Debug, Clone)]
pub struct CanvasObserverProperties {
    // CANVAS_OBSERVER
    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_initial_update: Option<Arc<ParsedScript>>, // ONINITIALUPDATE signal
    pub on_initial_updated: Option<Arc<ParsedScript>>, // ONINITIALUPDATED signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_update: Option<Arc<ParsedScript>>, // ONUPDATE signal
    pub on_updated: Option<Arc<ParsedScript>>, // ONUPDATED signal
    pub on_window_focus_off: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSOFF signal
    pub on_window_focus_on: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSON signal
}

#[derive(Debug, Clone, Default)]
struct CanvasObserverState {
    // deduced from methods
    background_data: ImageFileData,
    background_position: (isize, isize),
    // graphics: Vec<Arc<CnvObject>>,
}

#[derive(Debug, Clone)]
pub struct CanvasObserverEventHandlers {
    pub on_done: Option<Arc<ParsedScript>>, // ONDONE signal
    pub on_init: Option<Arc<ParsedScript>>, // ONINIT signal
    pub on_initial_update: Option<Arc<ParsedScript>>, // ONINITIALUPDATE signal
    pub on_initial_updated: Option<Arc<ParsedScript>>, // ONINITIALUPDATED signal
    pub on_signal: Option<Arc<ParsedScript>>, // ONSIGNAL signal
    pub on_update: Option<Arc<ParsedScript>>, // ONUPDATE signal
    pub on_updated: Option<Arc<ParsedScript>>, // ONUPDATED signal
    pub on_window_focus_off: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSOFF signal
    pub on_window_focus_on: Option<Arc<ParsedScript>>, // ONWINDOWFOCUSON signal
}

impl EventHandler for CanvasObserverEventHandlers {
    fn get(&self, name: &str, _argument: Option<&str>) -> Option<&Arc<ParsedScript>> {
        match name {
            "ONDONE" => self.on_done.as_ref(),
            "ONINIT" => self.on_init.as_ref(),
            "ONINITIALUPDATE" => self.on_initial_update.as_ref(),
            "ONINITIALUPDATED" => self.on_initial_updated.as_ref(),
            "ONSIGNAL" => self.on_signal.as_ref(),
            "ONUPDATE" => self.on_update.as_ref(),
            "ONUPDATED" => self.on_updated.as_ref(),
            "ONWINDOWFOCUSOFF" => self.on_window_focus_off.as_ref(),
            "ONWINDOWFOCUSON" => self.on_window_focus_on.as_ref(),
            _ => None,
        }
    }
}

lazy_static! {
    static ref GLOBAL_CANVAS_OBSERVER_STATE: Arc<RwLock<CanvasObserverState>> =
        Arc::new(RwLock::new(CanvasObserverState {
            ..Default::default()
        }));
}

#[derive(Debug, Clone)]
pub struct CanvasObserver {
    parent: Arc<CnvObject>,

    state: Arc<RwLock<CanvasObserverState>>,
    event_handlers: CanvasObserverEventHandlers,
}

impl CanvasObserver {
    pub fn from_initial_properties(
        parent: Arc<CnvObject>,
        props: CanvasObserverProperties,
    ) -> Self {
        Self {
            parent,
            state: Arc::clone(&GLOBAL_CANVAS_OBSERVER_STATE),
            event_handlers: CanvasObserverEventHandlers {
                on_done: props.on_done,
                on_init: props.on_init,
                on_initial_update: props.on_initial_update,
                on_initial_updated: props.on_initial_updated,
                on_signal: props.on_signal,
                on_update: props.on_update,
                on_updated: props.on_updated,
                on_window_focus_off: props.on_window_focus_off,
                on_window_focus_on: props.on_window_focus_on,
            },
        }
    }

    pub fn set_background_data(&self, background_data: ImageFileData) -> anyhow::Result<()> {
        GLOBAL_CANVAS_OBSERVER_STATE
            .write()
            .unwrap()
            .use_and_drop_mut(|state| state.background_data = background_data);
        Ok(())
    }

    pub fn get_background_to_show(&self) -> anyhow::Result<Option<(ImageDefinition, ImageData)>> {
        let runner = &self.parent.parent.runner;
        let mut state = GLOBAL_CANVAS_OBSERVER_STATE.write().unwrap();
        if let ImageFileData::NotLoaded(filename) = &state.background_data {
            let Some(current_scene) = runner.get_current_scene() else {
                return Ok(None);
            };
            let CnvContent::Scene(current_scene) = &current_scene.content else {
                unreachable!();
            };
            let path = ScenePath::new(&current_scene.get_script_path().unwrap(), filename);
            state.load_background(runner, &path)?;
        } else if let ImageFileData::Empty = &state.background_data {
            return Ok(None);
        }
        let ImageFileData::Loaded(loaded_background) = &state.background_data else {
            unreachable!();
        };
        let image = &loaded_background.image;
        Ok(Some((image.0.clone(), image.1.clone())))
    }
}

impl CnvType for CanvasObserver {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_type_id(&self) -> &'static str {
        "CANVASOBSERVER"
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue> {
        // log::trace!("Calling method: {:?} of object: {:?}", name, self);
        match name {
            CallableIdentifier::Method("ADD") => {
                self.state.write().unwrap().add().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("ENABLENOTIFY") => self
                .state
                .write()
                .unwrap()
                .enable_notify()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("GETBPP") => self
                .state
                .read()
                .unwrap()
                .get_bpp()
                .map(|v| CnvValue::Integer(v as i32)),
            CallableIdentifier::Method("GETGRAPHICSAT") => self
                .state
                .read()
                .unwrap()
                .get_graphics_at()
                .map(|v| v.map(CnvValue::String).unwrap_or_default()),
            CallableIdentifier::Method("GETGRAPHICSAT2") => self
                .state
                .read()
                .unwrap()
                .get_graphics_at2()
                .map(|v| v.map(CnvValue::String).unwrap_or_default()),
            CallableIdentifier::Method("MOVEBKG") => self
                .state
                .write()
                .unwrap()
                .move_bkg()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("PASTE") => {
                self.state.write().unwrap().paste().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("REDRAW") => {
                self.state.write().unwrap().redraw().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("REFRESH") => self
                .state
                .write()
                .unwrap()
                .refresh()
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("REMOVE") => {
                self.state.write().unwrap().remove().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SAVE") => {
                self.state.write().unwrap().save().map(|_| CnvValue::Null)
            }
            CallableIdentifier::Method("SETBACKGROUND") => self
                .state
                .write()
                .unwrap()
                .set_background(context, &arguments[0].to_str())
                .map(|_| CnvValue::Null),
            CallableIdentifier::Method("SETBKGPOS") => self
                .state
                .write()
                .unwrap()
                .set_bkg_pos()
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
        let on_initial_update = properties
            .remove("ONINITIALUPDATE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_initial_updated = properties
            .remove("ONINITIALUPDATED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_signal = properties
            .remove("ONSIGNAL")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_update = properties
            .remove("ONUPDATE")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_updated = properties
            .remove("ONUPDATED")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_window_focus_off = properties
            .remove("ONWINDOWFOCUSOFF")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        let on_window_focus_on = properties
            .remove("ONWINDOWFOCUSON")
            .and_then(discard_if_empty)
            .map(parse_event_handler)
            .transpose()?;
        Ok(CnvContent::CanvasObserver(Self::from_initial_properties(
            parent,
            CanvasObserverProperties {
                on_done,
                on_init,
                on_initial_update,
                on_initial_updated,
                on_signal,
                on_update,
                on_updated,
                on_window_focus_off,
                on_window_focus_on,
            },
        )))
    }
}

impl Initable for CanvasObserver {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()> {
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

impl CanvasObserverState {
    pub fn add(&mut self) -> anyhow::Result<()> {
        // ADD
        todo!()
    }

    pub fn enable_notify(&mut self) -> anyhow::Result<()> {
        // ENABLENOTIFY
        todo!()
    }

    pub fn get_bpp(&self) -> anyhow::Result<usize> {
        // GETBPP
        Ok(32)
    }

    pub fn get_graphics_at(&self) -> anyhow::Result<Option<String>> {
        // GETGRAPHICSAT
        todo!()
    }

    pub fn get_graphics_at2(&self) -> anyhow::Result<Option<String>> {
        // GETGRAPHICSAT2
        todo!()
    }

    pub fn move_bkg(&mut self) -> anyhow::Result<()> {
        // MOVEBKG
        todo!()
    }

    pub fn paste(&mut self) -> anyhow::Result<()> {
        // PASTE
        todo!()
    }

    pub fn redraw(&mut self) -> anyhow::Result<()> {
        // REDRAW
        todo!()
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        // REFRESH
        todo!()
    }

    pub fn remove(&mut self) -> anyhow::Result<()> {
        // REMOVE
        todo!()
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        // SAVE
        todo!()
    }

    pub fn set_background(
        &mut self,
        context: RunnerContext,
        object_name: &str,
    ) -> anyhow::Result<()> {
        // SETBACKGROUND
        let Some(object) = context.runner.get_object(object_name) else {
            return Err(RunnerError::ObjectNotFound {
                name: object_name.to_owned(),
            }
            .into());
        };
        let CnvContent::Image(image) = &object.content else {
            return Err(RunnerError::ExpectedGraphicsObject.into());
        };
        self.background_data = image.get_file_data()?;
        Ok(())
    }

    pub fn set_bkg_pos(&mut self) -> anyhow::Result<()> {
        // SETBKGPOS
        todo!()
    }

    // custom

    pub fn load_background(
        &mut self,
        runner: &Arc<CnvRunner>,
        path: &ScenePath,
    ) -> anyhow::Result<()> {
        let filesystem = Arc::clone(&runner.filesystem);
        let data = filesystem
            .write()
            .unwrap()
            .read_scene_asset(Arc::clone(&runner.game_paths), path)
            .map_err(|_| RunnerError::IoError {
                source: std::io::Error::from(std::io::ErrorKind::NotFound),
            })?;
        let data = parse_img(&data);
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
}
