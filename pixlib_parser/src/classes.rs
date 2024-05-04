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
        let Some(type_name) = properties.remove("TYPE") else {
            return Err("Missing type."); // TODO: readable errors
        };
        Ok(CnvObject {
            name: self.name,
            index: self.index,
            content: CnvType::new(type_name, properties),
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
    pub fn new(type_name: String, properties: HashMap<String, String>) -> Self {
        match type_name.as_ref() {
            "ANIMO" => CnvType::Animation(Animation::new(properties)),
            "APPLICATION" => CnvType::Application(Application::new(properties)),
            "ARRAY" => CnvType::Array(Array::new(properties)),
            "BEHAVIOUR" => CnvType::Behavior(Behavior::new(properties)),
            "BOOL" => CnvType::Boolean(Bool::new(properties)),
            "BUTTON" => CnvType::Button(Button::new(properties)),
            "CANVAS_OBSERVER" => CnvType::CanvasObserver(CanvasObserver::new(properties)),
            "CANVASOBSERVER" => CnvType::CanvasObserver(CanvasObserver::new(properties)),
            "CONDITION" => CnvType::Condition(Condition::new(properties)),
            "COMPLEXCONDITION" => CnvType::ComplexCondition(ComplexCondition::new(properties)),
            "DOUBLE" => CnvType::Double(Dbl::new(properties)),
            "EPISODE" => CnvType::Episode(Episode::new(properties)),
            "EXPRESSION" => CnvType::Expression(Expression::new(properties)),
            "FONT" => CnvType::Font(Font::new(properties)),
            "GROUP" => CnvType::Group(Group::new(properties)),
            "IMAGE" => CnvType::Image(Image::new(properties)),
            "INTEGER" => CnvType::Integer(Int::new(properties)),
            "KEYBOARD" => CnvType::Keyboard(Keyboard::new(properties)),
            "MOUSE" => CnvType::Mouse(Mouse::new(properties)),
            "RANDOM" => CnvType::Random(Random::new(properties)),
            "SCENE" => CnvType::Scene(Scene::new(properties)),
            "SEQUENCE" => CnvType::Sequence(Sequence::new(properties)),
            "SOUND" => CnvType::Sound(Sound::new(properties)),
            "STRING" => CnvType::String(Str::new(properties)),
            "STRUCT" => CnvType::Struct(Struct::new(properties)),
            "SYSTEM" => CnvType::System(System::new(properties)),
            "TEXT" => CnvType::Text(Text::new(properties)),
            "TIMER" => CnvType::Timer(Timer::new(properties)),
            _ => panic!("Unknown type: {}", &type_name),
        }
    }
}

fn parse_bool(s: String) -> bool {
    s == "TRUE"
}

fn parse_i32(s: String) -> i32 {
    s.parse().unwrap()
}

fn parse_f64(s: String) -> f64 {
    s.parse().unwrap()
}

fn parse_datetime(_s: String) -> DateTime<Utc> {
    DateTime::default() // TODO: parse date
}

fn parse_comma_separated(s: String) -> Vec<String> {
    s.split(',').map(|s| s.to_owned()).collect()
}

fn parse_program(_s: String) -> IgnorableProgram {
    IgnorableProgram {
        ignored: false,
        value: crate::ast::Program::Block(Vec::new()),
    } // TODO: parse program
}

fn parse_rect(s: String) -> (i32, i32, i32, i32) {
    s.split(',')
        .map(|s| s.parse().unwrap())
        .collect_tuple()
        .unwrap()
}

#[derive(Debug, Clone)]
pub struct Animation {
    // ANIMO
    pub as_button: bool,               // ASBUTTON
    pub filename: String,              // FILENAME
    pub flush_after_played: bool,      // FLUSHAFTERPLAYED
    pub fps: i32,                      // FPS
    pub monitor_collision: bool,       // MONITORCOLLISION
    pub monitor_collision_alpha: bool, // MONITORCOLLISIONALPHA
    pub preload: bool,                 // PRELOAD
    pub priority: i32,                 // PRIORITY
    pub release: bool,                 // RELEASE
    pub to_canvas: bool,               // TOCANVAS
    pub visible: bool,                 // VISIBLE
}

impl Animation {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let as_button = properties
            .remove("ASBUTTON")
            .map(parse_bool)
            .unwrap_or(false);
        let filename = properties.remove("FILENAME").unwrap_or_default();
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .map(parse_bool)
            .unwrap_or(false);
        let fps = properties.remove("FPS").map(parse_i32).unwrap_or(0);
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .map(parse_bool)
            .unwrap_or(false);
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .map(parse_bool)
            .unwrap_or(false);
        let preload = properties
            .remove("PRELOAD")
            .map(parse_bool)
            .unwrap_or(false);
        let priority = properties.remove("PRIORITY").map(parse_i32).unwrap_or(0);
        let release = properties
            .remove("RELEASE")
            .map(parse_bool)
            .unwrap_or(false);
        let to_canvas = properties
            .remove("TOCANVAS")
            .map(parse_bool)
            .unwrap_or(false);
        let visible = properties
            .remove("VISIBLE")
            .map(parse_bool)
            .unwrap_or(false);
        Self {
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Application {
    // APPLICATION
    author: String,                  // AUTHOR
    bloomoo_version: String,         // BLOOMOO_VERSION
    creation_time: DateTime<Utc>,    // CREATIONTIME
    description: String,             // DESCRIPTION
    episodes: Vec<EpisodeName>,      // EPISODES
    last_modify_time: DateTime<Utc>, // LASTMODIFYTIME
    path: String,                    // PATH
    scenes: Vec<SceneName>,          // SCENES
    start_with: EpisodeName,         // STARTWITH
    version: String,                 // VERSION
}

impl Application {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let author = properties.remove("AUTHOR").unwrap_or_default();
        let bloomoo_version = properties.remove("BLOOMOO_VERSION").unwrap_or_default();
        let creation_time = properties
            .remove("CREATIONTIME")
            .map(parse_datetime)
            .unwrap_or_default();
        let description = properties.remove("DESCRIPTION").unwrap_or_default();
        let episodes = properties
            .remove("EPISODES")
            .map(parse_comma_separated)
            .unwrap_or_default();
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .map(parse_datetime)
            .unwrap_or_default();
        let path = properties.remove("PATH").unwrap_or_default();
        let scenes = properties
            .remove("SCENES")
            .map(parse_comma_separated)
            .unwrap_or_default();
        let start_with = properties.remove("STARTWITH").unwrap_or_default();
        let version = properties.remove("VERSION").unwrap_or_default();
        Self {
            author,
            bloomoo_version,
            creation_time,
            description,
            episodes,
            last_modify_time,
            path,
            scenes,
            start_with,
            version,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Array {
    // ARRAY
    send_on_change: bool, // SENDONCHANGE
}

impl Array {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let send_on_change = properties
            .remove("SENDONCHANGE")
            .map(parse_bool)
            .unwrap_or(false);
        // TODO: too many properties
        Self { send_on_change }
    }
}

#[derive(Debug, Clone)]
pub struct Behavior {
    // BEHAVIOUR
    code: IgnorableProgram,   // CODE
    condition: ConditionName, // CONDITION
}

impl Behavior {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let code = properties
            .remove("CODE")
            .map(parse_program)
            .unwrap_or(IgnorableProgram {
                ignored: false,
                value: crate::ast::Program::Block(Vec::new()),
            });
        let condition = properties.remove("CONDITION").unwrap_or_default();
        Self { code, condition }
    }
}

#[derive(Debug, Clone)]
pub struct Bool {
    // BOOL
    default: bool,   // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: bool,     // VALUE
}

impl Bool {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let default = properties
            .remove("DEFAULT")
            .map(parse_bool)
            .unwrap_or(false);
        let netnotify = properties
            .remove("NETNOTIFY")
            .map(parse_bool)
            .unwrap_or(false);
        let to_ini = properties.remove("TOINI").map(parse_bool).unwrap_or(false);
        let value = properties.remove("VALUE").map(parse_bool).unwrap_or(false);
        Self {
            default,
            netnotify,
            to_ini,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Button {
    // BUTTON
    accent: bool,               // ACCENT
    drag: bool,                 // DRAG
    draggable: bool,            // DRAGGABLE
    enable: bool,               // ENABLE
    gfx_on_click: ImageName,    // GFXONCLICK
    gfx_on_move: ImageName,     // GFXONMOVE
    gfx_standard: ImageName,    // GFXSTANDARD
    priority: i32,              // PRIORITY
    rect: (i32, i32, i32, i32), // RECT
    snd_on_click: SoundName,    // SNDONCLICK
    snd_on_move: SoundName,     // SNDONMOVE
    snd_standard: SoundName,    // SNDSTANDARD
}

impl Button {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let accent = properties.remove("ACCENT").map(parse_bool).unwrap_or(false);
        let drag = properties.remove("DRAG").map(parse_bool).unwrap_or(false);
        let draggable = properties
            .remove("DRAGGABLE")
            .map(parse_bool)
            .unwrap_or(false);
        let enable = properties.remove("ENABLE").map(parse_bool).unwrap_or(false);
        let gfx_on_click = properties.remove("GFXONCLICK").unwrap_or_default();
        let gfx_on_move = properties.remove("GFXONMOVE").unwrap_or_default();
        let gfx_standard = properties.remove("GFXSTANDARD").unwrap_or_default();
        let priority = properties.remove("PRIORITY").map(parse_i32).unwrap_or(0);
        let rect = properties
            .remove("RECT")
            .map(parse_rect)
            .unwrap_or_default();
        let snd_on_click = properties.remove("SNDONCLICK").unwrap_or_default();
        let snd_on_move = properties.remove("SNDONMOVE").unwrap_or_default();
        let snd_standard = properties.remove("SNDSTANDARD").unwrap_or_default();
        Self {
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct CanvasObserver {
    // CANVAS_OBSERVER
}

impl CanvasObserver {
    pub fn new(_properties: HashMap<String, String>) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct Condition {
    // CONDITION
    operand1: VariableName,      // OPERAND1
    operand2: VariableName,      // OPERAND2
    operator: ConditionOperator, // OPERATOR
}

impl Condition {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let operand1 = properties.remove("OPERAND1").unwrap_or_default();
        let operand2 = properties.remove("OPERAND2").unwrap_or_default();
        let operator = properties
            .remove("OPERATOR")
            .map(ConditionOperator::parse)
            .unwrap_or_default();
        Self {
            operand1,
            operand2,
            operator,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComplexCondition {
    // COMPLEXCONDITION
    operand1: ConditionName,     // OPERAND1
    operand2: ConditionName,     // OPERAND2
    operator: ConditionOperator, // OPERATOR
}

impl ComplexCondition {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let operand1 = properties.remove("OPERAND1").unwrap_or_default();
        let operand2 = properties.remove("OPERAND2").unwrap_or_default();
        let operator = properties
            .remove("OPERATOR")
            .map(ConditionOperator::parse)
            .unwrap_or_default();
        Self {
            operand1,
            operand2,
            operator,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub enum ConditionOperator {
    #[default]
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

impl ConditionOperator {
    pub fn parse(s: String) -> Self {
        match s.as_ref() {
            "EQUAL" => Self::Equal,
            "NOTEQUAL" => Self::NotEqual,
            "LESS" => Self::Less,
            "GREATER" => Self::Greater,
            "LESSEQUAL" => Self::LessEqual,
            "GREATEREQUAL" => Self::GreaterEqual,
            _ => Self::Equal, // TODO: error
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dbl {
    // DOUBLE
    default: f64,    // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: f64,      // VALUE
}

impl Dbl {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let default = properties.remove("DEFAULT").map(parse_f64).unwrap_or(0.0);
        let netnotify = properties
            .remove("NETNOTIFY")
            .map(parse_bool)
            .unwrap_or(false);
        let to_ini = properties.remove("TOINI").map(parse_bool).unwrap_or(false);
        let value = properties.remove("VALUE").map(parse_f64).unwrap_or(0.0);
        Self {
            default,
            netnotify,
            to_ini,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Episode {
    // EPISODE
    author: String,                  // AUTHOR
    creation_time: DateTime<Utc>,    // CREATIONTIME
    description: String,             // DESCRIPTION
    last_modify_time: DateTime<Utc>, // LASTMODIFYTIME
    path: String,                    // PATH
    scenes: Vec<SceneName>,          // SCENES
    start_with: SceneName,           // STARTWITH
    version: String,                 // VERSION
}

impl Episode {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let author = properties.remove("AUTHOR").unwrap_or_default();
        let creation_time = properties
            .remove("CREATIONTIME")
            .map(parse_datetime)
            .unwrap_or_default();
        let description = properties.remove("DESCRIPTION").unwrap_or_default();
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .map(parse_datetime)
            .unwrap_or_default();
        let path = properties.remove("PATH").unwrap_or_default();
        let scenes = properties
            .remove("SCENES")
            .map(parse_comma_separated)
            .unwrap_or_default();
        let start_with = properties.remove("STARTWITH").unwrap_or_default();
        let version = properties.remove("VERSION").unwrap_or_default();
        Self {
            author,
            creation_time,
            description,
            last_modify_time,
            path,
            scenes,
            start_with,
            version,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Expression {
    // EXPRESSION
    operand1: VariableName,       // OPERAND1
    operand2: VariableName,       // OPERAND2
    operator: ExpressionOperator, // OPERATOR
}

impl Expression {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let operand1 = properties.remove("OPERAND1").unwrap_or_default();
        let operand2 = properties.remove("OPERAND2").unwrap_or_default();
        let operator = properties
            .remove("OPERATOR")
            .map(ExpressionOperator::parse)
            .unwrap_or_default();
        Self {
            operand1,
            operand2,
            operator,
        }
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
    pub fn parse(s: String) -> Self {
        match s.as_ref() {
            "ADD" => Self::Add,
            "SUB" => Self::Sub,
            "MUL" => Self::Mul,
            "DIV" => Self::Div,
            "MOD" => Self::Mod,
            _ => Self::Add, // TODO: something better
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
    defs: HashMap<FontDef, String>,
}

lazy_static! {
    static ref FONT_DEF_REGEX: Regex = Regex::new(r"^DEF_(\w+)_(\w+)_(\d+)$").unwrap();
}

impl Font {
    pub fn new(properties: HashMap<String, String>) -> Self {
        let defs: HashMap<FontDef, String> = properties
            .into_iter()
            .filter_map(|(k, v)| {
                FONT_DEF_REGEX.captures(k.as_ref()).map(|m| {
                    (
                        FontDef {
                            family: m[1].to_owned(),
                            style: m[2].to_owned(),
                            size: m[3].parse().unwrap(),
                        },
                        v,
                    )
                })
            })
            .collect();
        Self { defs }
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    // GROUP
}

impl Group {
    pub fn new(_properties: HashMap<String, String>) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct Image {
    // IMAGE
    pub as_button: bool,               // ASBUTTON
    pub filename: String,              // FILENAME
    pub flush_after_played: bool,      // FLUSHAFTERPLAYED
    pub monitor_collision: bool,       // MONITORCOLLISION
    pub monitor_collision_alpha: bool, // MONITORCOLLISIONALPHA
    pub preload: bool,                 // PRELOAD
    pub priority: i32,                 // PRIORITY
    pub release: bool,                 // RELEASE
    pub to_canvas: bool,               // TOCANVAS
    pub visible: bool,                 // VISIBLE
}

impl Image {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let as_button = properties
            .remove("ASBUTTON")
            .map(parse_bool)
            .unwrap_or(false);
        let filename = properties.remove("FILENAME").unwrap_or_default();
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .map(parse_bool)
            .unwrap_or(false);
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .map(parse_bool)
            .unwrap_or(false);
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .map(parse_bool)
            .unwrap_or(false);
        let preload = properties
            .remove("PRELOAD")
            .map(parse_bool)
            .unwrap_or(false);
        let priority = properties.remove("PRIORITY").map(parse_i32).unwrap_or(0);
        let release = properties
            .remove("RELEASE")
            .map(parse_bool)
            .unwrap_or(false);
        let to_canvas = properties
            .remove("TOCANVAS")
            .map(parse_bool)
            .unwrap_or(false);
        let visible = properties
            .remove("VISIBLE")
            .map(parse_bool)
            .unwrap_or(false);
        Self {
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Int {
    // INTEGER
    default: i32,    // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: i32,      // VALUE
}

impl Int {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let default = properties.remove("DEFAULT").map(parse_i32).unwrap_or(0);
        let netnotify = properties
            .remove("NETNOTIFY")
            .map(parse_bool)
            .unwrap_or(false);
        let to_ini = properties.remove("TOINI").map(parse_bool).unwrap_or(false);
        let value = properties.remove("VALUE").map(parse_i32).unwrap_or(0);
        Self {
            default,
            netnotify,
            to_ini,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Keyboard {
    // KEYBOARD
    keyboard: String, // KEYBOARD
}

impl Keyboard {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let keyboard = properties.remove("KEYBOARD").unwrap_or_default();
        Self { keyboard }
    }
}

#[derive(Debug, Clone)]
pub struct Mouse {
    // MOUSE
    mouse: String, // MOUSE
    raw: i32,      // RAW
}

impl Mouse {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let mouse = properties.remove("MOUSE").unwrap_or_default();
        let raw = properties.remove("RAW").map(parse_i32).unwrap_or(0);
        Self { mouse, raw }
    }
}

#[derive(Debug, Clone)]
pub struct Random {
    // RAND
}

impl Random {
    pub fn new(_properties: HashMap<String, String>) -> Self {
        Self {}
    }
}

#[derive(Debug, Clone)]
pub struct Scene {
    // SCENE
    pub author: String,                  // AUTHOR
    pub background: String,              // BACKGROUND
    pub coauthors: String,               // COAUTHORS
    pub creation_time: DateTime<Utc>,    // CREATIONTIME
    pub deamon: bool,                    // DEAMON
    pub description: String,             // DESCRIPTION
    pub dlls: Vec<String>,               // DLLS
    pub last_modify_time: DateTime<Utc>, // LASTMODIFYTIME
    pub music: String,                   // MUSIC
    pub path: String,                    // PATH
    pub version: String,                 // VERSION
}

impl Scene {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let author = properties.remove("AUTHOR").unwrap_or_default();
        let background = properties.remove("BACKGROUND").unwrap_or_default();
        let coauthors = properties.remove("COAUTHORS").unwrap_or_default();
        let creation_time = properties
            .remove("CREATIONTIME")
            .map(parse_datetime)
            .unwrap_or_default();
        let deamon = properties.remove("DEAMON").map(parse_bool).unwrap_or(false);
        let description = properties.remove("DESCRIPTION").unwrap_or_default();
        let dlls = properties
            .remove("DLLS")
            .map(parse_comma_separated)
            .unwrap_or_default();
        let last_modify_time = properties
            .remove("LASTMODIFYTIME")
            .map(parse_datetime)
            .unwrap_or_default();
        let music = properties.remove("MUSIC").unwrap_or_default();
        let path = properties.remove("PATH").unwrap_or_default();
        let version = properties.remove("VERSION").unwrap_or_default();
        Self {
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Sequence {
    // SEQUENCE
    filename: String, // FILENAME
}

impl Sequence {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let filename = properties.remove("FILENAME").unwrap_or_default();
        Self { filename }
    }
}

#[derive(Debug, Clone)]
pub struct Sound {
    // SOUND
    filename: String,         // FILENAME
    flush_after_played: bool, // FLUSHAFTERPLAYED
    preload: bool,            // PRELOAD
}

impl Sound {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let filename = properties.remove("FILENAME").unwrap_or_default();
        let flush_after_played = properties
            .remove("FLUSHAFTERPLAYED")
            .map(parse_bool)
            .unwrap_or(false);
        let preload = properties
            .remove("PRELOAD")
            .map(parse_bool)
            .unwrap_or(false);
        Self {
            filename,
            flush_after_played,
            preload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Str {
    // STRING
    default: String, // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: String,   // VALUE
}

impl Str {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let default = properties.remove("DEFAULT").unwrap_or_default();
        let netnotify = properties
            .remove("NETNOTIFY")
            .map(parse_bool)
            .unwrap_or(false);
        let to_ini = properties.remove("TOINI").map(parse_bool).unwrap_or(false);
        let value = properties.remove("VALUE").unwrap_or_default();
        Self {
            default,
            netnotify,
            to_ini,
            value,
        }
    }
}

lazy_static! {
    static ref STRUCT_FIELDS_REGEX: Regex = Regex::new(r"^(\w+)<(\w+)>$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Struct {
    // STRUCT
    fields: Vec<(String, TypeName)>,
}

impl Struct {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let fields = properties
            .remove("FIELDS")
            .map(|s| {
                s.split(',')
                    .map(|f| {
                        let m = STRUCT_FIELDS_REGEX.captures(f).unwrap();
                        (m[1].to_owned(), m[2].to_owned())
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self { fields }
    }
}

#[derive(Debug, Clone)]
pub struct System {
    // SYSTEM
    system: String, // SYSTEM
}

impl System {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let system = properties.remove("SYSTEM").unwrap_or_default();
        Self { system }
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    // TEXT
    font: FontName,                // FONT
    horizontal_justify: bool,      // HJUSTIFY
    hypertext: bool,               // HYPERTEXT
    monitor_collision: bool,       // MONITORCOLLISION
    monitor_collision_alpha: bool, // MONITORCOLLISIONALPHA
    priority: i32,                 // PRIORITY
    rect: (i32, i32, i32, i32),    // RECT
    text: String,                  // TEXT
    to_canvas: bool,               // TOCANVAS
    visible: bool,                 // VISIBLE
    vertical_justify: bool,        // VJUSTIFY
}

impl Text {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let font = properties.remove("FONT").unwrap_or_default();
        let horizontal_justify = properties
            .remove("HJUSTIFY")
            .map(parse_bool)
            .unwrap_or(false);
        let hypertext = properties
            .remove("HYPERTEXT")
            .map(parse_bool)
            .unwrap_or(false);
        let monitor_collision = properties
            .remove("MONITORCOLLISION")
            .map(parse_bool)
            .unwrap_or(false);
        let monitor_collision_alpha = properties
            .remove("MONITORCOLLISIONALPHA")
            .map(parse_bool)
            .unwrap_or(false);
        let priority = properties.remove("PRIORITY").map(parse_i32).unwrap_or(0);
        let rect = properties
            .remove("RECT")
            .map(parse_rect)
            .unwrap_or_default();
        let text = properties.remove("TEXT").unwrap_or_default();
        let to_canvas = properties
            .remove("TOCANVAS")
            .map(parse_bool)
            .unwrap_or(false);
        let visible = properties
            .remove("VISIBLE")
            .map(parse_bool)
            .unwrap_or(false);
        let vertical_justify = properties
            .remove("VJUSTIFY")
            .map(parse_bool)
            .unwrap_or(false);
        Self {
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
        }
    }
}

#[derive(Debug, Clone)]
pub struct Timer {
    // TIMER
    elapse: i32,   // ELAPSE
    enabled: bool, // ENABLED
    ticks: i32,    // TICKS
}

impl Timer {
    pub fn new(mut properties: HashMap<String, String>) -> Self {
        let elapse = properties.remove("ELAPSE").map(parse_i32).unwrap_or(0);
        let enabled = properties
            .remove("ENABLED")
            .map(parse_bool)
            .unwrap_or(false);
        let ticks = properties.remove("TICKS").map(parse_i32).unwrap_or(0);
        Self {
            elapse,
            enabled,
            ticks,
        }
    }
}
