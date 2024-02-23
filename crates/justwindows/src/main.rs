use justshow_x11::{
    error::Error,
    events::EventType,
    events::SomeEvent,
    keysym::KeySym,
    requests::{self, ConfigureWindowAttributes, GrabMode, KeyModifier},
    xerror::SomeError,
    Rectangle, WindowId, XDisplay,
};
use justshow_x11_simple::X11Connection;

#[derive(Clone)]
struct TallLayout {
    border_width: u16,
    window_pad: u16,
    active_border: u32,
    inactive_border: u32,
}

impl TallLayout {
    fn arrange_windows(
        &self,
        wm: &mut JustWindows,
        area: Rectangle,
        windows: &[WindowId],
    ) -> Result<(), Error> {
        let window_count = windows.len() as u16;

        for (idx, &window) in windows.iter().enumerate() {
            let width = area.width - self.border_width * 2 - self.window_pad * 2;
            let height = (area.height - (self.window_pad * (window_count + 1))) / window_count
                - self.border_width * 2;

            wm.conn
                .display_mut()
                .send_request(&requests::ConfigureWindow {
                    window,
                    attributes: ConfigureWindowAttributes::new()
                        .set_width(width)
                        .set_height(height)
                        .set_x(self.window_pad as i16 + area.x)
                        .set_y(
                            ((height as i16 + 2 * self.border_width as i16) * idx as i16
                                + (self.window_pad as i16 * (1 + idx as i16)))
                                + area.y,
                        )
                        .set_border_width(self.border_width),
                })?;
            let color = if wm.active_window == Some(window) {
                self.active_border
            } else {
                self.inactive_border
            };
            wm.conn.set_border_color(window, color)?;
        }

        Ok(())
    }
}

#[derive(Clone)]
enum Layout {
    Tall(TallLayout),
}

struct JustWindows {
    conn: X11Connection,
    managed_windows: Vec<WindowId>,
    layout: Layout,
    screen_width: u16,
    screen_height: u16,
    active_window: Option<WindowId>,
}

impl JustWindows {
    fn setup() -> Result<Self, Error> {
        let mut conn = X11Connection::new(XDisplay::open()?);

        let screen = conn.default_screen();
        let root = screen.root;

        conn.select_input(
            root,
            EventType::SUBSTRUCTURE_REDIRECT
                | EventType::SUBSTRUCTURE_NOTIFY
                | EventType::ENTER_WINDOW
                | EventType::LEAVE_WINDOW
                | EventType::STRUCTURE_NOTIFY,
        )?;
        conn.set_supported()?;

        let key_symbols = conn.key_symbols()?;

        for key in key_symbols.get_keycodes(KeySym::q) {
            conn.display_mut().send_request(&requests::GrabKey {
                owner_events: false,
                grab_window: root,
                modifiers: KeyModifier::CONTROL,
                key,
                pointer_mode: GrabMode::Asynchronous,
                keyboard_mode: GrabMode::Asynchronous,
            })?;
        }

        conn.flush()?;

        Ok(Self {
            conn,
            managed_windows: Vec::new(),
            layout: Layout::Tall(TallLayout {
                border_width: 3,
                window_pad: 10,
                inactive_border: 0xd0d0d0,
                active_border: 0x4eb4fa,
            }),
            screen_width: screen.width_in_pixels,
            screen_height: screen.height_in_pixels,
            active_window: None,
        })
    }

    fn arrange_windows(&mut self) -> Result<(), Error> {
        match self.layout.clone() {
            Layout::Tall(layout) => {
                if let Some((master_window, stack_windows)) = self
                    .managed_windows
                    .iter()
                    .copied()
                    .rev()
                    .collect::<Vec<WindowId>>()
                    .split_first()
                    .map(|(l, r)| (*l, r.to_vec()))
                {
                    if stack_windows.is_empty() {
                        layout.arrange_windows(
                            self,
                            Rectangle {
                                x: 0,
                                y: 0,
                                width: self.screen_width,
                                height: self.screen_height,
                            },
                            &[master_window],
                        )?;
                    } else {
                        layout.arrange_windows(
                            self,
                            Rectangle {
                                x: 0,
                                y: 0,
                                width: self.screen_width / 2 + layout.window_pad / 2,
                                height: self.screen_height,
                            },
                            &[master_window],
                        )?;

                        layout.arrange_windows(
                            self,
                            Rectangle {
                                x: self.screen_width as i16 / 2 - layout.window_pad as i16 / 2,
                                y: 0,
                                width: self.screen_width / 2 + layout.window_pad / 2,
                                height: self.screen_height,
                            },
                            &stack_windows,
                        )?;
                    }
                    self.conn.flush()?;
                };
            }
        }

        Ok(())
    }

