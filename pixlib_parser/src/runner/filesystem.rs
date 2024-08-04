use std::{
    io::ErrorKind,
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
        eprintln!(
            "read_scene_file({:?}, {:?}, {:?}, {:?})",
            game_paths.data_directory, scene_dir, filename, extension,
        );
        let mut path = PathBuf::from(filename);
        eprintln!("Trying path: {:?}", path);
        match self.read_file(path.to_str().unwrap()) {
            Ok(vec) => return Ok((vec, path.into())),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(_) => return Err(()),
        }
        if let Some(extension) = extension {
            let path = path.with_extension(extension);
            eprintln!("Trying path: {:?}", path);
            match self.read_file(path.to_str().unwrap()) {
                Ok(vec) => return Ok((vec, path.into())),
                Err(e) if e.kind() == ErrorKind::NotFound => {}
                Err(_) => return Err(()),
            }
        }
        if let Some(scene_dir) = scene_dir {
            path = PathBuf::from(scene_dir).join(path);
            eprintln!("Trying path: {:?}", path);
            match self.read_file(path.to_str().unwrap()) {
                Ok(vec) => return Ok((vec, path.into())),
                Err(e) if e.kind() == ErrorKind::NotFound => {}
                Err(_) => return Err(()),
            }
            if let Some(extension) = extension {
                let path = path.with_extension(extension);
                eprintln!("Trying path: {:?}", path);
                match self.read_file(path.to_str().unwrap()) {
                    Ok(vec) => return Ok((vec, path.into())),
                    Err(e) if e.kind() == ErrorKind::NotFound => {}
                    Err(_) => return Err(()),
                }
            }
        }
        path = game_paths.data_directory.join(path);
        eprintln!("Trying path: {:?}", path);
        match self.read_file(path.to_str().unwrap()) {
            Ok(vec) => return Ok((vec, path.into())),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(_) => return Err(()),
        }
        if let Some(extension) = extension {
            let path = path.with_extension(extension);
            eprintln!("Trying path: {:?}", path);
            match self.read_file(path.to_str().unwrap()) {
                Ok(vec) => return Ok((vec, path.into())),
                Err(e) if e.kind() == ErrorKind::NotFound => {}
                Err(_) => return Err(()),
            }
        }
        Err(())
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
