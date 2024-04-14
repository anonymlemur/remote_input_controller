use serde::{Deserialize, Serialize};

// Common type for all input requests
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action")]
pub enum InputRequest {
    Keyboard(KeyboardRequest),
    Mouse(MouseRequest),
}

// Detailed struct for keyboard input
#[derive(Serialize, Deserialize, Debug)]
pub struct KeyboardRequest {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub modifiers: Modifiers,
}
impl Default for KeyboardRequest {
    fn default() -> Self {
        KeyboardRequest {
            key: "".to_string(),
            modifiers: Modifiers::default(),
        }
    }
}

// Define the Modifiers struct
#[derive(Serialize, Deserialize, Debug)]
pub struct Modifiers {
    #[serde(default)]
    pub alt: bool,
    #[serde(default)]
    pub ctrl: bool,
    #[serde(default)]
    pub meta: bool,
    #[serde(default)]
    pub shift: bool,
}
impl Default for Modifiers {
    fn default() -> Self {
        Modifiers {
            alt: false,
            ctrl: false,
            meta: false,
            shift: false,
        }
    }
}
// Struct for move action (e.g., moving the mouse cursor)
#[derive(Serialize, Deserialize, Debug)]
pub struct MoveRequest {
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
}

impl Default for MoveRequest {
    fn default() -> Self {
        MoveRequest { x: 0, y: 0 }
    }
}

// Struct for mouse button actions (click, up, down)
#[derive(Serialize, Deserialize, Debug)]
pub struct MouseRequest {
    #[serde(default)]
    pub command: MouseCommand,
    #[serde(default)]
    pub click: ClickRequest,
    #[serde(default)]
    pub move_direction: MoveRequest,
    #[serde(default)]
    pub scroll: ScrollRequest,
}

impl Default for MouseRequest {
    fn default() -> Self {
        MouseRequest {
            command: MouseCommand::default(),
            click: ClickRequest::default(),
            move_direction: MoveRequest::default(),
            scroll: ScrollRequest::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MouseCommand {
    Move,
    Click,
    Scroll,
    StopMove,
}

impl Default for MouseCommand {
    fn default() -> Self {
        MouseCommand::Move
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClickRequest {
    #[serde(default)]
    pub button: Button,
    #[serde(default)]
    pub action: Action,
}
impl Default for ClickRequest {
    fn default() -> Self {
        ClickRequest {
            button: Button::default(),
            action: Action::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    Click,
    Up,
    Down,
}
impl Default for Action {
    fn default() -> Self {
        Action::Click
    }
}

// Enum for mouse buttons
#[derive(Serialize, Deserialize, Debug)]
pub enum Button {
    Left,
    Middle,
    Right,
    Forward,
    Back,
}

impl Default for Button {
    fn default() -> Self {
        Button::Left
    }
}

// Struct for stopping any continuous action
#[derive(Serialize, Deserialize, Debug)]
pub struct StopMoveRequest {
    #[serde(default)]
    pub reason: String,
}

impl Default for StopMoveRequest {
    fn default() -> Self {
        StopMoveRequest {
            reason: "No reason provided".to_string(),
        }
    }
}
// Struct for scroll actions
#[derive(Serialize, Deserialize, Debug)]
pub struct ScrollRequest {
    #[serde(default)]
    pub direction: ScrollDirection,
    #[serde(default)]
    pub delta: i32,
}
impl Default for ScrollRequest {
    fn default() -> Self {
        ScrollRequest {
            direction: ScrollDirection::default(),
            delta: 0,
        }
    }
}

// Enum for scroll directions
#[derive(Serialize, Deserialize, Debug)]
pub enum ScrollDirection {
    X,
    Y,
}
impl Default for ScrollDirection {
    fn default() -> Self {
        ScrollDirection::X
    }
}
