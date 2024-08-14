use parsers::TypeParsingError;
use pixlib_formats::file_formats::ann::LoopingSettings;
use std::{any::Any, collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;

use crate::runner::RunnerResult;
use crate::runner::{CnvStatement, CnvValue, RunnerContext};

pub trait CnvType: std::fmt::Debug {
    fn get_type_id(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn call_method(
        &self,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>>;

    fn new_content(
        parent: Arc<CnvObject>,
        properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError>
    where
        Self: Sized;
}

impl dyn CnvType {}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct DummyCnvType {}

impl CnvType for DummyCnvType {
    fn get_type_id(&self) -> &'static str {
        "DUMMY"
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        todo!("{:?} {:?}", self.get_type_id(), name)
    }

    fn new_content(
        _parent: Arc<CnvObject>,
        _properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError>
    where
        Self: Sized,
    {
        Ok(CnvContent::None(Self {}))
    }
}

pub struct CnvTypeFactory;

impl CnvTypeFactory {
    pub fn create(
        parent: Arc<CnvObject>,
        type_name: String,
        properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError> {
        match type_name.as_ref() {
            "ANIMO" => Animation::new_content(parent, properties),
            "APPLICATION" => Application::new_content(parent, properties),
            "ARRAY" => Array::new_content(parent, properties),
            "BEHAVIOUR" => Behavior::new_content(parent, properties),
            "BOOL" => BoolVar::new_content(parent, properties),
            "BUTTON" => Button::new_content(parent, properties),
            "CANVAS_OBSERVER" => CanvasObserver::new_content(parent, properties),
            "CANVASOBSERVER" => CanvasObserver::new_content(parent, properties),
            "CNVLOADER" => CnvLoader::new_content(parent, properties),
            "CONDITION" => Condition::new_content(parent, properties),
            "COMPLEXCONDITION" => ComplexCondition::new_content(parent, properties),
            "DOUBLE" => DoubleVar::new_content(parent, properties),
            "EPISODE" => Episode::new_content(parent, properties),
            "EXPRESSION" => Expression::new_content(parent, properties),
            "FONT" => Font::new_content(parent, properties),
            "GROUP" => Group::new_content(parent, properties),
            "IMAGE" => Image::new_content(parent, properties),
            "INTEGER" => IntegerVar::new_content(parent, properties),
            "KEYBOARD" => Keyboard::new_content(parent, properties),
            "MOUSE" => Mouse::new_content(parent, properties),
            "MULTIARRAY" => MultiArray::new_content(parent, properties),
            "MUSIC" => Music::new_content(parent, properties),
            "RAND" => Rand::new_content(parent, properties),
            "SCENE" => Scene::new_content(parent, properties),
            "SEQUENCE" => Sequence::new_content(parent, properties),
            "SOUND" => Sound::new_content(parent, properties),
            "STRING" => StringVar::new_content(parent, properties),
            "STRUCT" => Struct::new_content(parent, properties),
            "SYSTEM" => System::new_content(parent, properties),
            "TEXT" => Text::new_content(parent, properties),
            "TIMER" => Timer::new_content(parent, properties),
            _ => Err(TypeParsingError::UnknownType(type_name)),
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SequenceDefinition {
    pub name: String,
    pub opacity: u8,
    pub looping: LoopingSettings,
    pub frames: Vec<FrameDefinition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrameDefinition {
    pub name: String,
    pub offset_px: (i32, i32),
    pub opacity: u8,
    pub sprite_idx: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteDefinition {
    pub name: String,
    pub size_px: (u32, u32),
    pub offset_px: (i32, i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteData {
    pub hash: u64,
    pub data: Arc<[u8]>, // RGBA8888
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

pub type EpisodeName = String;
pub type SceneName = String;
pub type ConditionName = String;
pub type ImageName = String;
pub type SoundName = String;
pub type VariableName = String;
pub type TypeName = String;
pub type FontName = String;

mod animation;
mod application;
mod array;
mod behavior;
mod bool;
mod button;
mod canvasobserver;
mod cnvloader;
mod complexcondition;
mod condition;
mod content;
mod dbl;
mod episode;
mod expression;
mod font;
mod group;
mod image;
mod initable;
mod int;
mod keyboard;
mod mouse;
mod multiarray;
mod music;
mod object;
mod parsers;
mod random;
mod scene;
mod sequence;
mod sound;
mod str;
mod structure;
mod system;
mod text;
mod timer;

pub use animation::Animation;
pub use application::Application;
pub use array::Array;
pub use behavior::Behavior;
pub use bool::BoolVar;
pub use button::Button;
pub use canvasobserver::CanvasObserver;
pub use cnvloader::CnvLoader;
pub use complexcondition::ComplexCondition;
pub use condition::Condition;
pub use content::CnvContent;
pub use dbl::DoubleVar;
pub use episode::Episode;
pub use expression::Expression;
pub use font::Font;
pub use group::Group;
pub use image::Image;
pub use int::IntegerVar;
pub use keyboard::Keyboard;
pub use lalrpop_util::ParseError;
pub use mouse::{InternalMouseEvent, Mouse};
pub use multiarray::MultiArray;
pub use music::Music;
pub use object::CnvObject;
pub use object::CnvObjectBuilder;
pub use object::ObjectBuilderError;
pub use parsers::PropertyValue; // poison
pub use random::Rand;
pub use scene::Scene;
pub use sequence::Sequence;
pub use sound::Sound;
pub use str::StringVar;
pub use structure::Struct;
pub use system::System;
pub use text::Text;
pub use timer::Timer;
