use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use crate::common::LoggableToOption;
use crate::filesystems::{DummyFileSystem, GameDirectory};
use crate::runner::*;
use chrono::Local;
use object::CnvObjectBuilder;
use test_case::test_case;
// use uptime_lib;

static CONDITION_PROPERTIES: &[(&str, &str)] =
    &[("OPERAND1", "4"), ("OPERAND2", "5"), ("OPERATOR", "EQUAL")];

static COMPLEX_CONDITION_PROPERTIES: &[(&str, &str)] = &[
    ("CONDITION1", "HELPER_COND1"),
    ("CONDITION2", "HELPER_COND2"),
    ("OPERATOR", "AND"),
];
static ANIMATION_PROPERTIES: &[(&str, &str)] = &[("FILENAME", "TEST.ANN"), ("PRELOAD", "TRUE")];
static BEHAVIOR_PROPERTIES: &[(&str, &str)] = &[("CODE", "{}")];
static IMAGE_PROPERTIES: &[(&str, &str)] = &[("FILENAME", "TEST.IMG"), ("PRELOAD", "TRUE")];
static SOUND_PROPERTIES: &[(&str, &str)] = &[("FILENAME", "TEST.WAV")];
static SEQUENCE_PROPERTIES: &[(&str, &str)] = &[("FILENAME", "TEST.SEQ")];

#[test_case("ANIMO", ANIMATION_PROPERTIES)]
#[test_case("APPLICATION", &[])]
#[test_case("ARRAY", &[])]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES)]
#[test_case("BOOL", &[])]
#[test_case("BUTTON", &[])]
#[test_case("CANVAS_OBSERVER", &[])]
// #[test_case("CLASS", &[])]
#[test_case("CNVLOADER", &[])]
#[test_case("COMPLEXCONDITION", COMPLEX_CONDITION_PROPERTIES)]
#[test_case("CONDITION", CONDITION_PROPERTIES)]
// #[test_case("DATABASE", &[])]
#[test_case("DOUBLE", &[])]
#[test_case("EPISODE", &[])]
// #[test_case("FILTER", &[])]
#[test_case("FONT", &[])]
#[test_case("GROUP", &[])]
#[test_case("IMAGE", IMAGE_PROPERTIES)]
// #[test_case("INERTIA", &[])]
#[test_case("INTEGER", &[])]
#[test_case("KEYBOARD", &[])]
// #[test_case("MATRIX", &[])]
#[test_case("MOUSE", &[])]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")])]
#[test_case("MUSIC", &[])]
// #[test_case("PATTERN", &[])]
#[test_case("RAND", &[])]
#[test_case("SCENE", &[])]
#[test_case("SEQUENCE", SEQUENCE_PROPERTIES)]
#[test_case("SOUND", SOUND_PROPERTIES)]
// #[test_case("STATICFILTER", &[])]
#[test_case("STRING", &[])]
// #[test_case("STRUCT", &[])]
#[test_case("SYSTEM", &[])]
#[test_case("TEXT", &[])]
#[test_case("TIMER", &[])]
// #[test_case("VECTOR", &[])]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[])]
// #[test_case("WORLD", &[])]
#[ignore = "To be run separately"]
fn ensure_object_type_can_be_created(object_type: &str, object_properties: &[(&str, &str)]) {
    env_logger::try_init().ok_or_warn();

    let runner = CnvRunner::try_new(
        Arc::new(RwLock::new(DummyFileSystem)),
        Default::default(),
        (800, 600),
    )
    .unwrap();
    let test_script = Arc::new(CnvScript::new(
        Arc::clone(&runner),
        ScenePath {
            dir_path: ".".into(),
            file_path: "__TEST__".into(),
        },
        None,
        ScriptSource::Root,
    ));
    let mut object_properties = Vec::from(object_properties);
    object_properties.push(("TYPE", object_type));
    let object_name = String::from("TEST_") + object_type;

    create_object(&test_script, &object_name, &object_properties).expect("Could not create object");
}

