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

impl<'a> From<&'a CnvContent> for Option<&'a Animation> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Animation(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Application> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Application(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Array> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Array(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Behavior> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Behavior(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a BoolVar> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Bool(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Button> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Button(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a CanvasObserver> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::CanvasObserver(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a CnvLoader> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::CnvLoader(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Condition> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Condition(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a ComplexCondition> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::ComplexCondition(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a DoubleVar> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Double(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Episode> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Episode(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Expression> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Expression(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Font> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Font(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Group> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Group(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Image> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Image(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a IntegerVar> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Integer(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Keyboard> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Keyboard(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Mouse> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Mouse(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a MultiArray> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::MultiArray(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Music> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Music(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Rand> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Rand(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Scene> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Scene(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Sequence> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Sequence(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Sound> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Sound(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a StringVar> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::String(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Struct> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Struct(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a System> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::System(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Text> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Text(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a CnvContent> for Option<&'a Timer> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Timer(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Animation> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Animation(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Application> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Application(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Array> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Array(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Behavior> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Behavior(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut BoolVar> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Bool(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Button> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Button(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut CanvasObserver> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::CanvasObserver(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut CnvLoader> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::CnvLoader(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Condition> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Condition(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut ComplexCondition> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::ComplexCondition(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut DoubleVar> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Double(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Episode> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Episode(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Expression> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Expression(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Font> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Font(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Group> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Group(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Image> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Image(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut IntegerVar> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Integer(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Keyboard> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Keyboard(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Mouse> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Mouse(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut MultiArray> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::MultiArray(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Music> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Music(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Rand> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Rand(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Scene> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Scene(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Sequence> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Sequence(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Sound> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Sound(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut StringVar> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::String(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Struct> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Struct(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut System> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::System(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Text> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Text(content) => Some(content),
            _ => None,
        }
    }
}

impl<'a> From<&'a mut CnvContent> for Option<&'a mut Timer> {
    fn from(value: &'a mut CnvContent) -> Self {
        match value {
            CnvContent::Timer(content) => Some(content),
            _ => None,
        }
    }
}
