//! Touch screens (the Android build): in play, fingers drive the virtual pad of
//! `horizon_client_input::touch` and turn the view; over a screen, a finger is the mouse
//! pointer and a tap is a left click.
use super::*;
use horizon_client_input::touch::{self, Phase};
use winit::event::{Touch, TouchPhase};

impl Game<'_> {
    /// A `WindowEvent::Touch`. Returns false when touch controls are off, so the event goes on
    /// as usual.
    pub(super) fn touch_event(
        &mut self,
        event_loop: &dyn EventLoopCtl,
        id: WindowId,
        t: Touch,
    ) -> bool {
        if !touch::enabled() {
            return false;
        }
        let [w, h] = self
            .renderer
            .as_ref()
            .map_or([1, 1], |r| [r.config.width.max(1), r.config.height.max(1)]);
        let point = [
            t.location.x as f32 / w as f32,
            t.location.y as f32 / h as f32,
        ];
        let phase = match t.phase {
            TouchPhase::Started => Phase::Started,
            TouchPhase::Moved => Phase::Moved,
            TouchPhase::Ended | TouchPhase::Cancelled => Phase::Ended,
        };
        if self.screen == Screen::Playing && self.player.is_some() {
            touch::with(|c| c.touch(t.id, phase, point, w as f32 / h as f32));
            return true;
        }
        // A screen is open: the fingers lift off the play controls, and one finger is the mouse.
        touch::with(|c| c.release());
        let device_id = t.device_id;
        let origin = input_trace::InputOrigin::User;
        self.handle_window_event(
            event_loop,
            id,
            WindowEvent::CursorMoved {
                device_id,
                position: t.location,
            },
            origin,
        );
        let state = match t.phase {
            TouchPhase::Started => Some(ElementState::Pressed),
            TouchPhase::Ended | TouchPhase::Cancelled => Some(ElementState::Released),
            TouchPhase::Moved => None,
        };
        if let Some(state) = state {
            self.handle_window_event(
                event_loop,
                id,
                WindowEvent::MouseInput {
                    device_id,
                    state,
                    button: MouseButton::Left,
                },
                origin,
            );
        }
        true
    }

    /// Once per frame in play: the look drag turns the view as a grabbed mouse would.
    pub(super) fn touch_frame(&mut self) {
        if !touch::enabled() {
            return;
        }
        let (dx, dy) = touch::with(|c| c.take_look());
        if self.screen == Screen::Playing && (dx != 0.0 || dy != 0.0) {
            self.mouse.add_motion(dx, dy);
            // No pointer to grab on a phone: the look counts as a grabbed one, as an agent's does.
            self.agent_pointer = true;
        }
    }

    /// The pad the fingers hold, merged into the controllers' state.
    pub(super) fn touch_pad(&self) -> horizon_client::gamepad::PadState {
        if touch::enabled() && self.screen == Screen::Playing {
            touch::with(|c| c.state())
        } else {
            Default::default()
        }
    }

    /// The buttons and the movement stick, over the HUD in play.
    pub(super) fn draw_touch_controls(&self, batch: &mut HudBatch) {
        if !touch::enabled() || self.screen != Screen::Playing {
            return;
        }
        let atlas = &self.gui.atlas;
        let (w, h) = (batch.width, batch.height);
        let (stick, held): (_, Vec<bool>) = touch::with(|c| {
            (
                c.stick(),
                touch::BUTTONS.iter().map(|b| c.held(b.button)).collect(),
            )
        });
        for (button, down) in touch::BUTTONS.iter().zip(held) {
            let [x, y, bw, bh] = button.rect;
            let (x, y, bw, bh) = (x * w, y * h, bw * w, bh * h);
            let fill = if down {
                rgba(255, 255, 255, 110)
            } else {
                rgba(0, 0, 0, 90)
            };
            batch.fill(atlas, x, y, bw, bh, fill);
            let edge = rgba(255, 255, 255, 140);
            batch.fill(atlas, x, y, bw, 1.0, edge);
            batch.fill(atlas, x, y + bh - 1.0, bw, 1.0, edge);
            batch.fill(atlas, x, y, 1.0, bh, edge);
            batch.fill(atlas, x + bw - 1.0, y, 1.0, bh, edge);
            let tw = HudBatch::text_width(atlas, button.label);
            batch.text(
                atlas,
                button.label,
                (x + (bw - tw) / 2.0).floor(),
                (y + (bh - 8.0) / 2.0).floor(),
                u32::MAX,
                true,
            );
        }
        if let Some((centre, finger)) = stick {
            let r = touch::STICK_RADIUS * h;
            let (cx, cy) = (centre[0] * w, centre[1] * h);
            batch.fill(atlas, cx - r, cy - r, r * 2.0, r * 2.0, rgba(0, 0, 0, 70));
            let dx = (finger[0] * w - cx).clamp(-r, r);
            let dy = (finger[1] * h - cy).clamp(-r, r);
            let k = r * 0.4;
            batch.fill(
                atlas,
                cx + dx - k,
                cy + dy - k,
                k * 2.0,
                k * 2.0,
                rgba(255, 255, 255, 120),
            );
        }
    }
}
