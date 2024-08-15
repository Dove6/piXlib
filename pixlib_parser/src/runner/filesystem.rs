use std::{io::ErrorKind, sync::Arc};

use super::path::{Path, ScenePath};

pub trait FileSystem: std::fmt::Debug + Send + Sync {
    fn read_file(&mut self, filename: &str) -> std::io::Result<Vec<u8>>;
    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()>;
}

impl dyn FileSystem {
    pub fn read_scene_file(
        &mut self,
        game_paths: Arc<GamePaths>,
        scene_path: &ScenePath,
    ) -> std::io::Result<Vec<u8>> {
        println!(
            "read_scene_file({:?}, {:?})",
            game_paths.data_directory, scene_path,
        );
        let mut path = scene_path.file_path.clone();
        println!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        path.prepend(&scene_path.dir_path);
        println!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        path.prepend(&game_paths.data_directory);
        println!("Trying path: {:?}", path);
        match self.read_file(&path) {
            Ok(vec) => return Ok(vec),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        Err(std::io::Error::from(std::io::ErrorKind::NotFound))
    }
}

#[derive(Debug)]
pub struct DummyFileSystem;

impl FileSystem for DummyFileSystem {
    fn read_file(&mut self, _: &str) -> std::io::Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn write_file(&mut self, _: &str, _: &[u8]) -> std::io::Result<()> {
        Ok(())
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
