use std::collections::HashMap;

use chrono::{DateTime, Utc};
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{ast::IgnorableProgram, runner::CnvValue};

pub type EpisodeName = String;
pub type SceneName = String;
pub type ConditionName = String;
pub type ImageName = String;
pub type SoundName = String;
pub type VariableName = String;
pub type TypeName = String;
pub type FontName = String;

#[derive(Debug, Clone)]
pub struct CnvObjectBuilder {
    name: String,
    index: usize,
    properties: HashMap<String, String>,
}

impl CnvObjectBuilder {
    pub fn new(name: String, index: usize) -> Self {
        Self {
            name,
            index,
            properties: HashMap::new(),
        }
    }

    pub fn add_property(&mut self, property: String, value: String) {
        self.properties.insert(property, value); // TODO: report duplicates
    }

    pub fn build(self) -> Result<CnvObject, &'static str> {
        let mut properties = self.properties;
        let Some(type_name) = properties.remove("TYPE").and_then(discard_if_empty) else {
            return Err("Missing type."); // TODO: readable errors
        };
        let content = CnvType::new(type_name, properties).map_err(|_| "Parsing error.")?;
        Ok(CnvObject {
            name: self.name,
            index: self.index,
            content,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CnvObject {
    pub name: String,
    pub index: usize,
    pub content: CnvType,
}

impl CnvObject {
    pub fn call_method(&mut self, _name: &str) -> Option<CnvValue> {
        todo!()
    }

    pub fn get_value(&self) -> Option<CnvValue> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum CnvType {
    Animation(Animation),
    Application(Application),
    Array(Array),
    Behavior(Behavior),
    Boolean(Bool),
    Button(Button),
    CanvasObserver(CanvasObserver),
    CnvLoader(CnvLoader),
    Condition(Condition),
    ComplexCondition(ComplexCondition),
    Double(Dbl),
    Episode(Episode),
    Expression(Expression),
    Font(Font),
    Group(Group),
    Image(Image),
    Integer(Int),
    Keyboard(Keyboard),
    Mouse(Mouse),
    MultiArray(MultiArray),
    Music(Music),
    Random(Random),
    Scene(Scene),
    Sequence(Sequence),
    Sound(Sound),
    String(Str),
    Struct(Struct),
    System(System),
    Text(Text),
    Timer(Timer),
}

impl CnvType {
    pub fn new(type_name: String, properties: HashMap<String, String>) -> Result<Self, ()> {
        match type_name.as_ref() {
            "ANIMO" => Animation::new(properties).map(|o| CnvType::Animation(o)),
            "APPLICATION" => Application::new(properties).map(|o| CnvType::Application(o)),
            "ARRAY" => Array::new(properties).map(|o| CnvType::Array(o)),
            "BEHAVIOUR" => Behavior::new(properties).map(|o| CnvType::Behavior(o)),
            "BOOL" => Bool::new(properties).map(|o| CnvType::Boolean(o)),
            "BUTTON" => Button::new(properties).map(|o| CnvType::Button(o)),
            "CANVAS_OBSERVER" => {
                CanvasObserver::new(properties).map(|o| CnvType::CanvasObserver(o))
            }
            "CANVASOBSERVER" => CanvasObserver::new(properties).map(|o| CnvType::CanvasObserver(o)),
            "CNVLOADER" => CnvLoader::new(properties).map(|o| CnvType::CnvLoader(o)),
            "CONDITION" => Condition::new(properties).map(|o| CnvType::Condition(o)),
            "COMPLEXCONDITION" => {
                ComplexCondition::new(properties).map(|o| CnvType::ComplexCondition(o))
            }
            "DOUBLE" => Dbl::new(properties).map(|o| CnvType::Double(o)),
            "EPISODE" => Episode::new(properties).map(|o| CnvType::Episode(o)),
            "EXPRESSION" => Expression::new(properties).map(|o| CnvType::Expression(o)),
            "FONT" => Font::new(properties).map(|o| CnvType::Font(o)),
            "GROUP" => Group::new(properties).map(|o| CnvType::Group(o)),
            "IMAGE" => Image::new(properties).map(|o| CnvType::Image(o)),
            "INTEGER" => Int::new(properties).map(|o| CnvType::Integer(o)),
            "KEYBOARD" => Keyboard::new(properties).map(|o| CnvType::Keyboard(o)),
            "MOUSE" => Mouse::new(properties).map(|o| CnvType::Mouse(o)),
            "MULTIARRAY" => MultiArray::new(properties).map(|o| CnvType::MultiArray(o)),
            "MUSIC" => Music::new(properties).map(|o| CnvType::Music(o)),
            "RANDOM" => Random::new(properties).map(|o| CnvType::Random(o)),
            "SCENE" => Scene::new(properties).map(|o| CnvType::Scene(o)),
            "SEQUENCE" => Sequence::new(properties).map(|o| CnvType::Sequence(o)),
            "SOUND" => Sound::new(properties).map(|o| CnvType::Sound(o)),
            "STRING" => Str::new(properties).map(|o| CnvType::String(o)),
            "STRUCT" => Struct::new(properties).map(|o| CnvType::Struct(o)),
            "SYSTEM" => System::new(properties).map(|o| CnvType::System(o)),
            "TEXT" => Text::new(properties).map(|o| CnvType::Text(o)),
            "TIMER" => Timer::new(properties).map(|o| CnvType::Timer(o)),
            _ => panic!("Unknown type: {}", &type_name),
        }
    }
}

fn parse_bool(s: String) -> Result<bool, ()> {
    match s.as_ref() {
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        _ => Err(()),
    }
}

fn parse_i32(s: String) -> Result<i32, ()> {
    s.parse().map_err(|_| ())
}

fn parse_f64(s: String) -> Result<f64, ()> {
    s.parse().map_err(|_| ())
}

fn parse_datetime(_s: String) -> Result<DateTime<Utc>, ()> {
    Ok(DateTime::default()) // TODO: parse date
}

fn parse_comma_separated(s: String) -> Result<Vec<String>, ()> {
    Ok(s.split(',').map(|s| s.trim().to_owned()).collect())
}

fn parse_program(_s: String) -> Result<IgnorableProgram, ()> {
    Ok(IgnorableProgram {
        ignored: false,
        value: crate::ast::Program::Block(Vec::new()),
    }) // TODO: parse program
}

fn parse_rect(s: String) -> Result<Rect, ()> {
    if s.contains(',') {
        s.split(',')
            .map(|s| s.parse().unwrap())
            .collect_tuple()
            .map(|r| Rect::Literal(r))
            .ok_or(())
    } else {
        Ok(Rect::Reference(s))
    }
}

fn discard_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

#[derive(Debug, Clone)]
pub struct Animation {
    // ANIMO
    pub as_button: Option<bool>,               // ASBUTTON
    pub filename: Option<String>,              // FILENAME
    pub flush_after_played: Option<bool>,      // FLUSHAFTERPLAYED
    pub fps: Option<i32>,                      // FPS
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    pub preload: Option<bool>,                 // PRELOAD
    pub priority: Option<i32>,                 // PRIORITY
    pub release: Option<bool>,                 // RELEASE
    pub to_canvas: Option<bool>,               // TOCANVAS
    pub visible: Option<bool>,                 // VISIBLE
}

impl Animation {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let as_button = properties
            .remove("ASBUTTON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let fps = properties
            .remove("FPS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let release = properties
            .remove("RELEASE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            as_button,
            filename,
            flush_after_played,
            fps,
            monitor_collision,
            monitor_collision_alpha,
            preload,
            priority,
            release,
            to_canvas,
            visible,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    // APPLICATION
    pub author: Option<String>,                  // AUTHOR
    pub bloomoo_version: Option<String>,         // BLOOMOO_VERSION
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub description: Option<String>,             // DESCRIPTION
    pub episodes: Option<Vec<EpisodeName>>,      // EPISODES
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub path: Option<String>,                    // PATH
    pub start_with: Option<EpisodeName>,         // STARTWITH
    pub version: Option<String>,                 // VERSION
}

impl Application {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let bloomoo_version = properties
            .remove("BLOOMOO_VERSION")
            .and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let episodes = properties
            .remove("EPISODES")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let start_with = properties.remove("STARTWITH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        Ok(Self {
            author,
            bloomoo_version,
            creation_time,
            description,
            episodes,
            last_modify_time,
            path,
            start_with,
            version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    // ARRAY
    send_on_change: Option<bool>, // SENDONCHANGE
}

impl Array {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let send_on_change = properties
            .remove("SENDONCHANGE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        // TODO: too many properties
        Ok(Self { send_on_change })
    }
}

#[derive(Debug, Clone)]
pub struct Behavior {
    // BEHAVIOUR
    code: Option<IgnorableProgram>,   // CODE
    condition: Option<ConditionName>, // CONDITION
}

impl Behavior {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let code = properties
            .remove("CODE")
            .and_then(discard_if_empty)
            .map(parse_program)
            .transpose()?;
        let condition = properties.remove("CONDITION").and_then(discard_if_empty);
        Ok(Self { code, condition })
    }
}

#[derive(Debug, Clone)]
pub struct Bool {
    // BOOL
    default: Option<bool>,   // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<bool>,     // VALUE
}

impl Bool {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties
            .remove("VALUE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Rect {
    Literal((i32, i32, i32, i32)),
    Reference(String),
}

impl Default for Rect {
    fn default() -> Self {
        Self::Literal(Default::default())
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    // BUTTON
    accent: Option<bool>,            // ACCENT
    drag: Option<bool>,              // DRAG
    draggable: Option<bool>,         // DRAGGABLE
    enable: Option<bool>,            // ENABLE
    gfx_on_click: Option<ImageName>, // GFXONCLICK
    gfx_on_move: Option<ImageName>,  // GFXONMOVE
    gfx_standard: Option<ImageName>, // GFXSTANDARD
    priority: Option<i32>,           // PRIORITY
    rect: Option<Rect>,              // RECT
    snd_on_click: Option<SoundName>, // SNDONCLICK
    snd_on_move: Option<SoundName>,  // SNDONMOVE
    snd_standard: Option<SoundName>, // SNDSTANDARD
}

impl Button {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let accent = properties
            .remove("ACCENT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let drag = properties
            .remove("DRAG")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let draggable = properties
            .remove("DRAGGABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let enable = properties
            .remove("ENABLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let gfx_on_click = properties.remove("GFXONCLICK").and_then(discard_if_empty);
        let gfx_on_move = properties.remove("GFXONMOVE").and_then(discard_if_empty);
        let gfx_standard = properties.remove("GFXSTANDARD").and_then(discard_if_empty);
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let rect = properties
            .remove("RECT")
            .and_then(discard_if_empty)
            .map(parse_rect)
            .transpose()?;
        let snd_on_click = properties.remove("SNDONCLICK").and_then(discard_if_empty);
        let snd_on_move = properties.remove("SNDONMOVE").and_then(discard_if_empty);
        let snd_standard = properties.remove("SNDSTANDARD").and_then(discard_if_empty);
        Ok(Self {
            accent,
            drag,
            draggable,
            enable,
            gfx_on_click,
            gfx_on_move,
            gfx_standard,
            priority,
            rect,
            snd_on_click,
            snd_on_move,
            snd_standard,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CanvasObserver {
    // CANVAS_OBSERVER
}

impl CanvasObserver {
    pub fn new(_properties: HashMap<String, String>) -> Result<Self, ()> {
        Ok(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct CnvLoader {
    // CNVLOADER
    cnv_loader: Option<String>, // CNVLOADER
}

impl CnvLoader {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let cnv_loader = properties.remove("CNVLOADER").and_then(discard_if_empty);
        Ok(Self { cnv_loader })
    }
}

#[derive(Debug, Clone)]
pub enum ConditionOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

impl ConditionOperator {
    pub fn parse(s: String) -> Result<Self, ()> {
        match s.as_ref() {
            "EQUAL" => Ok(Self::Equal),
            "NOTEQUAL" => Ok(Self::NotEqual),
            "LESS" => Ok(Self::Less),
            "GREATER" => Ok(Self::Greater),
            "LESSEQUAL" => Ok(Self::LessEqual),
            "GREATEREQUAL" => Ok(Self::GreaterEqual),
            _ => Err(()), // TODO: error
        }
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    // CONDITION
    operand1: Option<VariableName>,      // OPERAND1
    operand2: Option<VariableName>,      // OPERAND2
    operator: Option<ConditionOperator>, // OPERATOR
}

impl Condition {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ConditionOperator::parse)
            .transpose()?;
        Ok(Self {
            operand1,
            operand2,
            operator,
        })
    }
}

#[derive(Debug, Clone)]
pub enum ComplexConditionOperator {
    And,
    Or,
}

impl ComplexConditionOperator {
    pub fn parse(s: String) -> Result<Self, ()> {
        match s.as_ref() {
            "AND" => Ok(Self::And),
            "OR" => Ok(Self::Or),
            _ => Err(()), // TODO: error
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComplexCondition {
    // COMPLEXCONDITION
    operand1: Option<ConditionName>,            // OPERAND1
    operand2: Option<ConditionName>,            // OPERAND2
    operator: Option<ComplexConditionOperator>, // OPERATOR
}

impl ComplexCondition {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ComplexConditionOperator::parse)
            .transpose()?;
        Ok(Self {
            operand1,
            operand2,
            operator,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Dbl {
    // DOUBLE
    default: Option<f64>,    // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<f64>,      // VALUE
}

impl Dbl {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_f64)
            .transpose()?;
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties
            .remove("VALUE")
            .and_then(discard_if_empty)
            .map(parse_f64)
            .transpose()?;
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Episode {
    // EPISODE
    pub author: Option<String>,                  // AUTHOR
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub description: Option<String>,             // DESCRIPTION
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub path: Option<String>,                    // PATH
    pub scenes: Option<Vec<SceneName>>,          // SCENES
    pub start_with: Option<SceneName>,           // STARTWITH
    pub version: Option<String>,                 // VERSION
}

impl Episode {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let scenes = properties
            .remove("SCENES")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let start_with = properties.remove("STARTWITH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        Ok(Self {
            author,
            creation_time,
            description,
            last_modify_time,
            path,
            scenes,
            start_with,
            version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    // EXPRESSION
    operand1: Option<VariableName>,       // OPERAND1
    operand2: Option<VariableName>,       // OPERAND2
    operator: Option<ExpressionOperator>, // OPERATOR
}

impl Expression {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let operand1 = properties.remove("OPERAND1").and_then(discard_if_empty);
        let operand2 = properties.remove("OPERAND2").and_then(discard_if_empty);
        let operator = properties
            .remove("OPERATOR")
            .and_then(discard_if_empty)
            .map(ExpressionOperator::parse)
            .transpose()?;
        Ok(Self {
            operand1,
            operand2,
            operator,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub enum ExpressionOperator {
    #[default]
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl ExpressionOperator {
    pub fn parse(s: String) -> Result<Self, ()> {
        match s.as_ref() {
            "ADD" => Ok(Self::Add),
            "SUB" => Ok(Self::Sub),
            "MUL" => Ok(Self::Mul),
            "DIV" => Ok(Self::Div),
            "MOD" => Ok(Self::Mod),
            _ => Err(()), // TODO: something better
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FontDef {
    family: String,
    style: String,
    size: usize,
}

#[derive(Debug, Clone)]
pub struct Font {
    // FONT
    defs: HashMap<FontDef, Option<String>>,
}

lazy_static! {
    static ref FONT_DEF_REGEX: Regex = Regex::new(r"^DEF_(\w+)_(\w+)_(\d+)$").unwrap();
}

impl Font {
    pub fn new(properties: HashMap<String, String>) -> Result<Self, ()> {
        let defs: HashMap<FontDef, Option<String>> = properties
            .into_iter()
            .filter_map(|(k, v)| {
                FONT_DEF_REGEX.captures(k.as_ref()).map(|m| {
                    (
                        FontDef {
                            family: m[1].to_owned(),
                            style: m[2].to_owned(),
                            size: m[3].parse().unwrap(),
                        },
                        Some(v),
                    )
                })
            })
            .collect();
        Ok(Self { defs })
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    // GROUP
}

impl Group {
    pub fn new(_properties: HashMap<String, String>) -> Result<Self, ()> {
        Ok(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    // IMAGE
    pub as_button: Option<bool>,               // ASBUTTON
    pub filename: Option<String>,              // FILENAME
    pub flush_after_played: Option<bool>,      // FLUSHAFTERPLAYED
    pub monitor_collision: Option<bool>,       // MONITORCOLLISION
    pub monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    pub preload: Option<bool>,                 // PRELOAD
    pub priority: Option<i32>,                 // PRIORITY
    pub release: Option<bool>,                 // RELEASE
    pub to_canvas: Option<bool>,               // TOCANVAS
    pub visible: Option<bool>,                 // VISIBLE
}

impl Image {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let as_button = properties
            .remove("ASBUTTON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let release = properties
            .remove("RELEASE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            as_button,
            filename,
            flush_after_played,
            monitor_collision,
            monitor_collision_alpha,
            preload,
            priority,
            release,
            to_canvas,
            visible,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Int {
    // INTEGER
    default: Option<i32>,    // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<i32>,      // VALUE
}

impl Int {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let default = properties
            .remove("DEFAULT")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties
            .remove("VALUE")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    // KEYBOARD
    keyboard: Option<String>, // KEYBOARD
}

impl Keyboard {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let keyboard = properties.remove("KEYBOARD").and_then(discard_if_empty);
        Ok(Self { keyboard })
    }
}

#[derive(Debug, Clone)]
pub struct Mouse {
    // MOUSE
    mouse: Option<String>, // MOUSE
    raw: Option<i32>,      // RAW
}

impl Mouse {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let mouse = properties.remove("MOUSE").and_then(discard_if_empty);
        let raw = properties
            .remove("RAW")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self { mouse, raw })
    }
}

#[derive(Debug, Clone)]
pub struct MultiArray {
    // MULTIARRAY
    dimensions: Option<i32>, // DIMENSIONS
}

impl MultiArray {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let dimensions = properties
            .remove("DIMENSIONS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self { dimensions })
    }
}

#[derive(Debug, Clone)]
pub struct Music {
    // MUSIC
    filename: Option<String>, // FILENAME
}

impl Music {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        Ok(Self { filename })
    }
}

#[derive(Debug, Clone)]
pub struct Random {
    // RAND
}

impl Random {
    pub fn new(_properties: HashMap<String, String>) -> Result<Self, ()> {
        Ok(Self {})
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    // SCENE
    pub author: Option<String>,                  // AUTHOR
    pub background: Option<String>,              // BACKGROUND
    pub coauthors: Option<String>,               // COAUTHORS
    pub creation_time: Option<DateTime<Utc>>,    // CREATIONTIME
    pub deamon: Option<bool>,                    // DEAMON
    pub description: Option<String>,             // DESCRIPTION
    pub dlls: Option<Vec<String>>,               // DLLS
    pub last_modify_time: Option<DateTime<Utc>>, // LASTMODIFYTIME
    pub music: Option<String>,                   // MUSIC
    pub path: Option<String>,                    // PATH
    pub version: Option<String>,                 // VERSION
}

impl Scene {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let author = properties.remove("AUTHOR").and_then(discard_if_empty);
        let background = properties
            .remove("BACKGROUND")
            .and_then(discard_if_empty)
            .and_then(|s| if s.is_empty() { None } else { Some(s) });
        let coauthors = properties.remove("COAUTHORS").and_then(discard_if_empty);
        let creation_time = properties
            .remove("CREATIONTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let deamon = properties
            .remove("DEAMON")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let description = properties.remove("DESCRIPTION").and_then(discard_if_empty);
        let dlls = properties
            .remove("DLLS")
            .and_then(discard_if_empty)
            .map(parse_comma_separated)
            .transpose()?;
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .and_then(discard_if_empty)
            .map(parse_datetime)
            .transpose()?;
        let music = properties.remove("MUSIC").and_then(discard_if_empty);
        let path = properties.remove("PATH").and_then(discard_if_empty);
        let version = properties.remove("VERSION").and_then(discard_if_empty);
        Ok(Self {
            author,
            background,
            coauthors,
            creation_time,
            deamon,
            description,
            dlls,
            last_modify_time,
            music,
            path,
            version,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    // SEQUENCE
    filename: Option<String>, // FILENAME
}

impl Sequence {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        Ok(Self { filename })
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    // SOUND
    filename: Option<String>,         // FILENAME
    flush_after_played: Option<bool>, // FLUSHAFTERPLAYED
    preload: Option<bool>,            // PRELOAD
}

impl Sound {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let filename = properties.remove("FILENAME").and_then(discard_if_empty);
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let preload = properties
            .remove("PRELOAD")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            filename,
            flush_after_played,
            preload,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Str {
    // STRING
    default: Option<String>, // DEFAULT
    netnotify: Option<bool>, // NETNOTIFY
    to_ini: Option<bool>,    // TOINI
    value: Option<String>,   // VALUE
}

impl Str {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let default = properties.remove("DEFAULT").and_then(discard_if_empty);
        let netnotify = properties
            .remove("NETNOTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let to_ini = properties
            .remove("TOINI")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let value = properties.remove("VALUE").and_then(discard_if_empty);
        Ok(Self {
            default,
            netnotify,
            to_ini,
            value,
        })
    }
}

lazy_static! {
    static ref STRUCT_FIELDS_REGEX: Regex = Regex::new(r"^(\w+)<(\w+)>$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Struct {
    // STRUCT
    fields: Option<Vec<(String, TypeName)>>,
}

impl Struct {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let fields = properties
            .remove("FIELDS")
            .and_then(discard_if_empty)
            .map(|s| {
                s.split(',')
                    .map(|f| {
                        let m = STRUCT_FIELDS_REGEX.captures(f).unwrap();
                        (m[1].to_owned(), m[2].to_owned())
                    })
                    .collect()
            });
        Ok(Self { fields })
    }
}

#[derive(Debug, Clone)]
pub struct System {
    // SYSTEM
    system: Option<String>, // SYSTEM
}

impl System {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let system = properties.remove("SYSTEM").and_then(discard_if_empty);
        Ok(Self { system })
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    // TEXT
    font: Option<FontName>,                // FONT
    horizontal_justify: Option<bool>,      // HJUSTIFY
    hypertext: Option<bool>,               // HYPERTEXT
    monitor_collision: Option<bool>,       // MONITORCOLLISION
    monitor_collision_alpha: Option<bool>, // MONITORCOLLISIONALPHA
    priority: Option<i32>,                 // PRIORITY
    rect: Option<Rect>,                    // RECT
    text: Option<String>,                  // TEXT
    to_canvas: Option<bool>,               // TOCANVAS
    visible: Option<bool>,                 // VISIBLE
    vertical_justify: Option<bool>,        // VJUSTIFY
}

impl Text {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let font = properties.remove("FONT").and_then(discard_if_empty);
        let horizontal_justify = properties
            .remove("HJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let hypertext = properties
            .remove("HYPERTEXT")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let priority = properties
            .remove("PRIORITY")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let rect = properties
            .remove("RECT")
            .and_then(discard_if_empty)
            .map(parse_rect)
            .transpose()?;
        let text = properties.remove("TEXT").and_then(discard_if_empty);
        let to_canvas = properties
            .remove("TOCANVAS")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let visible = properties
            .remove("VISIBLE")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let vertical_justify = properties
            .remove("VJUSTIFY")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        Ok(Self {
            font,
            horizontal_justify,
            hypertext,
            monitor_collision,
            monitor_collision_alpha,
            priority,
            rect,
            text,
            to_canvas,
            visible,
            vertical_justify,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Timer {
    // TIMER
    elapse: Option<i32>,   // ELAPSE
    enabled: Option<bool>, // ENABLED
    ticks: Option<i32>,    // TICKS
}

impl Timer {
    pub fn new(mut properties: HashMap<String, String>) -> Result<Self, ()> {
        let elapse = properties
            .remove("ELAPSE")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        let enabled = properties
            .remove("ENABLED")
            .and_then(discard_if_empty)
            .map(parse_bool)
            .transpose()?;
        let ticks = properties
            .remove("TICKS")
            .and_then(discard_if_empty)
            .map(parse_i32)
            .transpose()?;
        Ok(Self {
            elapse,
            enabled,
            ticks,
        })
    }
}