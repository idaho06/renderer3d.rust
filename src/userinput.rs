//! Keyboard and mouse input state for one frame.
//!
//! [`UserInput`] is populated by [`crate::display::Display::update_user_input`] each frame
//! and read by [`crate::camera::Camera::update`] and any other game-logic code.
//!
//! See book chapter: _Input handling_ (TODO: link when mdBook is set up).

/// The pressed/changed state of a single key or mouse button.
pub struct KeyState {
    /// `true` while the key is held down.
    pub pressed: bool,
    /// `true` for exactly the frame on which the key state changed.
    pub changed: bool,
}

/// Mouse position and button state.
pub struct MouseState {
    /// Cursor X — absolute screen coordinate, or relative delta if `is_relative`.
    pub x: i32,
    /// Cursor Y — absolute screen coordinate, or relative delta if `is_relative`.
    pub y: i32,
    /// Left mouse button state.
    pub left: KeyState,
    /// Right mouse button state.
    pub right: KeyState,
    /// Middle mouse button state.
    pub middle: KeyState,
    /// When `true`, `x`/`y` are relative deltas (SDL2 captured mouse mode).
    pub is_relative: bool,
}

/// Aggregated keyboard and mouse input for one frame.
pub struct UserInput {
    /// `true` if the user closed the window or pressed Escape.
    pub quit: bool,
    /// W key state (move camera forward).
    pub key_w: KeyState,
    /// A key state (strafe camera left).
    pub key_a: KeyState,
    /// S key state (move camera backward).
    pub key_s: KeyState,
    /// D key state (strafe camera right).
    pub key_d: KeyState,
    /// Mouse state.
    pub mouse: MouseState,
}

impl Default for UserInput {
    fn default() -> Self {
        Self::new()
    }
}

impl UserInput {
    #[must_use]
    /// Creates a zeroed `UserInput` with all keys released and mouse at origin.
    pub fn new() -> Self {
        Self {
            quit: false,
            key_w: KeyState {
                pressed: false,
                changed: false,
            },
            key_a: KeyState {
                pressed: false,
                changed: false,
            },
            key_s: KeyState {
                pressed: false,
                changed: false,
            },
            key_d: KeyState {
                pressed: false,
                changed: false,
            },
            mouse: MouseState {
                x: 0,
                y: 0,
                left: KeyState {
                    pressed: false,
                    changed: false,
                },
                right: KeyState {
                    pressed: false,
                    changed: false,
                },
                middle: KeyState {
                    pressed: false,
                    changed: false,
                },
                is_relative: false,
            },
        }
    }
    /// Clears all `changed` flags. Call at the start of each frame before polling events.
    pub fn reset(&mut self) {
        self.key_w.changed = false;
        self.key_a.changed = false;
        self.key_s.changed = false;
        self.key_d.changed = false;
        self.mouse.left.changed = false;
        self.mouse.right.changed = false;
        self.mouse.middle.changed = false;
    }
}