#[test_case("ANIMO", ANIMATION_PROPERTIES, "GETCENTERX", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "GETCENTERY", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "GETCFRAMEINEVENT", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETCURRFRAMEPOSX", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETCURRFRAMEPOSY", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETENDX", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETENDY", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETEVENTNAME", &[], CnvValue::String(String::from("")))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETFRAME", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETFRAMENAME", &[], CnvValue::String(String::from("")))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETHEIGHT", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETMAXWIDTH", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETNOE", &[], CnvValue::Integer(0))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETNOF", &[], CnvValue::Integer(0))]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "GETNOFINEVENT", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETOPACITY", &[], CnvValue::Integer(0))]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "GETPOSITIONX", &[], CnvValue::Integer(10))]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "GETPOSITIONY", &[], CnvValue::Integer(10))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETPRIORITY", &[], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "GETWIDTH", &[], CnvValue::Integer(0))]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "HIDE", &[], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "INVALIDATE", &[], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "ISAT", &[], CnvValue::Bool(false))]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "ISNEAR", &[CnvValue::String(String::from("HELPER_IMG")), CnvValue::Integer(0)], CnvValue::Bool(true))]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "ISNEAR", &[CnvValue::String(String::from("HELPER_IMG")), CnvValue::Integer(100)], CnvValue::Bool(false))]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "ISPLAYING", &[], CnvValue::Bool(false))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "ISVISIBLE", &[], CnvValue::Bool(false))]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "LOAD", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "MERGEALPHA", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "MONITORCOLLISION", &[CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "MOVE", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "NEXT", &[], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "NEXTFRAME", &[], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "NPLAY", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "PAUSE", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "PLAY", &[CnvValue::String(String::from("MAIN"))], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "PREVFRAME", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "REMOVEMONITORCOLLISION", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "RESUME", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SETANCHOR", &[CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SETASBUTTON", &[CnvValue::Bool(false), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "SETBACKWARD", &[], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "SETCLIPPING", &[], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "SETFORWARD", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SETFPS", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SETFRAME", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SETFRAME", &[CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "SETFRAMENAME", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("ANIMO", ANIMATION_PROPERTIES, "SETOPACITY", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SETPOSITION", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SETPRIORITY", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "SHOW", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "STOP", &[], CnvValue::Null)]
#[test_case("ANIMO", ANIMATION_PROPERTIES, "STOP", &[CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("APPLICATION", &[], "EXIT", &[], CnvValue::Null)]
#[test_case("APPLICATION", &[], "GETLANGUAGE", &[], CnvValue::String(String::from("")))]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from(""))], CnvValue::Integer(0))]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from(""))], CnvValue::String(String::from("")))]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from(""))], CnvValue::Double(0.0))]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from(""))], CnvValue::Bool(false))]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from("")), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from("")), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("APPLICATION", &[], "RUN", &[CnvValue::String(String::from("")), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("APPLICATION", &[], "RUNENV", &[], CnvValue::Null)]
#[test_case("APPLICATION", &[], "SETLANGUAGE", &[CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("ARRAY", &[], "ADD", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "ADDAT", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("ARRAY", &[], "ADDAT", &[CnvValue::Integer(0), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("ARRAY", &[], "ADDAT", &[CnvValue::Integer(0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("ARRAY", &[], "ADDAT", &[CnvValue::Integer(0), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("ARRAY", &[], "CHANGEAT", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "CLAMPAT", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "CONTAINS", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "COPYTO", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "FIND", &[], CnvValue::Null)]
#[test_case("ARRAY", &[], "GET", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("ARRAY", &[], "GETSIZE", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "GETSUMVALUE", &[], CnvValue::Null)]
#[test_case("ARRAY", &[], "INSERTAT", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("ARRAY", &[], "INSERTAT", &[CnvValue::Integer(0), CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("ARRAY", &[], "INSERTAT", &[CnvValue::Integer(0), CnvValue::Double(0.0)], CnvValue::Null)]
#[test_case("ARRAY", &[], "INSERTAT", &[CnvValue::Integer(0), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("ARRAY", &[], "LOAD", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "LOADINI", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "MODAT", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "MULAT", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "REMOVE", &[], CnvValue::Null)]
#[test_case("ARRAY", &[], "REMOVEALL", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "REMOVEAT", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "REVERSEFIND", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "SAVE", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "SAVEINI", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "SUB", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "SUBAT", &[], CnvValue::Null)]
// #[test_case("ARRAY", &[], "SUM", &[], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[], CnvValue::Null)]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[], CnvValue::Integer(0))]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[], CnvValue::String(String::from("")))]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[], CnvValue::Double(0.0))]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[], CnvValue::Bool(false))]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[CnvValue::Double(0.0)], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUN", &[CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[], CnvValue::Null)]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[], CnvValue::Integer(0))]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[], CnvValue::String(String::from("")))]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[], CnvValue::Double(0.0))]
// #[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[], CnvValue::Bool(false))]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[CnvValue::Double(0.0)], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNC", &[CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNLOOPED", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("BEHAVIOUR", BEHAVIOR_PROPERTIES, "RUNLOOPED", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(1)], CnvValue::Null)]
// #[test_case("BOOL", &[], "SET", &[CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("BOOL", &[], "SWITCH", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("BOOL", &[], "SWITCH", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("BOOL", &[], "SWITCH", &[CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("BOOL", &[], "SWITCH", &[CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("BUTTON", &[], "DISABLE", &[], CnvValue::Null)]
#[test_case("BUTTON", &[], "DISABLEBUTVISIBLE", &[], CnvValue::Null)]
#[test_case("BUTTON", &[], "ENABLE", &[], CnvValue::Null)]
// #[test_case("BUTTON", &[], "GETSTD", &[], CnvValue::Null)]
// #[test_case("BUTTON", &[], "SETONCLICK", &[], CnvValue::Null)]
// #[test_case("BUTTON", &[], "SETONMOVE", &[], CnvValue::Null)]
#[test_case("BUTTON", &[], "SETPRIORITY", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("BUTTON", &[], "SETRECT", &[CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("BUTTON", &[], "SETRECT", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("BUTTON", &[], "SETSTD", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "ADD", &[], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "ENABLENOTIFY", &[], CnvValue::Null)]
#[test_case("CANVAS_OBSERVER", &[], "GETGRAPHICSAT", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Bool(false), CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Bool(false)], CnvValue::String(String::from("")))]
// #[test_case("CANVAS_OBSERVER", &[], "MOVEBKG", &[], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "PASTE", &[], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "REDRAW", &[], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "REFRESH", &[], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "REMOVE", &[], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "SAVE", &[], CnvValue::Null)]
#[test_case("CANVAS_OBSERVER", &[], "SETBACKGROUND", &[CnvValue::String(String::from("TEST.IMG"))], CnvValue::Null)]
// #[test_case("CANVAS_OBSERVER", &[], "SETBKGPOS", &[], CnvValue::Null)]
// #[test_case("CLASS", &[], "NEW", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("CLASS", &[], "NEW", &[CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("CLASS", &[], "NEW", &[CnvValue::String(String::from("")), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("CLASS", &[], "NEW", &[CnvValue::String(String::from("")), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("CLASS", &[], "NEW", &[CnvValue::String(String::from("")), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("CNVLOADER", &[], "LOAD", &[], CnvValue::Null)]
// #[test_case("CNVLOADER", &[], "RELEASE", &[], CnvValue::Null)]
// #[test_case("COMPLEXCONDITION", COMPLEX_CONDITION_PROPERTIES, "BREAK", &[], CnvValue::Null)]
// #[test_case("COMPLEXCONDITION", COMPLEX_CONDITION_PROPERTIES, "CHECK", &[CnvValue::Bool(false)], CnvValue::Bool(false))]
#[test_case("COMPLEXCONDITION", COMPLEX_CONDITION_PROPERTIES, "ONE_BREAK", &[], CnvValue::Null)]
// #[test_case("CONDITION", CONDITION_PROPERTIES, "BREAK", &[CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("CONDITION", CONDITION_PROPERTIES, "CHECK", &[CnvValue::Bool(false)], CnvValue::Bool(false))]
// #[test_case("CONDITION", CONDITION_PROPERTIES, "ONE_BREAK", &[CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("DATABASE", &[], "ADD", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("DATABASE", &[], "FIND", &[CnvValue::String(String::from("")), CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("DATABASE", &[], "FIND", &[CnvValue::String(String::from("")), CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("DATABASE", &[], "FIND", &[CnvValue::String(String::from("")), CnvValue::Double(0.0), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("DATABASE", &[], "FIND", &[CnvValue::String(String::from("")), CnvValue::Bool(false), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("DATABASE", &[], "GETCURSORPOS", &[], CnvValue::Integer(0))]
// #[test_case("DATABASE", &[], "GETROWSNO", &[], CnvValue::Integer(0))]
// #[test_case("DATABASE", &[], "LOAD", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("DATABASE", &[], "NEXT", &[], CnvValue::Null)]
// #[test_case("DATABASE", &[], "REMOVEALL", &[], CnvValue::Null)]
// #[test_case("DATABASE", &[], "SAVE", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("DATABASE", &[], "SELECT", &[CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("DOUBLE", &[], "ADD", &[CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "ARCTAN", &[CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "ARCTANEX", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "ARCTANEX", &[CnvValue::Double(0.0), CnvValue::Double(0.0), CnvValue::Integer(0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "CLAMP", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "COSINUS", &[CnvValue::Double(0.0)], CnvValue::Double(0.0))]
#[test_case("DOUBLE", &[], "DIV", &[CnvValue::Double(1.0)], CnvValue::Null)]
// #[test_case("DOUBLE", &[], "LENGTH", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "MAXA", &[CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "MAXA", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "MAXA", &[CnvValue::Double(0.0), CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "MINA", &[CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "MINA", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "MINA", &[CnvValue::Double(0.0), CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "MUL", &[CnvValue::Double(0.0)], CnvValue::Null)]
#[test_case("DOUBLE", &[], "SET", &[CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("DOUBLE", &[], "SINUS", &[CnvValue::Double(0.0)], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "SQRT", &[], CnvValue::Double(0.0))]
// #[test_case("DOUBLE", &[], "SUB", &[CnvValue::Double(0.0)], CnvValue::Double(0.0))]
#[test_case("EPISODE", &[], "BACK", &[], CnvValue::Null)]
// #[test_case("EPISODE", &[], "GETCURRENTSCENE", &[], CnvValue::String(String::from("")))]
// #[test_case("EPISODE", &[], "GETLATESTSCENE", &[], CnvValue::String(String::from("")))]
#[test_case("EPISODE", &[], "GOTO", &[CnvValue::String(String::from("HELPER_SCENE"))], CnvValue::Null)]
#[test_case("GROUP", &[], "ADD", &[CnvValue::String(String::from("HELPER_IMG"))], CnvValue::Null)]
// #[test_case("GROUP", &[], "ADDCLONES", &[], CnvValue::Null)]
// #[test_case("GROUP", &[], "GETSIZE", &[], CnvValue::Null)]
// #[test_case("GROUP", &[], "NEXT", &[], CnvValue::Null)]
// #[test_case("GROUP", &[], "PREV", &[], CnvValue::Null)]
#[test_case("GROUP", &[], "REMOVE", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("GROUP", &[], "REMOVEALL", &[], CnvValue::Null)]
// #[test_case("GROUP", &[], "RESETMARKER", &[], CnvValue::Null)]
// #[test_case("GROUP", &[], "SETMARKERPOS", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "GETALPHA", &[], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "GETHEIGHT", &[], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "GETPIXEL", &[], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "GETPOSITIONX", &[], CnvValue::Null)]
#[test_case("IMAGE", IMAGE_PROPERTIES, "GETPOSITIONY", &[], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "GETWIDTH", &[], CnvValue::Null)]
#[test_case("IMAGE", IMAGE_PROPERTIES, "HIDE", &[], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "INVALIDATE", &[], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "ISVISIBLE", &[], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "LOAD", &[], CnvValue::Null)]
#[test_case("IMAGE", IMAGE_PROPERTIES, "MERGEALPHA", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("IMAGE", IMAGE_PROPERTIES, "MOVE", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "SETASBUTTON", &[], CnvValue::Null)]
#[test_case("IMAGE", IMAGE_PROPERTIES, "SETCLIPPING", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "SETOPACITY", &[], CnvValue::Null)]
#[test_case("IMAGE", IMAGE_PROPERTIES, "SETPOSITION", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("IMAGE", IMAGE_PROPERTIES, "SETPRIORITY", &[], CnvValue::Null)]
#[test_case("IMAGE", IMAGE_PROPERTIES, "SHOW", &[], CnvValue::Null)]
// #[test_case("INERTIA", &[], "ADDFORCE", &[CnvValue::Integer(0), CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("INERTIA", &[], "CREATESPHERE", &[CnvValue::Double(0.0), CnvValue::Double(0.0), CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Integer(0))]
// #[test_case("INERTIA", &[], "DELETEBODY", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("INERTIA", &[], "GETPOSITIONX", &[CnvValue::Integer(0)], CnvValue::Double(0.0))]
// #[test_case("INERTIA", &[], "GETPOSITIONY", &[CnvValue::Integer(0)], CnvValue::Double(0.0))]
// #[test_case("INERTIA", &[], "GETSPEED", &[CnvValue::Integer(0)], CnvValue::Double(0.0))]
// #[test_case("INERTIA", &[], "LINK", &[CnvValue::Integer(0), CnvValue::String(String::from("")), CnvValue::Bool(false), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("INERTIA", &[], "LOAD", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("INERTIA", &[], "RESETTIMER", &[], CnvValue::Null)]
// #[test_case("INERTIA", &[], "SETGRAVITY", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("INERTIA", &[], "SETLINEARDAMPING", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("INERTIA", &[], "SETMATERIAL", &[CnvValue::Integer(0), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("INERTIA", &[], "SETPOSITION", &[CnvValue::Integer(0), CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("INERTIA", &[], "SETVELOCITY", &[CnvValue::Integer(0), CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("INERTIA", &[], "TICK", &[], CnvValue::Null)]
// #[test_case("INERTIA", &[], "UNLINK", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("INTEGER", &[], "ABS", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
#[test_case("INTEGER", &[], "ADD", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("INTEGER", &[], "AND", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
#[test_case("INTEGER", &[], "CLAMP", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
#[test_case("INTEGER", &[], "DEC", &[], CnvValue::Null)]
#[test_case("INTEGER", &[], "DIV", &[CnvValue::Integer(1)], CnvValue::Null)]
#[test_case("INTEGER", &[], "INC", &[], CnvValue::Null)]
// #[test_case("INTEGER", &[], "LENGTH", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("INTEGER", &[], "MOD", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("INTEGER", &[], "MUL", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("INTEGER", &[], "OR", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("INTEGER", &[], "RANDOM", &[CnvValue::Integer(1)], CnvValue::Integer(0))]
// #[test_case("INTEGER", &[], "RANDOM", &[CnvValue::Integer(10), CnvValue::Integer(1)], CnvValue::Integer(10))]
#[test_case("INTEGER", &[], "RESETINI", &[], CnvValue::Null)]
#[test_case("INTEGER", &[], "SET", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("INTEGER", &[], "SUB", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
#[test_case("INTEGER", &[], "SWITCH", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("KEYBOARD", &[], "DISABLE", &[], CnvValue::Null)]
// #[test_case("KEYBOARD", &[], "ENABLE", &[], CnvValue::Null)]
// #[test_case("KEYBOARD", &[], "GETLATESTKEY", &[], CnvValue::Null)]
// #[test_case("KEYBOARD", &[], "ISENABLED", &[], CnvValue::Null)]
// #[test_case("KEYBOARD", &[], "ISKEYDOWN", &[], CnvValue::Null)]
// #[test_case("KEYBOARD", &[], "SETAUTOREPEAT", &[], CnvValue::Null)]
// #[test_case("MATRIX", &[], "CALCENEMYMOVEDEST", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "CALCENEMYMOVEDIR", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "CANHEROGOTO", &[CnvValue::Integer(0)], CnvValue::Bool(false))]
// #[test_case("MATRIX", &[], "GET", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "GETCELLOFFSET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "GETCELLPOSX", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "GETCELLPOSY", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "GETCELLSNO", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "GETFIELDPOSX", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "GETFIELDPOSY", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "GETOFFSET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "ISGATEEMPTY", &[], CnvValue::Bool(false))]
// #[test_case("MATRIX", &[], "ISINGATE", &[CnvValue::Integer(0)], CnvValue::Bool(false))]
// #[test_case("MATRIX", &[], "MOVE", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("MATRIX", &[], "NEXT", &[], CnvValue::Integer(0))]
// #[test_case("MATRIX", &[], "SET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("MATRIX", &[], "SETGATE", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("MATRIX", &[], "SETROW", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("MATRIX", &[], "SETROW", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("MATRIX", &[], "SETROW", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("MATRIX", &[], "TICK", &[], CnvValue::Null)]
#[test_case("MOUSE", &[], "DISABLE", &[], CnvValue::Null)]
// #[test_case("MOUSE", &[], "DISABLESIGNAL", &[], CnvValue::Null)]
#[test_case("MOUSE", &[], "ENABLE", &[], CnvValue::Null)]
// #[test_case("MOUSE", &[], "ENABLESIGNAL", &[], CnvValue::Null)]
#[test_case("MOUSE", &[], "GETPOSX", &[], CnvValue::Integer(0))]
#[test_case("MOUSE", &[], "GETPOSY", &[], CnvValue::Integer(0))]
#[test_case("MOUSE", &[], "HIDE", &[], CnvValue::Null)]
// #[test_case("MOUSE", &[], "ISLBUTTONDOWN", &[], CnvValue::Null)]
// #[test_case("MOUSE", &[], "SET", &[], CnvValue::Null)]
// #[test_case("MOUSE", &[], "SETCLIPRECT", &[], CnvValue::Null)]
// #[test_case("MOUSE", &[], "SETPOSITION", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("MOUSE", &[], "SHOW", &[], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "GET", &[CnvValue::Integer(0)], CnvValue::Integer(0))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "GET", &[CnvValue::Integer(0)], CnvValue::String(String::from("")))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "GET", &[CnvValue::Integer(0)], CnvValue::Double(0.0))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "GET", &[CnvValue::Integer(0)], CnvValue::Bool(false))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "GET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Integer(0))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "GET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::String(String::from("")))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "GET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Double(0.0))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "GET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Bool(false))]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "SET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "SET", &[CnvValue::Integer(0), CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "SET", &[CnvValue::Integer(0), CnvValue::Double(0.0)], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "1")], "SET", &[CnvValue::Integer(0), CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "SET", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "SET", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "SET", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Double(0.0)], CnvValue::Null)]
#[test_case("MULTIARRAY", &[("DIMENSIONS", "2")], "SET", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("MUSIC", &[], "PLAY", &[], CnvValue::Null)]
// #[test_case("PATTERN", &[], "ADD", &[CnvValue::String(String::from("")), CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("PATTERN", &[], "GETGRAPHICSAT", &[CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Bool(false), CnvValue::Bool(false), CnvValue::Integer(0)], CnvValue::String(String::from("")))]
// #[test_case("PATTERN", &[], "MOVE", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("RAND", &[], "GET", &[CnvValue::Integer(1)], CnvValue::Integer(0))]
#[test_case("RAND", &[], "GET", &[CnvValue::Integer(10), CnvValue::Integer(1)], CnvValue::Integer(10))]
// #[test_case("RAND", &[], "GETPLENTY", &[CnvValue::String(String::from("")), CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Integer(0), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("SCENE", &[], "GETMAXHSPRIORITY", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "GETMINHSPRIORITY", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "GETPLAYINGANIMO", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "GETPLAYINGSEQ", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "PAUSE", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "REMOVECLONES", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "RESUME", &[], CnvValue::Null)]
#[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN"))], CnvValue::Null)]
// #[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN"))], CnvValue::Integer(0))]
// #[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN"))], CnvValue::String(String::from("")))]
// #[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN"))], CnvValue::Double(0.0))]
// #[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN"))], CnvValue::Bool(false))]
#[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN")), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN")), CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN")), CnvValue::Double(0.0)], CnvValue::Null)]
#[test_case("SCENE", &[], "RUN", &[CnvValue::String(String::from("HELPER_BEH")), CnvValue::String(String::from("RUN")), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("SCENE", &[], "RUNCLONES", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "SETMAXHSPRIORITY", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "SETMINHSPRIORITY", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "SETMUSICVOLUME", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("SCENE", &[], "STARTMUSIC", &[], CnvValue::Null)]
// #[test_case("SCENE", &[], "STOPMUSIC", &[], CnvValue::Null)]
// #[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "GETEVENTNAME", &[], CnvValue::String(String::from("")))]
#[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "HIDE", &[], CnvValue::Null)]
#[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "ISPLAYING", &[], CnvValue::Bool(false))]
// #[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "PAUSE", &[], CnvValue::Null)]
#[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "PLAY", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "RESUME", &[], CnvValue::Null)]
#[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "STOP", &[], CnvValue::Null)]
#[test_case("SEQUENCE", SEQUENCE_PROPERTIES, "STOP", &[CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("SOUND", SOUND_PROPERTIES, "ISPLAYING", &[], CnvValue::Bool(false))]
// #[test_case("SOUND", SOUND_PROPERTIES, "LOAD", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("SOUND", SOUND_PROPERTIES, "PAUSE", &[], CnvValue::Null)]
#[test_case("SOUND", SOUND_PROPERTIES, "PLAY", &[], CnvValue::Null)]
// #[test_case("SOUND", SOUND_PROPERTIES, "RESUME", &[], CnvValue::Null)]
// #[test_case("SOUND", SOUND_PROPERTIES, "SETVOLUME", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("SOUND", SOUND_PROPERTIES, "STOP", &[], CnvValue::Null)]
// #[test_case("STATICFILTER", &[], "LINK", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("STATICFILTER", &[], "SETPROPERTY", &[CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("STATICFILTER", &[], "SETPROPERTY", &[CnvValue::String(String::from("")), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("STATICFILTER", &[], "SETPROPERTY", &[CnvValue::String(String::from("")), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("STATICFILTER", &[], "SETPROPERTY", &[CnvValue::String(String::from("")), CnvValue::Bool(false)], CnvValue::Null)]
// #[test_case("STATICFILTER", &[], "UNLINK", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("STRING", &[], "ADD", &[CnvValue::String(String::from(""))], CnvValue::String(String::from("")))]
// #[test_case("STRING", &[], "COPYFILE", &[CnvValue::String(String::from("")), CnvValue::String(String::from(""))], CnvValue::Bool(false))]
// #[test_case("STRING", &[], "CUT", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
#[test_case("STRING", &[], "FIND", &[CnvValue::String(String::from(""))], CnvValue::Integer(0))]
#[test_case("STRING", &[], "FIND", &[CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Integer(0))]
// #[test_case("STRING", &[], "GET", &[CnvValue::Integer(0)], CnvValue::String(String::from("")))]
#[test_case("STRING", &[], "GET", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::String(String::from("")))]
// #[test_case("STRING", &[], "LENGTH", &[], CnvValue::Integer(0))]
// #[test_case("STRING", &[], "REPLACE", &[CnvValue::String(String::from("")), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("STRING", &[], "REPLACEAT", &[CnvValue::Integer(0), CnvValue::String(String::from(""))], CnvValue::Null)]
#[test_case("STRING", &[], "RESETINI", &[], CnvValue::Null)]
#[test_case("STRING", &[], "SET", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("STRING", &[], "SUB", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("STRING", &[], "UPPER", &[], CnvValue::Null)]
// #[test_case("STRUCT", &[], "GETFIELD", &[CnvValue::String(String::from(""))], CnvValue::Integer(0))]
// #[test_case("STRUCT", &[], "GETFIELD", &[CnvValue::String(String::from(""))], CnvValue::String(String::from("")))]
// #[test_case("STRUCT", &[], "GETFIELD", &[CnvValue::String(String::from(""))], CnvValue::Double(0.0))]
// #[test_case("STRUCT", &[], "GETFIELD", &[CnvValue::String(String::from(""))], CnvValue::Bool(false))]
// #[test_case("STRUCT", &[], "SET", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("STRUCT", &[], "SETFIELD", &[CnvValue::String(String::from("")), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("STRUCT", &[], "SETFIELD", &[CnvValue::String(String::from("")), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("STRUCT", &[], "SETFIELD", &[CnvValue::String(String::from("")), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("STRUCT", &[], "SETFIELD", &[CnvValue::String(String::from("")), CnvValue::Bool(false)], CnvValue::Null)]
#[test_case("SYSTEM", &[], "GETDATE", &[], CnvValue::String(Local::now().format("%y%m%d").to_string()))]
// #[test_case("SYSTEM", &[], "GETMHZ", &[], CnvValue::Integer(0))]
// #[test_case("SYSTEM", &[], "GETMINUTES", &[], CnvValue::Integer(0))]
// #[test_case("SYSTEM", &[], "GETSECONDS", &[], CnvValue::Integer(0))]
// #[test_case("SYSTEM", &[], "GETSYSTEMTIME", &[], CnvValue::Integer(uptime_lib::get().unwrap().as_millis() as i32))]
// #[test_case("TEXT", &[], "HIDE", &[], CnvValue::Null)]
// #[test_case("TEXT", &[], "SETCOLOR", &[], CnvValue::Null)]
// #[test_case("TEXT", &[], "SETJUSTIFY", &[], CnvValue::Null)]
// #[test_case("TEXT", &[], "SETPOSITION", &[], CnvValue::Null)]
#[test_case("TEXT", &[], "SETTEXT", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("TEXT", &[], "SHOW", &[], CnvValue::Null)]
#[test_case("TIMER", &[], "DISABLE", &[], CnvValue::Null)]
#[test_case("TIMER", &[], "ENABLE", &[], CnvValue::Null)]
// #[test_case("TIMER", &[], "GETTICKS", &[], CnvValue::Integer(0))]
#[test_case("TIMER", &[], "RESET", &[], CnvValue::Null)]
#[test_case("TIMER", &[], "SET", &[CnvValue::Integer(1)], CnvValue::Null)]
// #[test_case("TIMER", &[], "SETELAPSE", &[CnvValue::Integer(100)], CnvValue::Null)]
// #[test_case("VECTOR", &[], "ADD", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("VECTOR", &[], "ASSIGN", &[CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("VECTOR", &[], "ASSIGN", &[CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("VECTOR", &[], "ASSIGN", &[CnvValue::Double(0.0), CnvValue::Double(0.0), CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("VECTOR", &[], "GET", &[CnvValue::Integer(0)], CnvValue::Double(0.0))]
// #[test_case("VECTOR", &[], "LEN", &[], CnvValue::Double(0.0))]
// #[test_case("VECTOR", &[], "MUL", &[CnvValue::Double(0.0)], CnvValue::Null)]
// #[test_case("VECTOR", &[], "NORMALIZE", &[], CnvValue::Null)]
// #[test_case("VECTOR", &[], "REFLECT", &[CnvValue::String(String::from("")), CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "GETHEIGHT", &[], CnvValue::Integer(0))]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "GETPOSITIONX", &[], CnvValue::Integer(0))]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "GETPOSITIONY", &[], CnvValue::Integer(0))]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "GETWIDTH", &[], CnvValue::Integer(0))]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "MOVE", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "SETMASK", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "SETPOSITION", &[CnvValue::Integer(0), CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "SETPRIORITY", &[CnvValue::Integer(0)], CnvValue::Null)]
// #[test_case("VIRTUALGRAPHICSOBJECT", &[], "SETSOURCE", &[CnvValue::String(String::from(""))], CnvValue::Null)]
// #[test_case("WORLD", &[], "ADDBODY", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "ADDFORCE", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "ADDGRAVITYEX", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "FINDPATH", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "FOLLOWPATH", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETANGLE", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETBKGPOSX", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETBKGPOSY", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETMOVEDISTANCE", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETPOSITIONX", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETPOSITIONY", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETPOSITIONZ", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETROTATIONZ", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "GETSPEED", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "JOIN", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "LINK", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "LOAD", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "MOVEOBJECTS", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "REMOVEOBJECT", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETACTIVE", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETBKGSIZE", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETBODYDYNAMICS", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETG", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETGRAVITY", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETGRAVITYCENTER", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETLIMIT", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETMAXSPEED", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETMOVEFLAGS", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETPOSITION", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETREFOBJECT", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "SETVELOCITY", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "START", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "STOP", &[], CnvValue::Null)]
// #[test_case("WORLD", &[], "UNLINK", &[], CnvValue::Null)]
#[ignore = "To be run separately"]
fn ensure_method_is_implemented(
    object_type: &str,
    object_properties: &[(&str, &str)],
    method_name: &str,
    arguments: &[CnvValue],
    expected: CnvValue,
) {
    env_logger::try_init().ok_or_warn();
    let test_dir_path = PathBuf::from_iter([env!("CARGO_MANIFEST_DIR"), "src/tests/unit_assets"]);
    let filesystem = Arc::new(RwLock::new(
        GameDirectory::new(test_dir_path.to_str().unwrap()).unwrap(),
    ));
    let runner = CnvRunner::try_new(filesystem, Default::default(), (800, 600)).unwrap();

    let test_script = Arc::new(CnvScript::new(
        Arc::clone(&runner),
        ScenePath {
            dir_path: ".".into(),
            file_path: "__TEST__".into(),
        },
        None,
        ScriptSource::Root,
    ));
    let mut object_properties = Vec::from(object_properties);
    object_properties.push(("TYPE", object_type));
    let object_name = String::from("TEST_") + object_type;
    let object = create_object(&test_script, &object_name, &object_properties)
        .expect("Could not create object");
    test_script
        .add_object(object.clone())
        .expect("Error adding object");

    // Additional test objects
    test_script
        .add_object(
            create_object(&test_script.clone(), "HELPER_SCENE", &[("TYPE", "SCENE")])
                .expect("Could not create object"),
        )
        .expect("Error adding object");
    test_script
        .add_object(
            create_object(
                &test_script.clone(),
                "HELPER_IMG",
                &[
                    ("TYPE", "IMAGE"),
                    ("FILENAME", "HELPER.IMG"),
                    ("PRELOAD", "TRUE"),
                ],
            )
            .expect("Could not create object"),
        )
        .expect("Error adding object");
    test_script
        .add_object(
            create_object(
                &test_script.clone(),
                "HELPER_BEH",
                &[("TYPE", "BEHAVIOUR"), ("CODE", "{}")],
            )
            .expect("Could not create object"),
        )
        .expect("Error adding object");
    test_script
        .add_object(
            create_object(
                &test_script.clone(),
                "HELPER_COND1",
                &[
                    ("TYPE", "CONDITION"),
                    ("OPERAND1", "1"),
                    ("OPERAND2", "1"),
                    ("OPERATOR", "EQUAL"),
                ],
            )
            .expect("Could not create object"),
        )
        .expect("Error adding object");
    test_script
        .add_object(
            create_object(
                &test_script.clone(),
                "HELPER_COND2",
                &[
                    ("TYPE", "CONDITION"),
                    ("OPERAND1", "1"),
                    ("OPERAND2", "1"),
                    ("OPERATOR", "EQUAL"),
                ],
            )
            .expect("Could not create object"),
        )
        .expect("Error adding object");

    runner
        .scripts
        .borrow_mut()
        .push_script(test_script)
        .expect("Error adding script to runner");
    runner.init_objects().expect("Error initializing objects");

    let context = RunnerContext::new_minimal(&runner, &object);
    let result = object
        .call_method(
            CallableIdentifier::Method(method_name),
            arguments,
            Some(context),
        )
        .expect("Error running method");
    assert_eq!(result, expected);
}

fn create_object(
    parent: &Arc<CnvScript>,
    name: &str,
    properties: &[(&str, &str)],
) -> anyhow::Result<Arc<CnvObject>> {
    let mut builder = CnvObjectBuilder::new(parent.clone(), name.to_owned(), 0);
    for (property, value) in properties {
        builder
            .add_property((*property).to_owned(), (*value).to_owned())
            .into_result()?;
    }
    Ok(builder.build()?)
}
