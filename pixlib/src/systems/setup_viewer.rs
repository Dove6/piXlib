use crate::animation::{AnimationBundle, CnvIdentifier};
use crate::iso::read_script;
use crate::resources::{
    ChosenScene, GamePaths, InsertedDisk, ObjectBuilderIssueManager, RootEntityToDespawn,
    SceneDefinition, ScriptRunner,
};
use bevy::hierarchy::BuildChildren;
use bevy::log::{error, info};
use bevy::prelude::{NonSendMut, SpatialBundle};
use bevy::sprite::SpriteBundle;
use bevy::{
    ecs::system::Res,
    prelude::{Commands, ResMut},
};
use pixlib_parser::classes::{CallableIdentifier, CnvObject, PropertyValue};
use pixlib_parser::runner::{CnvStatement, RunnerContext, ScriptSource};

use std::path::Path;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct OrderedGraphics {
    pub identifier: Option<String>,
    pub file_path: String,
    pub script_index: usize,
    pub object_index: usize,
    pub priority: i32,
}

pub fn setup_viewer(
    game_paths: Res<GamePaths>,
    inserted_disk: Res<InsertedDisk>,
    chosen_scene: Res<ChosenScene>,
    mut commands: Commands,
    mut script_runner: NonSendMut<ScriptRunner>,
    mut issue_manager: ResMut<ObjectBuilderIssueManager>,
) {
    let Some(iso) = inserted_disk.get() else {
        panic!("No disk inserted!");
    };
    let ChosenScene { list, index } = chosen_scene.as_ref();
    let Some(SceneDefinition { name, path, .. }) = list.get(*index) else {
        info!(
            "Could not load scene script: bad index {} for scene list {:?}",
            index, list
        );
        return;
    };
    let scene_script_path = read_script(
        iso,
        path.as_os_str().to_str().unwrap(),
        name,
        &game_paths,
        script_runner
            .as_ref()
            .borrow()
            .get_root_script()
            .map(|s| s.as_ref().borrow().path.clone()),
        ScriptSource::Scene,
        &script_runner,
        &mut issue_manager,
    );
    let Some(scene_object) = script_runner.as_ref().borrow().get_object(name) else {
        panic!(
            "Could not find scene object {}: {:?}",
            &name,
            script_runner.as_ref().borrow().get_object(name)
        );
    };
    if scene_object
        .content
        .borrow()
        .as_ref()
        .unwrap()
        .as_ref()
        .get_type_id()
        != "SCENE"
    {
        panic!(
            "Could not find scene object {}: {:?}",
            &name,
            script_runner.as_ref().borrow().get_object(name)
        );
    };
    let Some(PropertyValue::String(scene_path)) = scene_object
        .content
        .borrow()
        .as_ref()
        .unwrap()
        .get_property("PATH")
    else {
        error!("Scene {} has no path", &name);
        return;
    };
    let scene_script = script_runner
        .as_ref()
        .borrow()
        .get_script(&scene_script_path)
        .unwrap();

    let root_entity = commands
        .spawn(SpatialBundle::default())
        .with_children(|parent| {
            let mut initial_images = vec![];
            if let Some(PropertyValue::String(background_filename)) = scene_object
                .content
                .borrow()
                .as_ref()
                .unwrap()
                .get_property("BACKGROUND")
            {
                initial_images.push(OrderedGraphics {
                    identifier: None,
                    file_path: get_path_to_scene_file(
                        &game_paths,
                        &scene_path,
                        &background_filename,
                    )
                    .to_str()
                    .unwrap()
                    .to_owned(),
                    script_index: 0,
                    object_index: 0,
                    priority: 0,
                });
            }
            for OrderedGraphics {
                identifier,
                file_path,
                script_index,
                object_index,
                priority,
            } in initial_images.into_iter().chain(
                scene_script
                    .as_ref()
                    .borrow()
                    .objects
                    .iter()
                    .filter(|cnv_object| {
                        matches!(
                            cnv_object
                                .content
                                .borrow()
                                .as_ref()
                                .unwrap()
                                .get_type_id(),
                            "IMAGE" | "ANIMO"
                        )
                    })
                    .map(|cnv_object| {
                        let content_guard = cnv_object.content.borrow();
                        let content = content_guard.as_ref().unwrap();
                        let (filename, priority) = (
                            content
                                .get_property("FILENAME")
                                .and_then(|v| match v {
                                    PropertyValue::String(s) => Some(s),
                                    _ => None,
                                })
                                .unwrap(),
                            content
                                .get_property("PRIORITY")
                                .and_then(|v| match v {
                                    PropertyValue::Integer(i) => Some(i),
                                    _ => None,
                                })
                                .unwrap_or_default(),
                        );
                        OrderedGraphics {
                            identifier: Some(cnv_object.name.clone()),
                            file_path: get_path_to_scene_file(&game_paths, &scene_path, &filename)
                                .to_str()
                                .unwrap()
                                .to_owned(),
                            script_index: 1,
                            object_index: cnv_object.index,
                            priority,
                        }
                    }),
            ) {
                let z_position =
                    priority as f32 + (script_index * 1000 + object_index) as f32 / 100000f32;
                match file_path
                    .to_lowercase()
                    .chars()
                    .rev()
                    .take(3)
                    .collect::<String>()
                    .as_str()
                {
                    "gmi" => {
                        parent.spawn((CnvIdentifier(identifier), SpriteBundle::default()));
                    }
                    "nna" => {
                        parent.spawn((CnvIdentifier(identifier), AnimationBundle::default()));
                    }
                    _ => panic!(),
                }
            }
        })
        .id();
    commands.insert_resource(RootEntityToDespawn(Some(root_entity)));

    if let Some(init_beh_obj) = scene_script.as_ref().borrow().get_object("__INIT__") {
        if init_beh_obj
            .content
            .borrow()
            .as_ref()
            .unwrap()
            .get_type_id()
            != "BEHAVIOUR"
        {
            error!(
                "Expected __INIT__ object to be a behavior, not: {:?}",
                &init_beh_obj
                    .content
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .get_type_id()
            );
            return;
        }
        info!("Running __INIT__ behavior...");
        let mut context = RunnerContext {
            runner: &mut *script_runner.as_mut().borrow_mut(),
            self_object: init_beh_obj.name.clone(),
            current_object: init_beh_obj.name.clone(),
        };
        init_beh_obj.call_method(CallableIdentifier::Method("RUN"), &Vec::new(), &mut context);
    }

    let scene_script = script_runner
        .as_ref()
        .borrow()
        .get_script(&scene_script_path)
        .unwrap();
    info!(
        "Scene objects: {:#?}",
        scene_script
            .as_ref()
            .borrow()
            .objects
            .iter()
            .map(|o| o.name.clone())
            .collect::<Vec<_>>()
    );
    let mut initable_objects: Vec<Arc<CnvObject>> = Vec::new();
    scene_script.as_ref().borrow().find_objects(
        |o| {
            o.content
                .borrow()
                .as_ref()
                .unwrap()
                .has_event("ONINIT")
        },
        &mut initable_objects,
    );
    info!(
        "Found initable objects: {:?}",
        initable_objects
            .iter()
            .map(|o| o.name.clone())
            .collect::<Vec<_>>()
    );
    for object in initable_objects {
        let mut context = RunnerContext {
            runner: &mut *script_runner.as_mut().borrow_mut(),
            self_object: object.name.clone(),
            current_object: object.name.clone(),
        };
        if let Some(PropertyValue::Code(handler)) = object.get_property("ONINIT") {
            println!("Processing initable object: {:?}", object.name);
            handler.run(&mut context)
        }
    }
}

fn get_path_to_scene_file(game_paths: &GamePaths, scene_path: &str, filename: &str) -> Arc<Path> {
    let mut path = game_paths.data_directory.join(scene_path).join(filename);
    info!("PATHS: {:?}, {:?}, {:?}", &scene_path, &filename, &path);
    path.as_mut_os_string().make_ascii_uppercase();
    info!(
        "get_path_to_scene_file: {}, {}, {}",
        game_paths.data_directory.to_str().unwrap_or_default(),
        scene_path,
        filename
    );
    path.into()
}
