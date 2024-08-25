use std::{any::Any, collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;

use super::{content::CnvContent, parsers::TypeParsingError, CallableIdentifier, CnvObject};
use crate::runner::{CnvValue, RunnerContext};

pub trait CnvType: std::fmt::Debug {
    fn get_type_id(&self) -> &'static str;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn call_method(
        &self,
        identifier: CallableIdentifier,
        arguments: &[CnvValue],
        context: RunnerContext,
    ) -> anyhow::Result<CnvValue>;

    fn new_content(
        parent: Arc<CnvObject>,
        properties: HashMap<String, String>,
    ) -> Result<CnvContent, TypeParsingError>
    where
        Self: Sized;
}

impl dyn CnvType {}

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
    ) -> anyhow::Result<CnvValue> {
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

pub trait GeneralCondition {
    fn check(&self) -> anyhow::Result<bool>;
}

pub trait GeneralGraphics {
    fn show(&self) -> anyhow::Result<()>;
    fn hide(&self) -> anyhow::Result<()>;
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
mod double;
mod episode;
mod expression;
mod font;
mod group;
mod image;
mod integer;
mod keyboard;
mod mouse;
mod multiarray;
mod music;
mod rand;
mod scene;
mod sequence;
mod sound;
mod string;
mod r#struct;
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
pub use double::DoubleVar;
pub use episode::Episode;
pub use expression::Expression;
pub use font::Font;
pub use group::Group;
pub use image::Image;
pub use integer::IntegerVar;
pub use keyboard::Keyboard;
pub use lalrpop_util::ParseError;
pub use mouse::{InternalMouseEvent, Mouse};
pub use multiarray::MultiArray;
pub use music::Music;
pub use r#struct::Struct;
pub use rand::Rand;
pub use scene::Scene;
pub use sequence::Sequence;
pub use sound::Sound;
pub use string::StringVar;
pub use system::System;
pub use text::Text;
pub use timer::Timer;
