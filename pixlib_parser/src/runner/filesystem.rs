use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub trait FileSystem: std::fmt::Debug + Send + Sync {
    fn read_file(&self, filename: &str) -> std::io::Result<Vec<u8>>;
    fn write_file(&mut self, filename: &str, data: &[u8]) -> std::io::Result<()>;
}

impl dyn FileSystem {
    pub fn read_scene_file(
        &self,
        game_paths: Arc<GamePaths>,
        scene_dir: Option<&str>,
        filename: &str,
        extension: Option<&str>,
    ) -> Result<(Vec<u8>, Arc<Path>), ()> {
        let path = build_data_path(scene_dir.unwrap_or("./"), filename, &game_paths, extension);
        let read_file = self.read_file(path.to_str().unwrap()).map_err(|_| ())?;
        Ok((read_file, path))
    }
}

#[derive(Debug)]
pub struct DummyFileSystem;

impl FileSystem for DummyFileSystem {
    fn read_file(&self, _: &str) -> std::io::Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn write_file(&mut self, _: &str, _: &[u8]) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct GamePaths {
    pub data_directory: Arc<Path>,
    pub game_definition_filename: Arc<Path>,
    pub music_directory: Arc<Path>,
    pub dialogues_directory: Arc<Path>,
    pub sfx_directory: Arc<Path>,
    pub common_directory: Arc<Path>,
    pub classes_directory: Arc<Path>,
}

impl Default for GamePaths {
    fn default() -> Self {
        Self {
            data_directory: PathBuf::from("./DANE/").into(),
            game_definition_filename: PathBuf::from("./APPLICATION.DEF").into(),
            music_directory: PathBuf::from("./").into(),
            dialogues_directory: PathBuf::from("./WAVS/").into(),
            sfx_directory: PathBuf::from("./WAVS/SFX/").into(),
            common_directory: PathBuf::from("./COMMON/").into(),
            classes_directory: PathBuf::from("./COMMON/CLASSES/").into(),
        }
    }
}

impl GamePaths {
    pub fn get_game_definition_path(&self) -> Arc<Path> {
        self.data_directory
            .join(&self.game_definition_filename)
            .into()
    }
}

pub fn build_data_path(
    path: &str,
    filename: &str,
    game_paths: &GamePaths,
    extension: Option<&str>,
) -> Arc<Path> {
    let mut script_path = game_paths
        .data_directory
        .join(path.replace("\\", "/"))
        .join(filename.to_owned() + extension.unwrap_or(""));
    script_path.as_mut_os_string().make_ascii_uppercase();
    eprintln!(
        "build_data_path({}, {}, {:?}, {:?}) -> {:?}",
        path, filename, game_paths.data_directory, extension, script_path
    );
    script_path.into()
}
