use crate::web_socket::input_types::{Action, KeyboardRequest, MouseButton, MouseRequest};
use enigo::{
    Axis, Button, Coordinate, Direction,
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard, Mouse,
};
pub fn press_keys(enigo: &mut Enigo, request: &KeyboardRequest) {
    handle_modifiers(enigo, request, Press);

    if let Ok(key) = map_string_to_key(&request.key) {
        match enigo.key(key, Click) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error clicking key: {}", err);
            }
        }
    } else {
        eprintln!("Unrecognized key: {}", request.key);
    }

    handle_modifiers(enigo, request, Release);
}

fn handle_modifiers(enigo: &mut Enigo, request: &KeyboardRequest, direction: Direction) {
    for (modifier, is_pressed) in [
        (Key::Alt, request.modifiers.alt),
        (Key::Control, request.modifiers.ctrl),
        (Key::Meta, request.modifiers.meta),
        (Key::Shift, request.modifiers.shift),
    ] {
        if is_pressed {
            match enigo.key(modifier, direction) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("Error releasing key: {}", err);
                }
            }
        }
    }
}

fn map_string_to_key(key: &str) -> Result<Key, &'static str> {
    let key = key.to_lowercase();
    match key.as_str() {
        "caps_lock" => Ok(Key::CapsLock),
        "enter" | "retrun" => Ok(Key::Return),
        "meta" | "win" | "windows" => Ok(Key::Meta),
        "alt" => Ok(Key::Alt),
        "tab" => Ok(Key::Tab),
        "shift" => Ok(Key::Shift),
        "control" => Ok(Key::Control),
        "backspace" | "back" => Ok(Key::Backspace),
        "delete" | "del" => Ok(Key::Delete),
        "escape" | "esc" => Ok(Key::Escape),
        "space" => Ok(Key::Space),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "page_up" => Ok(Key::PageUp),
        "page_down" => Ok(Key::PageDown),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "insert" => Ok(Key::Insert),
        "print" | "print_screen" => Ok(Key::Print),
        // ScrollLock not supported?
        // "scroll_lock" => Ok(Key::ScrollLock),
        "pause" => Ok(Key::Pause),
        "media_play" | "media_pause" => Ok(Key::MediaPlayPause),
        "media_nex" => Ok(Key::MediaNextTrack),
        "media_prev" | "media_previous" => Ok(Key::MediaPrevTrack),
        "media_stop" => Ok(Key::MediaStop),
        "volume_up" => Ok(Key::VolumeUp),
        "volume_down" => Ok(Key::VolumeDown),
        "volume_mute" => Ok(Key::VolumeMute),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        "f13" => Ok(Key::F13),
        "f14" => Ok(Key::F14),
        "f15" => Ok(Key::F15),
        "f16" => Ok(Key::F16),
        "f17" => Ok(Key::F17),
        "f18" => Ok(Key::F18),
        "f19" => Ok(Key::F19),
        "f20" => Ok(Key::F20),
        "f21" => Ok(Key::F21),
        "f22" => Ok(Key::F22),
        "f23" => Ok(Key::F23),
        "f24" => Ok(Key::F24),

        _ if key.len() == 1 => key.chars().next().map(Key::Unicode).ok_or("Invalid key"),
        _ => Err("Unrecognized key"),
    }
}

pub fn move_mouse(enigo: &mut Enigo, delta_x: i32, delta_y: i32) {
    match enigo.move_mouse(delta_x, delta_y, Coordinate::Rel) {
        Ok(_) => { /* Handle success case */ }
        Err(_error) => { /* Handle error case */ }
    }
}

pub fn scroll_mouse_y(enigo: &mut Enigo, delta: i32) {
    match enigo.scroll(delta, Axis::Vertical) {
        Ok(_) => { /* Handle success case */ }
        Err(_error) => { /* Handle error case */ }
    }
}

pub fn scroll_mouse_x(enigo: &mut Enigo, delta: i32) {
    match enigo.scroll(delta, Axis::Horizontal) {
        Ok(_) => { /* Handle success case */ }
        Err(_error) => { /* Handle error case */ }
    }
}

pub fn handle_mouse_action(enigo: &mut Enigo, request: &MouseRequest) {
    let (button, method) = map_button_action(&request.click.button, &request.click.action);
    match method {
        MouseMethod::Click => match enigo.button(button, Click) {
            Ok(_) => { /* Handle success case */ }
            Err(_err) => { /* Handle error case */ }
        },
        MouseMethod::Down => match enigo.button(button, Press) {
            Ok(_) => { /* Handle success case */ }
            Err(_err) => { /* Handle error case */ }
        },
        MouseMethod::Up => match enigo.button(button, Release) {
            Ok(_) => { /* Handle success case */ }
            Err(_err) => { /* Handle error case */ }
        },
    }
}

enum MouseMethod {
    Click,
    Down,
    Up,
}

fn map_button_action(button: &MouseButton, action: &Action) -> (Button, MouseMethod) {
    let button = match *button {
        MouseButton::Left => Button::Left,
        MouseButton::Middle => Button::Middle,
        MouseButton::Right => Button::Right,
        MouseButton::Back => Button::Back,
        MouseButton::Forward => Button::Forward,
        MouseButton::ScrollUp => Button::ScrollUp,
        MouseButton::ScrollDown => Button::ScrollDown,
        MouseButton::ScrollLeft => Button::ScrollLeft,
        MouseButton::ScrollRight => Button::ScrollRight,
    };

    let method = match *action {
        // Dereference to match against the enum variants
        Action::Click => MouseMethod::Click,
        Action::Down => MouseMethod::Down,
        Action::Up => MouseMethod::Up,
    };

    (button, method)
}
