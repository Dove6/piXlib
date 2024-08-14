use bevy::{
    app::{App, Plugin, Update},
    input::ButtonInput,
    math::Vec2,
    prelude::{
        in_state, Camera, IntoSystemConfigs, KeyCode, MouseButton, NonSend, Query, Res, Transform,
        With,
    },
    time::Time,
    window::{PrimaryWindow, Window},
};
use pixlib_parser::runner::{KeyboardEvent, KeyboardKey, MouseEvent, TimerEvent};

use crate::{resources::ScriptRunner, states::AppState};

#[derive(Debug, Default)]
pub struct InputsPlugin;

impl Plugin for InputsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                queue_timer_input,
                queue_mouse_input,
                queue_keyboard_input,
                move_camera,
            )
                .run_if(in_state(AppState::SceneViewer)),
        );
    }
}

pub fn queue_timer_input(time: Res<Time>, runner: NonSend<ScriptRunner>) {
    let mut in_events = runner.events_in.timer.borrow_mut();
    in_events.push_back(TimerEvent::Elapsed {
        seconds: time.delta_seconds_f64(),
    });
}

pub fn queue_mouse_input(
    buttons: Res<ButtonInput<MouseButton>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    runner: NonSend<ScriptRunner>,
) {
    let mut in_events = runner.events_in.mouse.borrow_mut();
    let cursor_position = q_windows
        .single()
        .cursor_position()
        .unwrap_or(Vec2::new(0f32, 0f32));
    in_events.push_back(MouseEvent::MovedTo {
        x: cursor_position.x as isize,
        y: cursor_position.y as isize,
    });
    if buttons.just_pressed(MouseButton::Left) {
        in_events.push_back(MouseEvent::LeftButtonPressed);
    }
    if buttons.just_pressed(MouseButton::Right) {
        in_events.push_back(MouseEvent::RightButtonPressed);
    }
    if buttons.just_released(MouseButton::Left) {
        in_events.push_back(MouseEvent::LeftButtonReleased);
    }
    if buttons.just_released(MouseButton::Right) {
        in_events.push_back(MouseEvent::RightButtonReleased);
    }
}

// FIXME: remove (this is for debugging only)
pub fn move_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_transform: Query<(&mut Transform, &Camera)>,
) {
    let mut transform = camera_transform.single_mut().0;
    if keys.pressed(KeyCode::ArrowLeft) {
        transform.translation.x -= 2f32;
    }
    if keys.pressed(KeyCode::ArrowRight) {
        transform.translation.x += 2f32;
    }
    if keys.pressed(KeyCode::ArrowUp) {
        transform.translation.y -= 2f32;
    }
    if keys.pressed(KeyCode::ArrowDown) {
        transform.translation.y += 2f32;
    }
}

pub fn queue_keyboard_input(keys: Res<ButtonInput<KeyCode>>, runner: NonSend<ScriptRunner>) {
    let mut in_events = runner.events_in.keyboard.borrow_mut();
    for key in keys.get_just_pressed() {
        in_events.push_back(KeyboardEvent::KeyPressed {
            key_code: from_bevy_key_code(key),
        });
    }
}

