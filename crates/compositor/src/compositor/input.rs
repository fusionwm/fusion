use smithay::{
    backend::{
        input::{
            AbsolutePositionEvent, ButtonState, Event, InputEvent, KeyboardKeyEvent,
            PointerButtonEvent,
        },
        winit::WinitInput,
    },
    desktop::WindowSurfaceType,
    input::{
        keyboard::FilterResult,
        pointer::{ButtonEvent, MotionEvent},
    },
    utils::{Logical, Point, SERIAL_COUNTER},
};
use wayland_server::protocol::wl_surface::WlSurface;

use crate::compositor::{backend::Backend, state::App};

impl<B: Backend + 'static> App<B> {
    pub fn handle_input_event(&mut self, input: InputEvent<WinitInput>) {
        match input {
            InputEvent::PointerMotion { event } => {}
            InputEvent::PointerMotionAbsolute { event } => {
                let output_geo = {
                    let space = &self.globals.lock().unwrap().space;
                    let output = space.outputs().next().unwrap();

                    space.output_geometry(output).unwrap()
                };

                let pos = event.position_transformed(output_geo.size) + output_geo.loc.to_f64();

                let serial = SERIAL_COUNTER.next_serial();

                let pointer = self.seat.get_pointer().unwrap();

                let under = self.surface_under(pos);

                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location: pos,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerButton { event } => {
                let globals = self.globals.clone();
                let mut globals = globals.lock().unwrap();
                let pointer = self.seat.get_pointer().unwrap();
                let keyboard = self.seat.get_keyboard().unwrap();

                let serial = SERIAL_COUNTER.next_serial();

                let button = event.button_code();

                let button_state = event.state();

                if ButtonState::Pressed == button_state && !pointer.is_grabbed() {
                    if let Some((window, _loc)) = globals
                        .space
                        .element_under(pointer.current_location())
                        .map(|(w, l)| (w.clone(), l))
                    {
                        globals.space.raise_element(&window, true);
                        keyboard.set_focus(
                            self,
                            Some(window.toplevel().unwrap().wl_surface().clone()),
                            serial,
                        );
                        globals.space.elements().for_each(|window| {
                            window.toplevel().unwrap().send_pending_configure();
                        });
                    } else {
                        globals.space.elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_pending_configure();
                        });
                        keyboard.set_focus(self, Option::<WlSurface>::None, serial);
                    }
                }

                pointer.button(
                    self,
                    &ButtonEvent {
                        button,
                        state: button_state,
                        serial,
                        time: event.time_msec(),
                    },
                );
                pointer.frame(self);
            }
            InputEvent::PointerAxis { event } => {}
            InputEvent::Keyboard { event } => {
                let keyboard = self.seat.get_keyboard().unwrap();
                let serial = SERIAL_COUNTER.next_serial();
                keyboard.input::<(), _>(
                    self,
                    event.key_code(),
                    event.state(),
                    serial,
                    event.time_msec(),
                    |_, _, _| FilterResult::Forward,
                );
            }
            _ => {} //InputEvent::DeviceAdded { device } => todo!(),
                    //InputEvent::DeviceRemoved { device } => todo!(),
                    //InputEvent::GestureSwipeBegin { event } => todo!(),
                    //InputEvent::GestureSwipeUpdate { event } => todo!(),
                    //InputEvent::GestureSwipeEnd { event } => todo!(),
                    //InputEvent::GesturePinchBegin { event } => todo!(),
                    //InputEvent::GesturePinchUpdate { event } => todo!(),
                    //InputEvent::GesturePinchEnd { event } => todo!(),
                    //InputEvent::GestureHoldBegin { event } => todo!(),
                    //InputEvent::GestureHoldEnd { event } => todo!(),
                    //InputEvent::TouchDown { event } => todo!(),
                    //InputEvent::TouchMotion { event } => todo!(),
                    //InputEvent::TouchUp { event } => todo!(),
                    //InputEvent::TouchCancel { event } => todo!(),
                    //InputEvent::TouchFrame { event } => todo!(),
                    //InputEvent::TabletToolAxis { event } => todo!(),
                    //InputEvent::TabletToolProximity { event } => todo!(),
                    //InputEvent::TabletToolTip { event } => todo!(),
                    //InputEvent::TabletToolButton { event } => todo!(),
                    //InputEvent::SwitchToggle { event } => todo!(),
                    //InputEvent::Special(_) => todo!(),
        }
    }

    pub fn surface_under(
        &self,
        pos: Point<f64, Logical>,
    ) -> Option<(WlSurface, Point<f64, Logical>)> {
        self.globals()
            .space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::ALL)
                    .map(|(s, p)| (s, (p + location).to_f64()))
            })
    }
}
