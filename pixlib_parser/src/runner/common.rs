use std::sync::Arc;

use pixlib_formats::file_formats::ann::LoopingSettings;

#[derive(Debug, Clone)]
pub enum CallableIdentifier<'a> {
    Method(&'a str),
    Event(&'a str),
}

impl<'a> CallableIdentifier<'a> {
    pub fn to_owned(&self) -> CallableIdentifierOwned {
        match *self {
            CallableIdentifier::Method(m) => CallableIdentifierOwned::Method(m.to_owned()),
            CallableIdentifier::Event(e) => CallableIdentifierOwned::Event(e.to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CallableIdentifierOwned {
    Method(String),
    Event(String),
}

impl<'a> From<&'a CallableIdentifierOwned> for CallableIdentifier<'a> {
    fn from(value: &'a CallableIdentifierOwned) -> Self {
        match value {
            CallableIdentifierOwned::Method(m) => CallableIdentifier::Method(m.as_ref()),
            CallableIdentifierOwned::Event(e) => CallableIdentifier::Event(e.as_ref()),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum SoundFileData {
    #[default]
    Empty,
    NotLoaded(String),
    Loaded(LoadedSound),
}

#[derive(Debug, Clone)]
pub struct LoadedSound {
    pub filename: Option<String>,
    pub sound: SoundData,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoundData {
    pub hash: u64,
    pub data: Arc<[u8]>, // RGBA8888
}

#[derive(Debug, Clone, Default)]
pub enum SequenceFileData {
    #[default]
    Empty,
    NotLoaded(String),
    Loaded(LoadedSequence),
}

#[derive(Debug, Clone)]
pub struct LoadedSequence {
    pub filename: Option<String>,
    pub sequence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageDefinition {
    pub size_px: (u32, u32),
    pub offset_px: (i32, i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageData {
    pub hash: u64,
    pub data: Arc<[u8]>, // RGBA8888
}

#[derive(Debug, Clone)]
pub struct LoadedImage {
    pub filename: Option<String>,
    pub image: (ImageDefinition, ImageData),
}

#[derive(Debug, Clone, Default)]
pub enum ImageFileData {
    #[default]
    Empty,
    NotLoaded(String),
    Loaded(LoadedImage),
}

#[derive(Clone, Debug)]
pub struct SequenceDefinition {
    pub name: String,
    pub opacity: u8,
    pub looping: LoopingSettings,
    pub frames: Vec<FrameDefinition>,
}

#[derive(Clone, Debug)]
pub struct FrameDefinition {
    pub name: String,
    pub offset_px: (i32, i32),
    pub opacity: u8,
    pub sprite_idx: usize,
    pub sfx: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SpriteDefinition {
    pub name: String,
    pub size_px: (u32, u32),
    pub offset_px: (i32, i32),
}

#[derive(Clone, Debug)]
pub struct SpriteData {
    pub hash: u64,
    pub data: Arc<[u8]>, // RGBA8888
}

impl PartialEq for SpriteData {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash
    }
}

#[derive(Debug, Clone, Default)]
pub enum AnimationFileData {
    #[default]
    Empty,
    NotLoaded(String),
    Loaded(LoadedAnimation),
}

#[derive(Debug, Clone)]
pub struct LoadedAnimation {
    pub filename: Option<String>,
    pub sequences: Vec<SequenceDefinition>,
    pub sprites: Vec<(SpriteDefinition, SpriteData)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Copy)]
pub struct FrameIdentifier {
    pub sequence_idx: usize,
    pub frame_idx: usize,
}
