#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod anchors;
pub mod arguments;
pub mod components;
pub mod events_plugin;
pub mod graphics_plugin;
pub mod image;
pub mod inputs_plugin;
pub mod iso;
pub mod resources;
pub mod states;
pub mod systems;
pub mod util;

use std::{
    cell::RefCell,
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

use bevy::{
    ecs::{
        change_detection::DetectChanges,
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter, OnExit},
        system::{Res, ResMut},
    },
    log::{error, info, warn},
    prelude::{default, App, NonSend, PluginGroup, Startup, Update},
    render::texture::ImagePlugin,
    window::{PresentMode, Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use events_plugin::EventsPlugin;
use graphics_plugin::GraphicsPlugin;
use inputs_plugin::InputsPlugin;
use pixlib_parser::{
    classes::{Application, CnvContent, Episode, ObjectBuilderError, Scene},
    common::{DroppableRefMut, Issue, IssueHandler, IssueKind, IssueManager},
    runner::{CnvRunner, FileSystem, GamePaths, RunnerIssue, ScenePath, ScriptSource},
    scanner::parse_cnv,
};
use resources::{
    ChosenScene, DebugSettings, InsertedDisk, InsertedDiskResource, ObjectBuilderIssueManager,
    SceneDefinition, ScriptRunner, WindowConfiguration,
};
use states::AppState;
use systems::{
    cleanup_root, detect_return_to_chooser, draw_cursor, handle_dropped_iso, navigate_chooser,
    setup, setup_chooser, update_chooser_labels,
};
use zip::{result::ZipError, ZipArchive};

const WINDOW_SIZE: (usize, usize) = (800, 600);
const WINDOW_TITLE: &str = "piXlib";

#[allow(clippy::arc_with_non_send_sync)]
fn main() {
    let mut issue_manager: IssueManager<ObjectBuilderError> = Default::default();
    issue_manager.set_handler(Box::new(IssuePrinter));
    let mut runner_issue_manager: IssueManager<RunnerIssue> = Default::default();
    runner_issue_manager.set_handler(Box::new(IssuePrinter));
    let inserted_disk = Arc::new(RefCell::new(
        InsertedDisk::try_from(env::args()).expect("Usage: pixlib path_to_iso [path_to_patch...]"),
    ));
    let layered_fs = Arc::new(RefCell::new(LayeredFileSystem::default()));
    layered_fs.borrow_mut().use_and_drop_mut(|fs| {
        fs.components.insert(0, inserted_disk.clone());
        for arg in env::args().skip(2) {
            let compressed_patch = Arc::new(RefCell::new(
                CompressedPatch::try_from(PathBuf::from(&arg).as_ref()).unwrap(),
            ));
            fs.components.insert(0, compressed_patch);
        }
    });
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (WINDOW_SIZE.0 as f32, WINDOW_SIZE.1 as f32).into(),
                        present_mode: PresentMode::AutoVsync,
                        title: WINDOW_TITLE.to_owned(),
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(WinitSettings::game())
        .insert_resource(WindowConfiguration {
            size: WINDOW_SIZE,
            title: WINDOW_TITLE,
        })
        .insert_resource(DebugSettings {
            force_animation_infinite_looping: false,
        })
        .insert_non_send_resource(InsertedDiskResource(inserted_disk))
        .insert_resource(ChosenScene::default())
        .insert_non_send_resource(ScriptRunner(
            CnvRunner::try_new(
                layered_fs,
                Arc::new(GamePaths::default()),
                runner_issue_manager,
            )
            .unwrap(),
        ))
        .insert_resource(ObjectBuilderIssueManager(issue_manager))
        .init_state::<AppState>()
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cursor)
        .add_systems(OnEnter(AppState::SceneChooser), setup_chooser)
        .add_systems(
            Update,
            (handle_dropped_iso, navigate_chooser, update_chooser_labels)
                .run_if(in_state(AppState::SceneChooser)),
        )
        .add_systems(OnExit(AppState::SceneChooser), cleanup_root)
        .add_systems(Update, reload_main_script)
        .add_systems(
            Update,
            reload_scene_script.run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(
            Update,
            (detect_return_to_chooser).run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(OnExit(AppState::SceneViewer), cleanup_root)
        .add_plugins(GraphicsPlugin)
        .add_plugins(InputsPlugin)
        .add_plugins(EventsPlugin)
        .add_systems(
            Update,
            step_script_runner.run_if(in_state(AppState::SceneViewer)),
        )
        .run();
}

#[derive(Debug)]
struct IssuePrinter;

impl<I: Issue> IssueHandler<I> for IssuePrinter {
    fn handle(&mut self, issue: I) {
        match issue.kind() {
            IssueKind::Warning => warn!("{:?}", issue),
            _ => error!("{:?}", issue),
        }
    }
}

#[derive(Default, Debug)]
pub struct LayeredFileSystem {
    components: Vec<Arc<RefCell<dyn FileSystem>>>,
}

impl FileSystem for LayeredFileSystem {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Vec<u8>> {
        for filesystem in self.components.iter() {
            match filesystem.borrow_mut().read_file(filename) {
                Ok(v) => return Ok(v),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()> {
        for filesystem in self.components.iter() {
            match filesystem.borrow_mut().write_file(filename, data) {
                Err(e) if e.kind() == std::io::ErrorKind::Unsupported => continue,
                Err(e) => return Err(e),
                _ => return Ok(()),
            }
        }
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

pub struct CompressedPatch {
    handle: ZipArchive<File>,
}

impl std::fmt::Debug for CompressedPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompressedPatch")
            .field("handle", &"...")
            .finish()
    }
}

impl FileSystem for CompressedPatch {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Vec<u8>> {
        let sought_name = self
            .handle
            .file_names()
            .find(|n| n.eq_ignore_ascii_case(filename))
            .map(|s| s.to_owned());
        let Some(sought_name) = sought_name else {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
        };
        let mut entry = self
            .handle
            .by_name(&sought_name)
            .map_err(|e| match e {
                ZipError::FileNotFound => std::io::Error::from(std::io::ErrorKind::NotFound),
                ZipError::Io(io_error) => io_error,
                _ => std::io::Error::from(std::io::ErrorKind::Other),
            })
            .inspect_err(|e| eprintln!("{}", e))?;
        if entry.is_file() {
            let mut vec = Vec::new();
            entry.read_to_end(&mut vec)?;
            Ok(vec)
        } else {
            Err(std::io::Error::from(std::io::ErrorKind::NotFound))
        }
    }

    fn write_file(&mut self, _filename: &str, _data: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::from(std::io::ErrorKind::Unsupported))
    }
}

impl CompressedPatch {
    pub fn new(handle: File) -> Result<Self, ZipError> {
        Ok(Self {
            handle: ZipArchive::new(handle)?,
        })
    }
}

impl TryFrom<&Path> for CompressedPatch {
    type Error = ZipError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let file = File::open(path).map_err(ZipError::Io)?;
        Self::new(file)
    }
}

fn step_script_runner(runner: NonSend<ScriptRunner>) {
    runner.0.step().unwrap();
}

fn reload_scene_script(script_runner: NonSend<ScriptRunner>, chosen_scene: Res<ChosenScene>) {
    if !chosen_scene.is_changed() {
        return;
    }
    let game_paths = Arc::clone(&script_runner.game_paths);
    script_runner
        .scripts
        .borrow_mut()
        .remove_scene_script()
        .unwrap();
    let scene_name = chosen_scene.list[chosen_scene.index].name.clone();
    let Some(scene_object) = script_runner.get_object(&scene_name) else {
        panic!("Cannot find defined scene object {}", scene_name); // TODO: check if == 1, not >= 1
    };
    let scene_guard = scene_object.content.borrow();
    let scene: Option<&Scene> = (&*scene_guard).into();
    let scene = scene.unwrap();
    let path = scene.get_script_path().unwrap();
    let path = ScenePath::new(&path, &(scene_name.clone() + ".cnv"));
    let contents = script_runner
        .filesystem
        .borrow_mut()
        .read_scene_file(game_paths, &path)
        .unwrap();
    let contents = parse_cnv(&contents);
    script_runner
        .0
        .load_script(
            path,
            contents.as_parser_input(),
            Some(Arc::clone(&scene_object)),
            ScriptSource::Scene,
        )
        .unwrap();
}

fn reload_main_script(
    inserted_disk: NonSend<InsertedDiskResource>,
    script_runner: NonSend<ScriptRunner>,
    mut chosen_scene: ResMut<ChosenScene>,
) {
    if !inserted_disk.is_changed() {
        return;
    }
    let game_paths = Arc::clone(&script_runner.game_paths);
    script_runner.unload_all_scripts();

    //#region Loading application.def
    let root_script_path = script_runner.game_paths.game_definition_filename.clone();
    let root_script_path = ScenePath::new(".", &root_script_path);
    let contents = script_runner
        .filesystem
        .borrow_mut()
        .read_scene_file(game_paths.clone(), &root_script_path)
        .inspect_err(|e| eprint!("{}", e))
        .unwrap();
    let contents = parse_cnv(&contents);
    script_runner
        .0
        .load_script(
            root_script_path,
            contents.as_parser_input(),
            None,
            ScriptSource::Root,
        )
        .unwrap();
    //#endregion

    let Some(application_object) =
        script_runner.find_object(|o| matches!(&*o.content.borrow(), CnvContent::Application(_)))
    else {
        panic!("Invalid root script - missing application object"); // TODO: check if == 1, not >= 1
    };
    let application_name = application_object.name.clone();
    let application_guard = application_object.content.borrow();
    let application: Option<&Application> = (&*application_guard).into();
    let application = application.unwrap();

    //#region Loading application script
    if let Some(application_script_path) = application.get_script_path() {
        let application_script_path = ScenePath::new(
            &application_script_path,
            &(application_name.clone() + ".cnv"),
        );
        let contents = script_runner
            .filesystem
            .borrow_mut()
            .read_scene_file(game_paths.clone(), &application_script_path)
            .unwrap();
        let contents = parse_cnv(&contents);
        script_runner
            .0
            .load_script(
                application_script_path,
                contents.as_parser_input(),
                Some(Arc::clone(&application_object)),
                ScriptSource::Application,
            )
            .unwrap();
    };
    //#endregion

    let episode_list = application.get_episode_list();
    if episode_list.is_empty() {
        panic!(
            "Invalid application object {} - no episodes defined",
            application_name
        );
    }
    let episode_name = episode_list[0].clone();
    let Some(episode_object) = script_runner.get_object(&episode_name) else {
        panic!("Cannot find defined episode object {}", episode_name); // TODO: check if == 1, not >= 1
    };
    let episode_guard = episode_object.content.borrow();
    let episode: Option<&Episode> = (&*episode_guard).into();
    let episode = episode.unwrap();

    //#region Loading the first episode script
    if let Some(episode_script_path) = episode.get_script_path() {
        let episode_script_path =
            ScenePath::new(&episode_script_path, &(episode_name.clone() + ".cnv"));
        let contents = script_runner
            .filesystem
            .borrow_mut()
            .read_scene_file(game_paths, &episode_script_path)
            .unwrap();
        let contents = parse_cnv(&contents);
        script_runner
            .0
            .load_script(
                episode_script_path,
                contents.as_parser_input(),
                Some(Arc::clone(&episode_object)),
                ScriptSource::Episode,
            )
            .unwrap();
    };
    //#endregion

    let scene_list = episode.get_scene_list();
    if scene_list.is_empty() {
        panic!(
            "Invalid episode object {} - no scenes defined",
            episode_name
        );
    }
    chosen_scene.index = 0;
    chosen_scene.list.clear();
    for scene_name in scene_list {
        let Some(scene_object) = script_runner.get_object(&scene_name) else {
            panic!("Cannot find defined scene object {}", scene_name); // TODO: check if == 1, not >= 1
        };
        let scene_guard = scene_object.content.borrow();
        let scene: Option<&Scene> = (&*scene_guard).into();
        let scene = scene.unwrap();
        chosen_scene.list.push(SceneDefinition {
            name: scene_name,
            path: scene.get_script_path().unwrap().into(),
            background: scene.get_background_path(),
        });
    }
    chosen_scene.list.sort();
    info!("scenes: {:?}", chosen_scene.list);
}
