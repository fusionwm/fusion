use smithay::{
    backend::input::{
        AbsolutePositionEvent, ButtonState, Event, InputBackend, InputEvent, KeyState,
        KeyboardKeyEvent, Keycode, PointerButtonEvent, PointerMotionEvent,
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
    pub fn handle_input_event<I: InputBackend>(&mut self, input: InputEvent<I>)
    where
        I::Device: 'static,
    {
        match input {
            InputEvent::PointerMotion { event } => {
                let delta = event.delta();
                self.cursor_pos += delta;

                let location = self.cursor_pos;
                let under = self.surface_under(location);

                // Ограничиваем, чтобы мышь не ушла за экран (Screen Clipping)
                //let size = output.current_mode().size;
                //state.cursor_pos.x = state.cursor_pos.x.clamp(0.0, size.w as f64);
                //state.cursor_pos.y = state.cursor_pos.y.clamp(0.0, size.h as f64);
                //println!("Cursor pos: {:#?}", pointer.current_location());

                let pointer = self.seat.get_pointer().unwrap();
                let serial = SERIAL_COUNTER.next_serial();
                pointer.motion(
                    self,
                    under,
                    &MotionEvent {
                        location,
                        serial,
                        time: event.time_msec(),
                    },
                );
            }
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
                    let location = self.cursor_pos;

                    // Ищем окно и ПОВЕРХНОСТЬ под курсором
                    let under = globals.space.element_under(location).map(|(w, l)| {
                        // важно: берем поверхность с учетом локальных координат внутри окна
                        let surface = w
                            .surface_under(location - l.to_f64(), WindowSurfaceType::all())
                            .unwrap()
                            .0;
                        (w.clone(), surface)
                    });

                    if let Some((window, surface)) = under {
                        // 1. Поднимаем окно в Space
                        globals.space.raise_element(&window, true);

                        // 2. Активируем окно (важно для XDG Shell)
                        window.set_activated(true);

                        // 3. Устанавливаем фокус клавиатуры
                        keyboard.set_focus(
                            self,
                            Some(surface), // Передаем конкретную поверхность
                            serial,
                        );

                        // 4. Генерируем Configure события
                        window.toplevel().unwrap().send_configure();
                    } else {
                        // Если кликнули мимо — снимаем фокус
                        globals.space.elements().for_each(|window| {
                            window.set_activated(false);
                            window.toplevel().unwrap().send_configure();
                        });
                        keyboard.set_focus(self, Option::<WlSurface>::None, serial);
                    }
                }

                // ВАЖНО: Перед кликом Smithay должен знать, где находится указатель
                // Обычно это делается в PointerMotion, но для надежности можно и тут:
                // pointer.motion(self, under.map(|(_, s)| (s, location)), ...);

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

                if event.key_code() == Keycode::new(9) {
                    std::process::exit(0);
                }
                if event.key_code() == Keycode::new(10) && event.state() == KeyState::Released {
                    let mut cmd = std::process::Command::new("kitty");
                    cmd.spawn().unwrap();
                }
            }
            _ => {}
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
