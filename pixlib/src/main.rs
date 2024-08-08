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

use std::{cell::RefCell, env, sync::Arc};

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
    common::{Issue, IssueHandler, IssueKind, IssueManager},
    runner::{CnvRunner, GamePaths, RunnerIssue, ScriptSource},
    scanner::parse_cnv,
};
use resources::{
    ChosenScene, DebugSettings, InsertedDisk, ObjectBuilderIssueManager, SceneDefinition,
    ScriptRunner, WindowConfiguration,
};
use states::AppState;
use systems::{
    cleanup_root, detect_return_to_chooser, draw_cursor, handle_dropped_iso, navigate_chooser,
    setup, setup_chooser, update_chooser_labels,
};

const WINDOW_SIZE: (usize, usize) = (800, 600);
const WINDOW_TITLE: &str = "piXlib";

fn main() {
    let mut issue_manager: IssueManager<ObjectBuilderError> = Default::default();
    issue_manager.set_handler(Box::new(IssuePrinter));
    let mut runner_issue_manager: IssueManager<RunnerIssue> = Default::default();
    runner_issue_manager.set_handler(Box::new(IssuePrinter));
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
        .insert_resource(InsertedDisk::try_from(env::args()).expect("Usage: pixlib path_to_iso"))
        .insert_resource(ChosenScene::default())
        .insert_non_send_resource(ScriptRunner(CnvRunner::new(
            Arc::new(RefCell::new(InsertedDisk::try_from(env::args()).unwrap())),
            Arc::new(GamePaths::default()),
            runner_issue_manager,
        )))
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
    let (contents, path) = script_runner
        .filesystem
        .borrow()
        .read_scene_file(game_paths, Some(&path), &scene_name, Some("CNV"))
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
    inserted_disk: Res<InsertedDisk>,
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
    let (contents, root_script_path) = script_runner
        .filesystem
        .borrow()
        .read_scene_file(
            game_paths.clone(),
            None,
            &root_script_path.to_str().unwrap(),
            None,
        )
        .unwrap();
    let contents = parse_cnv(&contents);
    script_runner
        .0
        .load_script(
            root_script_path.clone(),
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
        let (contents, application_script_path) = script_runner
            .filesystem
            .borrow()
            .read_scene_file(
                game_paths.clone(),
                Some(&application_script_path),
                &application_name,
                Some("CNV"),
            )
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
        let (contents, episode_script_path) = script_runner
            .filesystem
            .borrow()
            .read_scene_file(
                game_paths,
                Some(&episode_script_path),
                &episode_name,
                Some("CNV"),
            )
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
