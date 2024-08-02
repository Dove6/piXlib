#![allow(clippy::too_many_arguments, clippy::type_complexity)]

pub mod anchors;
pub mod arguments;
pub mod components;
pub mod graphics_plugin;
pub mod image;
pub mod iso;
pub mod resources;
pub mod states;
pub mod systems;
pub mod util;

use std::{cell::RefCell, env, path::PathBuf, sync::Arc};

use bevy::{
    ecs::{
        change_detection::DetectChanges,
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter, OnExit},
        system::{Res, ResMut},
    },
    log::{error, warn},
    prelude::{default, App, NonSendMut, PluginGroup, Startup, Update},
    render::texture::ImagePlugin,
    window::{PresentMode, Window, WindowPlugin},
    winit::WinitSettings,
    DefaultPlugins,
};
use graphics_plugin::GraphicsPlugin;
use iso::{read_game_definition, read_script};
use pixlib_parser::{
    classes::{ObjectBuilderError, PropertyValue},
    common::{Issue, IssueHandler, IssueKind, IssueManager},
    runner::{CnvRunner, ScriptSource},
};
use resources::{
    ChosenScene, DebugSettings, GamePaths, InsertedDisk, ObjectBuilderIssueManager,
    SceneDefinition, ScriptRunner, WindowConfiguration,
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
        .insert_resource(GamePaths {
            data_directory: "./DANE/".into(),
            game_definition_filename: "./APPLICATION.DEF".into(),
            music_directory: "./".into(),
            dialogues_directory: "./WAVS/".into(),
            sfx_directory: "./WAVS/SFX/".into(),
            common_directory: "./COMMON/".into(),
            classes_directory: "./COMMON/CLASSES/".into(),
        })
        .insert_resource(InsertedDisk::try_from(env::args()).expect("Usage: pixlib path_to_iso"))
        .insert_resource(ChosenScene::default())
        .insert_non_send_resource(ScriptRunner(Arc::new(RefCell::new(CnvRunner {
            scripts: Default::default(),
            filesystem: Arc::new(RefCell::new(InsertedDisk::try_from(env::args()).unwrap())),
            current_scene: None,
        }))))
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
            (detect_return_to_chooser).run_if(in_state(AppState::SceneViewer)),
        )
        .add_systems(OnExit(AppState::SceneViewer), cleanup_root)
        .add_plugins(GraphicsPlugin)
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

fn reload_main_script(
    inserted_disk: Res<InsertedDisk>,
    game_paths: Res<GamePaths>,
    mut script_runner: NonSendMut<ScriptRunner>,
    mut chosen_scene: ResMut<ChosenScene>,
    mut issue_manager: ResMut<ObjectBuilderIssueManager>,
) {
    if !inserted_disk.is_changed() {
        return;
    }
    script_runner.borrow_mut().unload_all_scripts();
    let Some(iso) = inserted_disk.get() else {
        return;
    };
    let root_script_path =
        read_game_definition(iso, &game_paths, &mut script_runner, &mut issue_manager);
    let mut vec = Vec::new();
    script_runner.0.borrow().find_objects(
        |o| {
            matches!(
                o.content.borrow().as_ref().unwrap().get_type_id(),
                "APPLICATION"
            )
        },
        &mut vec,
    );
    if vec.len() != 1 {
        error!(
            "Incorrect number of APPLICATION objects (should be 1): {:?}",
            vec.iter().map(|o| o.name.clone())
        );
        return;
    }
    let application = vec.into_iter().next().unwrap();
    let application_name = application.name.clone();
    if let Some(PropertyValue::String(ref application_path)) = application
        .content
        .borrow()
        .as_ref()
        .unwrap()
        .get_property("PATH")
    {
        read_script(
            iso,
            application_path,
            &application_name,
            &game_paths,
            Some(Arc::clone(&root_script_path)),
            ScriptSource::Application,
            &mut script_runner,
            &mut issue_manager,
        );
    }
    let Some(PropertyValue::List(episode_list)) = application
        .content
        .borrow()
        .as_ref()
        .unwrap()
        .get_property("EPISODES")
    else {
        panic!();
    };
    let episode_object_name = if episode_list.len() == 1 {
        episode_list.into_iter().next().unwrap()
    } else {
        error!(
            "Unexpected number of episodes (expected 1): {:?}",
            episode_list
        );
        return;
    };
    let episode_object_name = episode_object_name.clone();
    if let Some(episode_object) = script_runner.borrow().get_object(&episode_object_name) {
        let episode_name = episode_object.name.clone();
        if let Some(PropertyValue::String(ref episode_path)) = episode_object
            .content
            .borrow()
            .as_ref()
            .unwrap()
            .get_property("PATH")
        {
            read_script(
                iso,
                episode_path,
                &episode_name,
                &game_paths,
                Some(Arc::clone(&root_script_path)),
                ScriptSource::Episode,
                &script_runner,
                &mut issue_manager,
            );
        }
        chosen_scene.list.clear();
        chosen_scene.index = 0;
        let Some(PropertyValue::List(scene_list)) = episode_object
            .content
            .borrow()
            .as_ref()
            .unwrap()
            .get_property("SCENES")
        else {
            panic!();
        };
        for scene_name in scene_list.iter() {
            if let Some(scene_object) = script_runner.borrow().get_object(scene_name) {
                if scene_object
                    .content
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .get_type_id()
                    != "SCENE"
                {
                    panic!();
                };
                let Some(PropertyValue::String(scene_script_path)) = scene_object
                    .content
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .get_property("PATH")
                else {
                    error!("Scene {} has no path", scene_name);
                    continue;
                };
                let scene_defintion = SceneDefinition {
                    name: scene_name.to_string(),
                    path: PathBuf::from(scene_script_path),
                    background: scene_object
                        .content
                        .borrow()
                        .as_ref()
                        .unwrap()
                        .get_property("BACKGROUND")
                        .and_then(|v| match v {
                            PropertyValue::String(s) => Some(s),
                            _ => None,
                        }),
                };
                chosen_scene.list.push(scene_defintion);
            }
        }
        chosen_scene.list.sort();
    } else {
        error!(
            "Could not find episode object with name: {}",
            episode_object_name
        );
    };
}

/*
fn reload_scene_script(
    chosen_scene: Res<ChosenScene>,
    inserted_disk: Res<InsertedDisk>,
    game_paths: Res<GamePaths>,
    mut script_runner: ResMut<ScriptRunner>,
) {
    if !chosen_scene.is_changed() {
        return;
    }
    let Some(iso) = &inserted_disk.get() else {
        return;
    };
    let mut vec = Vec::new();
    script_runner.find_scripts(
        |s| matches!(s.source_kind, ScriptSource::Scene | ScriptSource::CnvLoader),
        &mut vec,
    );
    for episode_script in vec.iter() {
        script_runner.0.unload_script(episode_script);
    }
    let ChosenScene { list, index } = chosen_scene.as_ref();
    let Some(SceneDefinition {
        name,
        path,
        ..
    }) = list.get(*index)
    else {
        println!(
            "Could not load scene script: bad index {} for scene list {:?}",
            index, list
        );
        return;
    };
    read_script(
        iso,
        &path.as_os_str().to_str().unwrap(),
        &name,
        &game_paths,
        script_runner.get_root_script().map(|s| Arc::clone(&s.path)),
        ScriptSource::Scene,
        &mut script_runner,
    );
}
*/
