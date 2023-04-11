pub struct KeyState {
    pub pressed: bool,
    pub changed: bool,
}

pub struct MouseState {
    pub x: i32,
    pub y: i32,
    pub left: KeyState,
    pub right: KeyState,
    pub middle: KeyState,
    pub is_relative: bool,
}

pub struct UserInput {
    pub quit: bool,
    pub key_w: KeyState,
    pub key_a: KeyState,
    pub key_s: KeyState,
    pub key_d: KeyState,
    pub mouse: MouseState,
}

impl UserInput {
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