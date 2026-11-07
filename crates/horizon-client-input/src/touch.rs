//! Touch controls for phones (the Android build): fingers on the screen become a
//! virtual controller and a look drag.
//!
//! - A finger landing in the left movement zone is a floating stick: its offset from
//!   where it landed tilts the left stick.
//! - A finger landing anywhere else that is not a button turns the view, as a mouse
//!   drag would.
//! - The buttons are pad buttons, so they go through the controller bindings
//!   (`horizon-controller.txt`) like a real pad.
//!
//! Positions are fractions of the window (0..1, origin top left), so the layout is the
//! same on every screen size and the HUD can draw it in its own units.
use crate::gamepad::{Button, PadState};
use std::sync::Mutex;

/// One on-screen button. `rect` is `[x, y, w, h]` in window fractions.
#[derive(Clone, Copy, Debug)]
pub struct TouchButton {
    pub button: Button,
    pub label: &'static str,
    pub rect: [f32; 4],
    /// A toggle stays down until pressed again (sneak, sprint).
    pub toggle: bool,
}

const fn b(button: Button, label: &'static str, rect: [f32; 4], toggle: bool) -> TouchButton {
    TouchButton {
        button,
        label,
        rect,
        toggle,
    }
}

/// The layout. Right side: jump, attack, use, sneak. Top row: menus and hotbar.
pub const BUTTONS: &[TouchButton] = &[
    b(Button::South, "Jump", [0.86, 0.72, 0.12, 0.22], false),
    b(Button::RightTrigger, "Hit", [0.86, 0.46, 0.12, 0.22], false),
    b(Button::LeftTrigger, "Use", [0.72, 0.58, 0.12, 0.22], false),
    b(Button::RightStick, "Sneak", [0.72, 0.84, 0.12, 0.13], true),
    b(Button::LeftStick, "Run", [0.02, 0.36, 0.10, 0.12], true),
    b(Button::North, "Inv", [0.58, 0.84, 0.10, 0.13], false),
    b(Button::LeftBumper, "<", [0.36, 0.02, 0.08, 0.12], false),
    b(Button::RightBumper, ">", [0.56, 0.02, 0.08, 0.12], false),
    b(Button::DPadDown, "Drop", [0.46, 0.02, 0.08, 0.12], false),
    b(Button::DPadUp, "Chat", [0.02, 0.02, 0.08, 0.12], false),
    b(Button::Select, "View", [0.12, 0.02, 0.08, 0.12], false),
    b(Button::Start, "||", [0.90, 0.02, 0.08, 0.12], false),
];

/// The movement zone, `[x, y, w, h]`: the lower left of the screen.
pub const MOVE_ZONE: [f32; 4] = [0.0, 0.40, 0.45, 0.60];
/// How far (in window heights) a finger travels for a full stick tilt.
pub const STICK_RADIUS: f32 = 0.12;
/// Look drag: mouse units per window width of finger travel.
pub const LOOK_SCALE: f64 = 1600.0;

fn inside([x, y, w, h]: [f32; 4], [px, py]: [f32; 2]) -> bool {
    px >= x && px < x + w && py >= y && py < y + h
}

/// The button under a point, if any.
pub fn button_at(point: [f32; 2]) -> Option<&'static TouchButton> {
    BUTTONS.iter().find(|b| inside(b.rect, point))
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Role {
    Move { start: [f32; 2] },
    Look { last: [f32; 2] },
    Button(Button),
}

#[derive(Clone, Copy, Debug)]
struct Finger {
    id: u64,
    role: Role,
    pos: [f32; 2],
}

/// The phase of a touch, as winit reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Started,
    Moved,
    Ended,
}

/// The fingers on the screen and what each one does.
#[derive(Debug, Default)]
pub struct TouchControls {
    fingers: Vec<Finger>,
    toggled: u32,
    look: (f64, f64),
    aspect: f32,
}

impl TouchControls {
    /// A touch at `point` (window fractions). `aspect` is width / height.
    pub fn touch(&mut self, id: u64, phase: Phase, point: [f32; 2], aspect: f32) {
        self.aspect = aspect.max(0.1);
        match phase {
            Phase::Started => {
                self.fingers.retain(|f| f.id != id);
                let role = if let Some(button) = button_at(point) {
                    if button.toggle {
                        self.toggled ^= 1 << button.button as u32;
                    }
                    Role::Button(button.button)
                } else if inside(MOVE_ZONE, point) {
                    Role::Move { start: point }
                } else {
                    Role::Look { last: point }
                };
                self.fingers.push(Finger {
                    id,
                    role,
                    pos: point,
                });
            }
            Phase::Moved => {
                if let Some(f) = self.fingers.iter_mut().find(|f| f.id == id) {
                    if let Role::Look { last } = &mut f.role {
                        self.look.0 += f64::from(point[0] - last[0]) * LOOK_SCALE;
                        // Fractions of the height, scaled to the same units as x.
                        self.look.1 += f64::from((point[1] - last[1]) / self.aspect) * LOOK_SCALE;
                        *last = point;
                    }
                    f.pos = point;
                }
            }
            Phase::Ended => self.fingers.retain(|f| f.id != id),
        }
    }

