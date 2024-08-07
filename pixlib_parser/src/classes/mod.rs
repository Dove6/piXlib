use parsers::TypeParsingError;
use pixlib_formats::file_formats::ann::LoopingSettings;
use std::{any::Any, collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;

use crate::runner::RunnerResult;
use crate::{
    ast::IgnorableProgram,
    runner::{CnvStatement, CnvValue, RunnerContext},
};

pub trait CnvType: std::fmt::Debug {
    fn get_type_id(&self) -> &'static str;
    fn has_event(&self, name: &str) -> bool;
    fn has_property(&self, name: &str) -> bool;
    fn has_method(&self, name: &str) -> bool;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn get_property(&self, name: &str) -> Option<PropertyValue>;
    fn call_method(
        &self,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>>;

    fn new(
        parent: Arc<CnvObject>,
        properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError>
    where
        Self: Sized;
}

impl dyn CnvType {}

#[derive(Debug)]
pub enum CallableIdentifier<'a> {
    Method(&'a str),
    Event(&'a str),
}

#[derive(Debug)]
pub struct DummyCnvType {}

impl CnvType for DummyCnvType {
    fn get_type_id(&self) -> &'static str {
        "DUMMY"
    }

    fn has_event(&self, _name: &str) -> bool {
        false
    }

    fn has_property(&self, _name: &str) -> bool {
        false
    }

    fn has_method(&self, _name: &str) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }

    fn get_property(&self, _name: &str) -> Option<PropertyValue> {
        None
    }

    fn call_method(
        &self,
        name: CallableIdentifier,
        _arguments: &[CnvValue],
        _context: RunnerContext,
    ) -> RunnerResult<Option<CnvValue>> {
        match name {
            ident => todo!("{:?}.call_method for {:?}", self.get_type_id(), ident),
        }
    }

    fn new(
        _parent: Arc<CnvObject>,
        _properties: HashMap<String, String>,
    ) -> Result<Self, TypeParsingError>
    where
        Self: Sized,
    {
        Ok(Self {})
    }
}

pub struct CnvTypeFactory;

impl CnvTypeFactory {
    pub fn create(
        parent: Arc<CnvObject>,
        type_name: String,
        properties: HashMap<String, String>,
    ) -> Result<Box<dyn CnvType>, TypeParsingError> {
        match type_name.as_ref() {
            "ANIMO" => Animation::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "APPLICATION" => {
                Application::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "ARRAY" => Array::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "BEHAVIOUR" => {
                Behavior::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "BOOL" => Bool::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "BUTTON" => Button::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "CANVAS_OBSERVER" => {
                CanvasObserver::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CANVASOBSERVER" => {
                CanvasObserver::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CNVLOADER" => {
                CnvLoader::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "CONDITION" => {
                Condition::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "COMPLEXCONDITION" => {
                ComplexCondition::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "DOUBLE" => Dbl::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "EPISODE" => Episode::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "EXPRESSION" => {
                Expression::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "FONT" => Font::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "GROUP" => Group::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "IMAGE" => Image::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "INTEGER" => Int::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "KEYBOARD" => {
                Keyboard::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "MOUSE" => Mouse::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "MULTIARRAY" => {
                MultiArray::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "MUSIC" => Music::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "RANDOM" => Random::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SCENE" => Scene::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SEQUENCE" => {
                Sequence::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>)
            }
            "SOUND" => Sound::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "STRING" => Str::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "STRUCT" => Struct::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "SYSTEM" => System::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "TEXT" => Text::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            "TIMER" => Timer::new(parent, properties).map(|o| Box::new(o) as Box<dyn CnvType>),
            _ => Err(TypeParsingError::UnknownType(type_name)),
        }
    }
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
mod dbl;
mod episode;
mod expression;
mod font;
mod group;
mod image;
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
pub use bool::Bool;
pub use button::Button;
pub use canvasobserver::CanvasObserver;
pub use cnvloader::CnvLoader;
pub use complexcondition::ComplexCondition;
pub use condition::Condition;
pub use dbl::Dbl;
pub use episode::Episode;
pub use expression::Expression;
pub use font::Font;
pub use group::Group;
pub use image::Image;
pub use int::Int;
pub use keyboard::Keyboard;
pub use lalrpop_util::ParseError;
pub use mouse::Mouse;
pub use multiarray::MultiArray;
pub use music::Music;
pub use object::CnvObject;
pub use object::CnvObjectBuilder;
pub use object::ObjectBuilderError;
pub use parsers::PropertyValue; // poison
pub use random::Random;
pub use scene::Scene;
pub use sequence::Sequence;
pub use sound::Sound;
pub use str::Str;
pub use structure::Struct;
pub use system::System;
pub use text::Text;
pub use timer::Timer;
