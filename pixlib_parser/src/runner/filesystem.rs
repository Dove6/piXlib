use std::{io::ErrorKind, sync::Arc};

use log::{info, trace};

use super::path::{Path, ScenePath};

pub trait FileSystem: std::fmt::Debug + Send + Sync {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Arc<Vec<u8>>>;
    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()>;
}

impl dyn FileSystem {
    pub fn read_scene_asset(
        &mut self,
        game_paths: Arc<GamePaths>,
        scene_path: &ScenePath,
    ) -> std::io::Result<Arc<Vec<u8>>> {
        info!(
            "read_scene_file({:?}, {:?})",
            game_paths.data_directory, scene_path,
        );
        let mut path = scene_path.file_path.clone();
        trace!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        path.prepend(&scene_path.dir_path);
        trace!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        path.prepend(&game_paths.data_directory);
        trace!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    pub fn read_sound(
        &mut self,
        game_paths: Arc<GamePaths>,
        scene_path: &ScenePath,
    ) -> std::io::Result<Arc<Vec<u8>>> {
        info!(
            "read_sound_file(({:?}, {:?}), {:?})",
            game_paths.dialogues_directory, game_paths.data_directory, scene_path,
        );
        let mut path = scene_path.file_path.clone();
        trace!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        path.prepend(&game_paths.dialogues_directory);
        trace!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let mut path = scene_path.file_path.with_prepended(&scene_path.dir_path);
        trace!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        path.prepend(&game_paths.data_directory);
        trace!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }

    pub fn write_scene_asset(
        &mut self,
        game_paths: Arc<GamePaths>,
        scene_path: &ScenePath,
        data: &[u8],
    ) -> std::io::Result<()> {
        info!(
            "write_scene_file({:?}, {:?})",
            game_paths.data_directory, scene_path,
        );
        let mut path = scene_path.flatten();
        trace!("Flattened path: {}", path.as_ref());
        path.prepend(&game_paths.data_directory);
        trace!("Saving at path: {:?}", path);
        self.write_file(&path, data)
    }
}

#[derive(Debug, Clone)]
pub struct GamePaths {
    pub data_directory: Path,
    pub game_definition_filename: Path,
    pub music_directory: Path,
    pub dialogues_directory: Path,
    pub sfx_directory: Path,
    pub common_directory: Path,
    pub classes_directory: Path,
}

impl Default for GamePaths {
    fn default() -> Self {
        Self {
            data_directory: "./DANE/".into(),
            game_definition_filename: "./APPLICATION.DEF".into(),
            music_directory: "./".into(),
            dialogues_directory: "./WAVS/".into(),
            sfx_directory: "./WAVS/SFX/".into(),
            common_directory: "./COMMON/".into(),
            classes_directory: "./COMMON/CLASSES/".into(),
        }
    }
}
