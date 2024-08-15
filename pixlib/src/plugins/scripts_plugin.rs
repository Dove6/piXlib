use std::{
    cell::RefCell,
    env,
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::{Arc, RwLock},
};

use bevy::{
    app::{App, Plugin, Update},
    log::info,
    prelude::{in_state, DetectChanges, IntoSystemConfigs, NonSend, Res, ResMut},
};
use pixlib_parser::{
    common::{DroppableRefMut, IssueManager},
    runner::{
        classes::{Application, Episode},
        CnvContent, CnvRunner, FileSystem, GamePaths, RunnerIssue, ScenePath, ScriptSource,
    },
    scanner::parse_cnv,
};

use crate::{
    filesystems::{CompressedPatch, InsertedDiskResource, LayeredFileSystem},
    resources::{ChosenScene, SceneDefinition},
    util::IssuePrinter,
    AppState,
};

#[derive(Debug)]
pub struct ScriptsPlugin {
    pub inserted_disk: Arc<RwLock<dyn FileSystem>>,
}

#[allow(clippy::arc_with_non_send_sync)]
impl Plugin for ScriptsPlugin {
    fn build(&self, app: &mut App) {
        let mut runner_issue_manager: IssueManager<RunnerIssue> = Default::default();
        runner_issue_manager.set_handler(Box::new(IssuePrinter));
        let layered_fs = Arc::new(RefCell::new(LayeredFileSystem::new(Arc::clone(
            &self.inserted_disk,
        ))));
        layered_fs.borrow_mut().use_and_drop_mut(|l| {
            for arg in env::args().skip(2) {
                l.push_layer(Arc::new(RwLock::new(
                    CompressedPatch::try_from(PathBuf::from(&arg).as_ref()).unwrap(),
                )));
            }
        });
        app.insert_non_send_resource(ScriptRunner(
            CnvRunner::try_new(
                layered_fs,
                Arc::new(GamePaths::default()),
                runner_issue_manager,
            )
            .unwrap(),
        ))
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

fn reload_main_script(
    inserted_disk: Res<InsertedDiskResource>,
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
        .read_scene_asset(game_paths.clone(), &root_script_path)
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
    info!(
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
