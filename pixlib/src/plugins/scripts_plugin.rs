use std::{
    env,
    ops::{Deref, DerefMut},
    sync::{Arc, RwLock},
};

use bevy::{
    app::{App, Plugin, Startup, Update},
    asset::{AssetServer, Handle},
    log::{error, trace, warn},
    prelude::{in_state, DetectChanges, IntoSystemConfigs, NextState, NonSend, Res, ResMut},
};
#[cfg(not(target_family = "wasm"))]
use pixlib_parser::filesystems::GameDirectory;
use pixlib_parser::{
    common::{IssueManager, LoggableToOption},
    runner::{CnvContent, CnvRunner, FileSystem, GamePaths, RunnerIssue, ScenePath, ScriptSource},
    scanner::parse_cnv,
};

use crate::{
    filesystems::{FileSystemResource, PendingHandle},
    resources::{ChosenScene, SceneDefinition},
    util::IssuePrinter,
    AppState,
};

use super::ui_plugin::Blob;

#[derive(Debug)]
pub struct ScriptsPlugin {
    pub filesystem: Arc<RwLock<dyn FileSystem>>,
    pub window_resolution: (usize, usize),
}

#[allow(clippy::arc_with_non_send_sync)]
impl Plugin for ScriptsPlugin {
    fn build(&self, app: &mut App) {
        let mut runner_issue_manager: IssueManager<RunnerIssue> = Default::default();
        runner_issue_manager.set_handler(Box::new(IssuePrinter));
        app.insert_non_send_resource(ScriptRunner(
            CnvRunner::try_new(
                self.filesystem.clone(),
                Arc::new(GamePaths::default()),
                self.window_resolution,
            )
            .unwrap(),
        ))
        .add_systems(Startup, read_args)
        .add_systems(Update, reload_main_script)
        .add_systems(
            Update,
            reload_scene_script.run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(
            Update,
            step_script_runner.run_if(in_state(AppState::SceneViewer)),
        );
    }
}

fn step_script_runner(runner: NonSend<ScriptRunner>) {
    runner.0.step().unwrap();
}

fn reload_scene_script(script_runner: NonSend<ScriptRunner>, chosen_scene: Res<ChosenScene>) {
    if !chosen_scene.is_changed() {
        return;
    }
    script_runner
        .scripts
        .borrow_mut()
        .remove_scene_script()
        .unwrap();
    let scene_name = chosen_scene.list[chosen_scene.index].name.clone();
    script_runner.change_scene(&scene_name).unwrap();
}

fn read_args(asset_server: Res<AssetServer>, mut filesystem: ResMut<FileSystemResource>) {
    for arg in env::args().skip(1) {
        load_filesystem(&asset_server, &mut filesystem, arg);
    }
}

fn load_filesystem(asset_server: &AssetServer, filesystem: &mut FileSystemResource, path: String) {
    #[cfg(not(target_family = "wasm"))]
    {
        let is_dir = !path.bytes().any(|c| c == b'.')
            || (path
                .bytes()
                .rposition(|c| c == b'/')
                .is_some_and(|slash_pos| {
                    path.bytes().rposition(|c| c == b'.').unwrap() < slash_pos
                }));
        if is_dir {
            (*filesystem)
                .write()
                .unwrap()
                .push_layer(Arc::new(RwLock::new(
                    GameDirectory::new(&path).ok_or_error().unwrap(),
                )));
            return;
        }
    }
    let is_main = path.to_uppercase().trim().ends_with(".ISO");
    let handle: Handle<Blob> = asset_server.load(path);
    filesystem.insert_handle(PendingHandle::new(handle, is_main, is_main));
}

fn reload_main_script(
    filesystem: Res<FileSystemResource>,
    script_runner: NonSend<ScriptRunner>,
    mut chosen_scene: ResMut<ChosenScene>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if !filesystem.is_changed() {
        return;
    }
    if !filesystem.is_ready() {
        warn!("Filesystem not ready");
        return;
    }
    let game_paths = Arc::clone(&script_runner.game_paths);
    script_runner.unload_all_scripts();

    //#region Loading application.def
    let root_script_path = script_runner.game_paths.game_definition_filename.clone();
    let root_script_path = ScenePath::new(".", &root_script_path);
    let contents = script_runner
        .filesystem
        .write()
        .unwrap()
        .read_scene_asset(game_paths.clone(), &root_script_path);
    let contents = match contents {
        Ok(v) => v,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            error!("Application definition file not found!");
            return;
        }
        Err(e) => {
            error!("Error accessing application definition file: {}", e);
            return;
        }
    };
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
        script_runner.find_object(|o| matches!(&o.content, CnvContent::Application(_)))
    else {
        panic!("Invalid root script - missing application object"); // TODO: check if == 1, not >= 1
    };
    let application_name = application_object.name.clone();
    let CnvContent::Application(ref application) = &application_object.content else {
        panic!();
    };

    //#region Loading application script
    if let Some(application_script_path) = application.get_script_path() {
        let application_script_path = ScenePath::new(
            &application_script_path,
            &(application_name.clone() + ".cnv"),
        );
        let contents = script_runner
            .filesystem
            .write()
            .unwrap()
            .read_scene_asset(game_paths.clone(), &application_script_path)
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

    let episode_name = application.get_starting_episode().unwrap_or_else(|| {
        panic!(
            "Invalid application object {} - no episodes defined",
            application_name
        )
    });
    let Some(episode_object) = script_runner.get_object(&episode_name) else {
        panic!("Cannot find defined episode object {}", episode_name); // TODO: check if == 1, not >= 1
    };
    let CnvContent::Episode(ref episode) = &episode_object.content else {
        panic!();
    };

    //#region Loading the first episode script
    if let Some(episode_script_path) = episode.get_script_path() {
        let episode_script_path =
            ScenePath::new(&episode_script_path, &(episode_name.clone() + ".cnv"));
        let contents = script_runner
            .filesystem
            .write()
            .unwrap()
            .read_scene_asset(game_paths, &episode_script_path)
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
        script_runner
            .get_object(&scene_name)
            .unwrap_or_else(|| panic!("Cannot find defined scene object {}", scene_name)); // TODO: check if == 1, not >= 1
        chosen_scene.list.push(SceneDefinition { name: scene_name });
    }
    chosen_scene.list.sort();
    let scene_name = episode.get_starting_scene().unwrap_or_else(|| {
        panic!(
            "Invalid episode object {} - no scenes defined",
            episode_name
        )
    });
    chosen_scene.index = chosen_scene
        .list
        .iter()
        .position(|s| s.name == scene_name)
        .unwrap_or_default();
    next_state.set(AppState::SceneViewer);
    trace!(
        "scenes: {:?}",
        chosen_scene
            .list
            .iter()
            .map(|s| s.name.clone())
            .collect::<Vec<_>>()
    );
}

#[derive(Debug, Clone)]
pub struct ScriptRunner(pub Arc<CnvRunner>);

impl Deref for ScriptRunner {
    type Target = Arc<CnvRunner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScriptRunner {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AsRef<Arc<CnvRunner>> for ScriptRunner {
    fn as_ref(&self) -> &Arc<CnvRunner> {
        &self.0
    }
}
