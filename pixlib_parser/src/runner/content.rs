use std::{ops::Deref, sync::Arc};

use crate::parser::ast::ParsedScript;

use super::classes::*;

#[derive(Debug)]
pub enum CnvContent {
    Animation(Animation),
    Application(Application),
    Array(Array),
    Behavior(Behavior),
    Bool(BoolVar),
    Button(Button),
    CanvasObserver(CanvasObserver),
    CnvLoader(CnvLoader),
    Condition(Condition),
    ComplexCondition(ComplexCondition),
    Double(DoubleVar),
    Episode(Episode),
    Expression(Expression),
    Font(Font),
    Group(Group),
    Image(Image),
    Integer(IntegerVar),
    Keyboard(Keyboard),
    Mouse(Mouse),
    MultiArray(MultiArray),
    Music(Music),
    Rand(Rand),
    Scene(Scene),
    Sequence(Sequence),
    Sound(Sound),
    String(StringVar),
    Struct(Struct),
    System(System),
    Text(Text),
    Timer(Timer),
    Custom(Box<dyn CnvType>), // TODO: allow for ONINIT here
    None(DummyCnvType),
}

pub trait EventHandler {
    fn get(&self, name: &str, argument: Option<&str>) -> Option<&Arc<ParsedScript>>;
}

impl AsRef<dyn CnvType> for CnvContent {
    fn as_ref(&self) -> &(dyn CnvType + 'static) {
        match self {
            CnvContent::Animation(content) => content,
            CnvContent::Application(content) => content,
            CnvContent::Array(content) => content,
            CnvContent::Behavior(content) => content,
            CnvContent::Bool(content) => content,
            CnvContent::Button(content) => content,
            CnvContent::CanvasObserver(content) => content,
            CnvContent::CnvLoader(content) => content,
            CnvContent::Condition(content) => content,
            CnvContent::ComplexCondition(content) => content,
            CnvContent::Double(content) => content,
            CnvContent::Episode(content) => content,
            CnvContent::Expression(content) => content,
            CnvContent::Font(content) => content,
            CnvContent::Group(content) => content,
            CnvContent::Image(content) => content,
            CnvContent::Integer(content) => content,
            CnvContent::Keyboard(content) => content,
            CnvContent::Mouse(content) => content,
            CnvContent::MultiArray(content) => content,
            CnvContent::Music(content) => content,
            CnvContent::Rand(content) => content,
            CnvContent::Scene(content) => content,
            CnvContent::Sequence(content) => content,
            CnvContent::Sound(content) => content,
            CnvContent::String(content) => content,
            CnvContent::Struct(content) => content,
            CnvContent::System(content) => content,
            CnvContent::Text(content) => content,
            CnvContent::Timer(content) => content,
            CnvContent::Custom(content) => &**content,
            CnvContent::None(content) => content,
        }
    }
}

impl Deref for CnvContent {
    type Target = dyn CnvType;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
