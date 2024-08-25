use crate::runner::RunnerContext;

use super::content::CnvContent;

pub trait Initable {
    fn initialize(&self, context: RunnerContext) -> anyhow::Result<()>;
}

impl<'a> From<&'a CnvContent> for Option<&'a dyn Initable> {
    fn from(value: &'a CnvContent) -> Self {
        match value {
            CnvContent::Animation(content) => Some(content),
            CnvContent::Array(content) => Some(content),
            CnvContent::Behavior(content) => Some(content),
            CnvContent::Bool(content) => Some(content),
            CnvContent::Button(content) => Some(content),
            CnvContent::CanvasObserver(content) => Some(content),
            CnvContent::Double(content) => Some(content),
            CnvContent::Font(content) => Some(content),
            CnvContent::Group(content) => Some(content),
            CnvContent::Image(content) => Some(content),
            CnvContent::Integer(content) => Some(content),
            CnvContent::Keyboard(content) => Some(content),
            CnvContent::Mouse(content) => Some(content),
            CnvContent::Scene(content) => Some(content),
            CnvContent::Sequence(content) => Some(content),
            CnvContent::Sound(content) => Some(content),
            CnvContent::String(content) => Some(content),
            CnvContent::Struct(content) => Some(content),
            CnvContent::Text(content) => Some(content),
            CnvContent::Timer(content) => Some(content),
            _ => None,
        }
    }
}
