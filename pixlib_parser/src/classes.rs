use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::ast::IgnorableProgram;

pub type EpisodeName = String;
pub type SceneName = String;
pub type ConditionName = String;
pub type ImageName = String;
pub type SoundName = String;
pub type VariableName = String;
pub type TypeName = String;
pub type FontName = String;

pub struct Animation {
    // ANIMO
    as_button: bool,               // ASBUTTON
    filename: PathBuf,             // FILENAME
    flush_after_played: bool,      // FLUSHAFTERPLAYED
    fps: i32,                      // FPS
    monitor_collision: bool,       // MONITORCOLLISION
    monitor_collision_alpha: bool, // MONITORCOLLISIONALPHA
    preload: bool,                 // PRELOAD
    priority: i32,                 // PRIORITY
    release: bool,                 // RELEASE
    to_canvas: bool,               // TOCANVAS
    visible: bool,                 // VISIBLE
}

pub struct Application {
    // APPLICATION
    author: String,                  // AUTHOR
    bloomoo_version: String,         // BLOOMOO_VERSION
    creation_time: DateTime<Utc>,    // CREATIONTIME
    description: String,             // DESCRIPTION
    episodes: Vec<EpisodeName>,      // EPISODES
    last_modify_time: DateTime<Utc>, // LASTMODIFYTIME
    path: PathBuf,                   // PATH
    scenes: Vec<SceneName>,          // SCENES
    start_with: EpisodeName,         // STARTWITH
    version: String,                 // VERSION
}

pub struct Array {
    // ARRAY
    send_on_change: bool, // SENDONCHANGE
}

pub struct Behavior {
    // BEHAVIOUR
    code: IgnorableProgram,   // CODE
    condition: ConditionName, // CONDITION
}

pub struct Bool {
    // BOOL
    default: bool,   // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: bool,     // VALUE
}

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

pub struct Condition {
    // CONDITION
    operand1: VariableName,      // OPERAND1
    operand2: VariableName,      // OPERAND2
    operator: ConditionOperator, // OPERATOR
}

pub struct ComplexCondition {
    // COMPLEXCONDITION
    operand1: ConditionName,     // OPERAND1
    operand2: ConditionName,     // OPERAND2
    operator: ConditionOperator, // OPERATOR
}

pub enum ConditionOperator {
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
}

pub struct Dbl {
    // DOUBLE
    default: f64,    // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: f64,      // VALUE
}

pub struct Episode {
    // EPISODE
    author: String,                  // AUTHOR
    creation_time: DateTime<Utc>,    // CREATIONTIME
    description: String,             // DESCRIPTION
    last_modify_time: DateTime<Utc>, // LASTMODIFYTIME
    path: PathBuf,                   // PATH
    scenes: Vec<SceneName>,          // SCENES
    start_with: SceneName,           // STARTWITH
    version: String,                 // VERSION
}

pub struct Expression {
    // EXPRESSION
    operand1: VariableName,       // OPERAND1
    operand2: VariableName,       // OPERAND2
    operator: ExpressionOperator, // OPERATOR
}

pub enum ExpressionOperator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

pub struct Font {
    // FONT
    family: String,
    style: String,
    size: usize,
}

pub struct Group {
    // GROUP
}

pub struct Image {
    // IMAGE
    as_button: bool,               // ASBUTTON
    filename: PathBuf,             // FILENAME
    flush_after_played: bool,      // FLUSHAFTERPLAYED
    monitor_collision: bool,       // MONITORCOLLISION
    monitor_collision_alpha: bool, // MONITORCOLLISIONALPHA
    preload: bool,                 // PRELOAD
    priority: i32,                 // PRIORITY
    release: bool,                 // RELEASE
    to_canvas: bool,               // TOCANVAS
    visible: bool,                 // VISIBLE
}

pub struct Int {
    // INTEGER
    default: i32,    // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: i32,      // VALUE
}

pub struct Keyboard {
    // KEYBOARD
    keyboard: String, // KEYBOARD
}

pub struct Mouse {
    // MOUSE
    mouse: String, // MOUSE
    raw: i32,      // RAW
}

pub struct Random {
    // RAND
}

pub struct Scene {
    // SCENE
    author: String,                  // AUTHOR
    background: PathBuf,             // BACKGROUND
    coauthors: String,               // COAUTHORS
    creation_time: DateTime<Utc>,    // CREATIONTIME
    deamon: bool,                    // DEAMON
    description: String,             // DESCRIPTION
    dlls: Vec<String>,               // DLLS
    last_modify_time: DateTime<Utc>, // LASTMODIFYTIME
    music: PathBuf,                  // MUSIC
    path: PathBuf,                   // PATH
    version: String,                 // VERSION
}

pub struct Sequence {
    // SEQUENCE
    filename: PathBuf, // FILENAME
}

pub struct Sound {
    // SOUND
    filename: PathBuf,        // FILENAME
    flush_after_played: bool, // FLUSHAFTERPLAYED
    preload: bool,            // PRELOAD
}

pub struct Str {
    // STRING
    default: String, // DEFAULT
    netnotify: bool, // NETNOTIFY
    to_ini: bool,    // TOINI
    value: String,   // VALUE
}

pub struct Struct {
    // STRUCT
    fields: Vec<(String, TypeName)>,
}

pub struct System {
    // SYSTEM
    system: String, // SYSTEM
}

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

pub struct Timer {
    // TIMER
    elapse: i32,   // ELAPSE
    enabled: bool, // ENABLED
    ticks: i32,    // TICKS
}