fn from_bevy_key_code(key_code: &KeyCode) -> KeyboardKey {
    match key_code {
        KeyCode::Unidentified(_) => KeyboardKey::Unidentified,
        KeyCode::Backquote => KeyboardKey::Backquote,
        KeyCode::Backslash => KeyboardKey::Backslash,
        KeyCode::BracketLeft => KeyboardKey::BracketLeft,
        KeyCode::BracketRight => KeyboardKey::BracketRight,
        KeyCode::Comma => KeyboardKey::Comma,
        KeyCode::Digit0 => KeyboardKey::Digit0,
        KeyCode::Digit1 => KeyboardKey::Digit1,
        KeyCode::Digit2 => KeyboardKey::Digit2,
        KeyCode::Digit3 => KeyboardKey::Digit3,
        KeyCode::Digit4 => KeyboardKey::Digit4,
        KeyCode::Digit5 => KeyboardKey::Digit5,
        KeyCode::Digit6 => KeyboardKey::Digit6,
        KeyCode::Digit7 => KeyboardKey::Digit7,
        KeyCode::Digit8 => KeyboardKey::Digit8,
        KeyCode::Digit9 => KeyboardKey::Digit9,
        KeyCode::Equal => KeyboardKey::Equal,
        KeyCode::IntlBackslash => KeyboardKey::IntlBackslash,
        KeyCode::IntlRo => KeyboardKey::IntlRo,
        KeyCode::IntlYen => KeyboardKey::IntlYen,
        KeyCode::KeyA => KeyboardKey::KeyA,
        KeyCode::KeyB => KeyboardKey::KeyB,
        KeyCode::KeyC => KeyboardKey::KeyC,
        KeyCode::KeyD => KeyboardKey::KeyD,
        KeyCode::KeyE => KeyboardKey::KeyE,
        KeyCode::KeyF => KeyboardKey::KeyF,
        KeyCode::KeyG => KeyboardKey::KeyG,
        KeyCode::KeyH => KeyboardKey::KeyH,
        KeyCode::KeyI => KeyboardKey::KeyI,
        KeyCode::KeyJ => KeyboardKey::KeyJ,
        KeyCode::KeyK => KeyboardKey::KeyK,
        KeyCode::KeyL => KeyboardKey::KeyL,
        KeyCode::KeyM => KeyboardKey::KeyM,
        KeyCode::KeyN => KeyboardKey::KeyN,
        KeyCode::KeyO => KeyboardKey::KeyO,
        KeyCode::KeyP => KeyboardKey::KeyP,
        KeyCode::KeyQ => KeyboardKey::KeyQ,
        KeyCode::KeyR => KeyboardKey::KeyR,
        KeyCode::KeyS => KeyboardKey::KeyS,
        KeyCode::KeyT => KeyboardKey::KeyT,
        KeyCode::KeyU => KeyboardKey::KeyU,
        KeyCode::KeyV => KeyboardKey::KeyV,
        KeyCode::KeyW => KeyboardKey::KeyW,
        KeyCode::KeyX => KeyboardKey::KeyX,
        KeyCode::KeyY => KeyboardKey::KeyY,
        KeyCode::KeyZ => KeyboardKey::KeyZ,
        KeyCode::Minus => KeyboardKey::Minus,
        KeyCode::Period => KeyboardKey::Period,
        KeyCode::Quote => KeyboardKey::Quote,
        KeyCode::Semicolon => KeyboardKey::Semicolon,
        KeyCode::Slash => KeyboardKey::Slash,
        KeyCode::AltLeft => KeyboardKey::AltLeft,
        KeyCode::AltRight => KeyboardKey::AltRight,
        KeyCode::Backspace => KeyboardKey::Backspace,
        KeyCode::CapsLock => KeyboardKey::CapsLock,
        KeyCode::ContextMenu => KeyboardKey::ContextMenu,
        KeyCode::ControlLeft => KeyboardKey::ControlLeft,
        KeyCode::ControlRight => KeyboardKey::ControlRight,
        KeyCode::Enter => KeyboardKey::Enter,
        KeyCode::SuperLeft => KeyboardKey::MetaLeft,
        KeyCode::SuperRight => KeyboardKey::MetaRight,
        KeyCode::ShiftLeft => KeyboardKey::ShiftLeft,
        KeyCode::ShiftRight => KeyboardKey::ShiftRight,
        KeyCode::Space => KeyboardKey::Space,
        KeyCode::Tab => KeyboardKey::Tab,
        KeyCode::Convert => KeyboardKey::Convert,
        KeyCode::KanaMode => KeyboardKey::KanaMode,
        KeyCode::Lang1 => KeyboardKey::Lang1,
        KeyCode::Lang2 => KeyboardKey::Lang2,
        KeyCode::Lang3 => KeyboardKey::Lang3,
        KeyCode::Lang4 => KeyboardKey::Lang4,
        KeyCode::Lang5 => KeyboardKey::Lang5,
        KeyCode::NonConvert => KeyboardKey::NonConvert,
        KeyCode::Delete => KeyboardKey::Delete,
        KeyCode::End => KeyboardKey::End,
        KeyCode::Help => KeyboardKey::Help,
        KeyCode::Home => KeyboardKey::Home,
        KeyCode::Insert => KeyboardKey::Insert,
        KeyCode::PageDown => KeyboardKey::PageDown,
        KeyCode::PageUp => KeyboardKey::PageUp,
        KeyCode::ArrowDown => KeyboardKey::ArrowDown,
        KeyCode::ArrowLeft => KeyboardKey::ArrowLeft,
        KeyCode::ArrowRight => KeyboardKey::ArrowRight,
        KeyCode::ArrowUp => KeyboardKey::ArrowUp,
        KeyCode::NumLock => KeyboardKey::NumLock,
        KeyCode::Numpad0 => KeyboardKey::Numpad0,
        KeyCode::Numpad1 => KeyboardKey::Numpad1,
        KeyCode::Numpad2 => KeyboardKey::Numpad2,
        KeyCode::Numpad3 => KeyboardKey::Numpad3,
        KeyCode::Numpad4 => KeyboardKey::Numpad4,
        KeyCode::Numpad5 => KeyboardKey::Numpad5,
        KeyCode::Numpad6 => KeyboardKey::Numpad6,
        KeyCode::Numpad7 => KeyboardKey::Numpad7,
        KeyCode::Numpad8 => KeyboardKey::Numpad8,
        KeyCode::Numpad9 => KeyboardKey::Numpad9,
        KeyCode::NumpadAdd => KeyboardKey::NumpadAdd,
        KeyCode::NumpadBackspace => KeyboardKey::NumpadBackspace,
        KeyCode::NumpadClear => KeyboardKey::NumpadClear,
        KeyCode::NumpadClearEntry => KeyboardKey::NumpadClearEntry,
        KeyCode::NumpadComma => KeyboardKey::NumpadComma,
        KeyCode::NumpadDecimal => KeyboardKey::NumpadDecimal,
        KeyCode::NumpadDivide => KeyboardKey::NumpadDivide,
        KeyCode::NumpadEnter => KeyboardKey::NumpadEnter,
        KeyCode::NumpadEqual => KeyboardKey::NumpadEqual,
        KeyCode::NumpadHash => KeyboardKey::NumpadHash,
        KeyCode::NumpadMemoryAdd => KeyboardKey::NumpadMemoryAdd,
        KeyCode::NumpadMemoryClear => KeyboardKey::NumpadMemoryClear,
        KeyCode::NumpadMemoryRecall => KeyboardKey::NumpadMemoryRecall,
        KeyCode::NumpadMemoryStore => KeyboardKey::NumpadMemoryStore,
        KeyCode::NumpadMemorySubtract => KeyboardKey::NumpadMemorySubtract,
        KeyCode::NumpadMultiply => KeyboardKey::NumpadMultiply,
        KeyCode::NumpadParenLeft => KeyboardKey::NumpadParenLeft,
        KeyCode::NumpadParenRight => KeyboardKey::NumpadParenRight,
        KeyCode::NumpadStar => KeyboardKey::NumpadStar,
        KeyCode::NumpadSubtract => KeyboardKey::NumpadSubtract,
        KeyCode::Escape => KeyboardKey::Escape,
        KeyCode::Fn => KeyboardKey::Fn,
        KeyCode::FnLock => KeyboardKey::FnLock,
        KeyCode::PrintScreen => KeyboardKey::PrintScreen,
        KeyCode::ScrollLock => KeyboardKey::ScrollLock,
        KeyCode::Pause => KeyboardKey::Pause,
        KeyCode::BrowserBack => KeyboardKey::BrowserBack,
        KeyCode::BrowserFavorites => KeyboardKey::BrowserFavorites,
        KeyCode::BrowserForward => KeyboardKey::BrowserForward,
        KeyCode::BrowserHome => KeyboardKey::BrowserHome,
        KeyCode::BrowserRefresh => KeyboardKey::BrowserRefresh,
        KeyCode::BrowserSearch => KeyboardKey::BrowserSearch,
        KeyCode::BrowserStop => KeyboardKey::BrowserStop,
        KeyCode::Eject => KeyboardKey::Eject,
        KeyCode::LaunchApp1 => KeyboardKey::LaunchApp1,
        KeyCode::LaunchApp2 => KeyboardKey::LaunchApp2,
        KeyCode::LaunchMail => KeyboardKey::LaunchMail,
        KeyCode::MediaPlayPause => KeyboardKey::MediaPlayPause,
        KeyCode::MediaSelect => KeyboardKey::MediaSelect,
        KeyCode::MediaStop => KeyboardKey::MediaStop,
        KeyCode::MediaTrackNext => KeyboardKey::MediaTrackNext,
        KeyCode::MediaTrackPrevious => KeyboardKey::MediaTrackPrevious,
        KeyCode::Power => KeyboardKey::Power,
        KeyCode::Sleep => KeyboardKey::Sleep,
        KeyCode::AudioVolumeDown => KeyboardKey::AudioVolumeDown,
        KeyCode::AudioVolumeMute => KeyboardKey::AudioVolumeMute,
        KeyCode::AudioVolumeUp => KeyboardKey::AudioVolumeUp,
        KeyCode::WakeUp => KeyboardKey::WakeUp,
        KeyCode::Meta => KeyboardKey::Super,
        KeyCode::Hyper => KeyboardKey::Hyper,
        KeyCode::Turbo => KeyboardKey::Turbo,
        KeyCode::Abort => KeyboardKey::Abort,
        KeyCode::Resume => KeyboardKey::Resume,
        KeyCode::Suspend => KeyboardKey::Suspend,
        KeyCode::Again => KeyboardKey::Again,
        KeyCode::Copy => KeyboardKey::Copy,
        KeyCode::Cut => KeyboardKey::Cut,
        KeyCode::Find => KeyboardKey::Find,
        KeyCode::Open => KeyboardKey::Open,
        KeyCode::Paste => KeyboardKey::Paste,
        KeyCode::Props => KeyboardKey::Props,
        KeyCode::Select => KeyboardKey::Select,
        KeyCode::Undo => KeyboardKey::Undo,
        KeyCode::Hiragana => KeyboardKey::Hiragana,
        KeyCode::Katakana => KeyboardKey::Katakana,
        KeyCode::F1 => KeyboardKey::F1,
        KeyCode::F2 => KeyboardKey::F2,
        KeyCode::F3 => KeyboardKey::F3,
        KeyCode::F4 => KeyboardKey::F4,
        KeyCode::F5 => KeyboardKey::F5,
        KeyCode::F6 => KeyboardKey::F6,
        KeyCode::F7 => KeyboardKey::F7,
        KeyCode::F8 => KeyboardKey::F8,
        KeyCode::F9 => KeyboardKey::F9,
        KeyCode::F10 => KeyboardKey::F10,
        KeyCode::F11 => KeyboardKey::F11,
        KeyCode::F12 => KeyboardKey::F12,
        KeyCode::F13 => KeyboardKey::F13,
        KeyCode::F14 => KeyboardKey::F14,
        KeyCode::F15 => KeyboardKey::F15,
        KeyCode::F16 => KeyboardKey::F16,
        KeyCode::F17 => KeyboardKey::F17,
        KeyCode::F18 => KeyboardKey::F18,
        KeyCode::F19 => KeyboardKey::F19,
        KeyCode::F20 => KeyboardKey::F20,
        KeyCode::F21 => KeyboardKey::F21,
        KeyCode::F22 => KeyboardKey::F22,
        KeyCode::F23 => KeyboardKey::F23,
        KeyCode::F24 => KeyboardKey::F24,
        KeyCode::F25 => KeyboardKey::F25,
        KeyCode::F26 => KeyboardKey::F26,
        KeyCode::F27 => KeyboardKey::F27,
        KeyCode::F28 => KeyboardKey::F28,
        KeyCode::F29 => KeyboardKey::F29,
        KeyCode::F30 => KeyboardKey::F30,
        KeyCode::F31 => KeyboardKey::F31,
        KeyCode::F32 => KeyboardKey::F32,
        KeyCode::F33 => KeyboardKey::F33,
        KeyCode::F34 => KeyboardKey::F34,
        KeyCode::F35 => KeyboardKey::F35,
    }
}