    /// Lift every finger (the app went to the background, or a screen opened).
    pub fn release(&mut self) {
        self.fingers.clear();
        self.look = (0.0, 0.0);
    }

    /// The look drag since the last call, in mouse units.
    pub fn take_look(&mut self) -> (f64, f64) {
        std::mem::take(&mut self.look)
    }

    /// The pad the fingers hold now.
    pub fn state(&self) -> PadState {
        let mut pad = PadState::default();
        for button in BUTTONS.iter().filter(|b| b.toggle) {
            pad.set(button.button, self.toggled & (1 << button.button as u32) != 0);
        }
        for f in &self.fingers {
            match f.role {
                Role::Button(button) => {
                    if BUTTONS.iter().any(|b| b.button == button && !b.toggle) {
                        match button {
                            Button::LeftTrigger => pad.triggers[0] = 1.0,
                            Button::RightTrigger => pad.triggers[1] = 1.0,
                            _ => pad.set(button, true),
                        }
                    }
                }
                Role::Move { start } => {
                    let dx = (f.pos[0] - start[0]) * self.aspect / STICK_RADIUS;
                    let dy = (f.pos[1] - start[1]) / STICK_RADIUS;
                    let len = (dx * dx + dy * dy).sqrt().max(1.0);
                    // Up on the screen is up on the stick.
                    pad.left = [dx / len, -dy / len];
                }
                Role::Look { .. } => {}
            }
        }
        pad
    }

    /// Where the movement stick is drawn: its centre and the finger, if a finger holds it.
    pub fn stick(&self) -> Option<([f32; 2], [f32; 2])> {
        self.fingers.iter().find_map(|f| match f.role {
            Role::Move { start } => Some((start, f.pos)),
            _ => None,
        })
    }

    /// The button is held (for drawing it pressed).
    pub fn held(&self, button: Button) -> bool {
        self.toggled & (1 << button as u32) != 0
            || self
                .fingers
                .iter()
                .any(|f| f.role == Role::Button(button))
    }
}

/// The touch controls of the one game window a phone has.
pub static TOUCH: Mutex<TouchControls> = Mutex::new(TouchControls {
    fingers: Vec::new(),
    toggled: 0,
    look: (0.0, 0.0),
    aspect: 1.0,
});

/// The touch controls are on: the Android build, or `HORIZON_TOUCH=1` on a desktop for testing.
pub fn enabled() -> bool {
    cfg!(target_os = "android") || std::env::var_os("HORIZON_TOUCH").is_some_and(|v| v == "1")
}

/// Runs `f` on the shared touch controls.
pub fn with<R>(f: impl FnOnce(&mut TouchControls) -> R) -> R {
    let mut guard = TOUCH.lock().unwrap_or_else(|e| e.into_inner());
    f(&mut guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_zone_tilts_the_stick() {
        let mut t = TouchControls::default();
        t.touch(1, Phase::Started, [0.2, 0.7], 2.0);
        t.touch(1, Phase::Moved, [0.2, 0.7 - STICK_RADIUS], 2.0);
        let s = t.state();
        assert!(s.left[1] > 0.99, "{:?}", s.left);
        t.touch(1, Phase::Ended, [0.2, 0.5], 2.0);
        assert_eq!(t.state().left, [0.0, 0.0]);
    }

    #[test]
    fn buttons_and_toggles() {
        let mut t = TouchControls::default();
        let jump = BUTTONS.iter().find(|b| b.button == Button::South).unwrap();
        let p = [jump.rect[0] + 0.01, jump.rect[1] + 0.01];
        t.touch(2, Phase::Started, p, 2.0);
        assert!(t.state().held(Button::South));
        t.touch(2, Phase::Ended, p, 2.0);
        assert!(!t.state().held(Button::South));
        let sneak = BUTTONS.iter().find(|b| b.button == Button::RightStick).unwrap();
        let p = [sneak.rect[0] + 0.01, sneak.rect[1] + 0.01];
        t.touch(3, Phase::Started, p, 2.0);
        t.touch(3, Phase::Ended, p, 2.0);
        assert!(t.state().held(Button::RightStick));
    }

    #[test]
    fn right_side_drag_looks() {
        let mut t = TouchControls::default();
        t.touch(4, Phase::Started, [0.6, 0.4], 2.0);
        t.touch(4, Phase::Moved, [0.7, 0.4], 2.0);
        let (x, y) = t.take_look();
        assert!(x > 100.0 && y == 0.0);
        assert_eq!(t.take_look(), (0.0, 0.0));
    }

    #[test]
    fn buttons_do_not_overlap() {
        for (i, a) in BUTTONS.iter().enumerate() {
            for b in &BUTTONS[i + 1..] {
                let [ax, ay, aw, ah] = a.rect;
                let [bx, by, bw, bh] = b.rect;
                let apart = ax + aw <= bx || bx + bw <= ax || ay + ah <= by || by + bh <= ay;
                assert!(apart, "{} overlaps {}", a.label, b.label);
            }
        }
    }
}