    fn find_managed_window(&self, window: WindowId) -> Option<usize> {
        self.managed_windows
            .iter()
            .enumerate()
            .find_map(|(idx, w)| (*w == window).then_some(idx))
    }

    fn manage_window(&mut self, window: WindowId) -> Result<(), Error> {
        if self.find_managed_window(window).is_some() {
            eprintln!(
                "justwindows: debug: window is already managed: {:?}",
                window
            );
        } else {
            self.managed_windows.push(window);
        }

        Ok(())
    }

    fn unmanage_window(&mut self, window: WindowId) -> Result<(), Error> {
        if let Some(destroyed_window_idx) = self.find_managed_window(window) {
            self.managed_windows.remove(destroyed_window_idx);
            self.arrange_windows()?;
        } else {
            eprintln!(
                "justwindows: debug: Destroyed window that is not managed: {:?}",
                window
            );
        }
        Ok(())
    }

    fn restore_windows(&mut self) -> Result<(), Error> {
        let root = self.root_window();
        let tree = self.conn.query_tree(root)?;
        for window in tree.children {
            self.manage_window(window)?;
        }
        self.conn.flush()?;

        Ok(())
    }

    fn root_window(&self) -> WindowId {
        self.conn.display().screens()[0].root
    }

    fn set_initial_window_properties(&mut self, window: WindowId) -> Result<(), Error> {
        self.conn.select_input(
            window,
            EventType::ENTER_WINDOW | EventType::STRUCTURE_NOTIFY | EventType::PROPERTY_CHANGE,
        )?;
        self.conn.flush()?;
        Ok(())
    }

    fn is_client(&self, window: WindowId) -> bool {
        self.managed_windows.contains(&window)
    }

    fn rescreen(&mut self) -> Result<(), Error> {
        // TODO: Run xinerama's `getScreenInfo` when it's implemented.
        let root = self.root_window();
        let root_geometry = self.conn.get_window_geometry(root)?;
        self.screen_width = root_geometry.width;
        self.screen_height = root_geometry.height;
        Ok(())
    }

    fn handle_event(&mut self, event: SomeEvent) -> Result<(), Error> {
        match event {
            SomeEvent::ConfigureRequest(event) => {
                let attributes = ConfigureWindowAttributes::from(&event);
                self.conn
                    .display_mut()
                    .send_request(&requests::ConfigureWindow {
                        window: event.window,
                        attributes,
                    })?;
                self.set_initial_window_properties(event.window)?;
            }
            SomeEvent::MapRequest(event) => {
                self.conn.display_mut().send_request(&requests::MapWindow {
                    window: event.window,
                })?;
                self.manage_window(event.window)?;
                self.arrange_windows()?;
                self.conn.set_focus(event.window)?;
                self.conn.flush()?;
            }
            SomeEvent::DestroyNotify(event) => {
                if self.is_client(event.window) {
                    self.unmanage_window(event.window)?;
                }
            }
            SomeEvent::ClientMessage(event) => {
                dbg!(event);
            }
            SomeEvent::UnknownEvent(event) => {
                dbg!(event);
            }
            SomeEvent::EnterNotify(event) => {
                let root = self.root_window();
                if event.event != root {
                    self.active_window = Some(event.event);
                    self.arrange_windows()?;
                }
            }
            SomeEvent::LeaveNotify(event) => {
                let root = self.root_window();
                if event.event == root && !event.same_screen() {
                    // self.conn.set_focus(root)?;
                }
            }
            SomeEvent::ConfigureNotify(event) => {
                let root = self.root_window();
                if event.window == root {
                    self.rescreen()?;
                }
            }
            SomeEvent::MapNotify(_)
            | SomeEvent::CreateNotify(_)
            | SomeEvent::UnmapNotify(_)
            | SomeEvent::MappingNotify(_)
            | SomeEvent::PropertyNotify(_) => {}
            _ => {
                dbg!(event);
            }
        }

        self.conn.flush()?;
        Ok(())
    }
}

pub fn go() -> Result<(), Error> {
    let mut wm = JustWindows::setup()?;
    wm.restore_windows()?;

    // std::process::Command::new("xterm").spawn()?;
    // std::process::Command::new("xterm").spawn()?;
    std::process::Command::new("xterm").spawn()?;

    loop {
        for error in wm.conn.display_mut().errors() {
            match error {
                SomeError::Access(error) => {
                    panic!("justwindows: Other window manager is running: {:?}", error)
                }
                _ => {
                    dbg!(error);
                    // panic!();
                }
            }
        }

        while let Some(event) = wm.conn.display_mut().next_event()? {
            wm.handle_event(event)?;
        }
    }
}

fn main() {
    match go() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("justwindows: error: {}", err);
        }
    }
}
