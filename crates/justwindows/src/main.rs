use justshow_x11::{
    error::Error,
    events::EventType,
    events::SomeEvent,
    requests::{self, ConfigureWindowAttributes, NoReply, WindowCreationAttributes},
    xerror::SomeError,
    PendingReply, Rectangle, WindowId, XDisplay,
};
use std::ops::DerefMut;

macro_rules! request_blocking {
    ($display:expr, $request:expr) => {{
        (|| {
            let pending_reply = $display.send_request(&($request))?;
            $display.flush()?;
            let reply = $display.await_pending_reply(pending_reply)?;
            Ok::<_, Error>(reply)
        })()
    }};
}

trait XDisplayUtils: Sized + DerefMut<Target = XDisplay> {
    fn select_input(
        mut self,
        window: WindowId,
        events: EventType,
    ) -> Result<PendingReply<NoReply>, Error> {
        self.send_request(&requests::ChangeWindowAttributes {
            window,
            attributes: WindowCreationAttributes::new().set_event_mask(events),
        })
    }

    fn default_screen(self) -> justshow_x11::Screen {
        self.screens()[0].clone()
    }

    fn set_border_width(
        mut self,
        window: WindowId,
        border_width: u16,
    ) -> Result<PendingReply<NoReply>, Error> {
        self.send_request(&requests::ConfigureWindow {
            window,
            attributes: ConfigureWindowAttributes::new().set_border_width(border_width),
        })
    }

    fn set_border_color(
        mut self,
        window: WindowId,
        border_color: u32,
    ) -> Result<PendingReply<NoReply>, Error> {
        self.send_request(&requests::ChangeWindowAttributes {
            window,
            attributes: WindowCreationAttributes::new().set_border_pixel(border_color),
        })
    }

    fn map_window(mut self, window: WindowId) -> Result<PendingReply<NoReply>, Error> {
        self.send_request(&requests::MapWindow { window })
    }
}

impl XDisplayUtils for &mut XDisplay {}

struct TallLayout {
    border_width: u16,
    window_pad: u16,
}

impl TallLayout {
    fn arrange_windows(
        &self,
        display: &mut XDisplay,
        area: Rectangle,
        windows: &[WindowId],
    ) -> Result<(), Error> {
        let window_count = windows.len() as u16;

        for (idx, &window) in windows.iter().enumerate() {
            let width = area.width - self.border_width * 2 - self.window_pad * 2;
            let height = (area.height - (self.window_pad * (window_count + 1))) / window_count
                - self.border_width * 2;

            display.send_request(&requests::ConfigureWindow {
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
            display.set_border_color(window, 0x00aa22 + idx as u32 * 100)?;
        }

        Ok(())
    }
}

enum Layout {
    Tall(TallLayout),
}

struct JustWindows {
    display: XDisplay,
    managed_windows: Vec<WindowId>,
    layout: Layout,
    screen_width: u16,
    screen_height: u16,
}

impl JustWindows {
    fn setup() -> Result<Self, Error> {
        let mut display = XDisplay::open()?;

        let screen = display.default_screen();
        let root = screen.root;

        display.select_input(
            root,
            EventType::SUBSTRUCTURE_REDIRECT | EventType::SUBSTRUCTURE_NOTIFY,
        )?;
        display.flush()?;

        Ok(Self {
            display,
            managed_windows: Vec::new(),
            layout: Layout::Tall(TallLayout {
                border_width: 4,
                window_pad: 16,
            }),
            screen_width: screen.width_in_pixels,
            screen_height: screen.height_in_pixels,
        })
    }

    fn arrange_windows(&mut self) -> Result<(), Error> {
        match &self.layout {
            Layout::Tall(layout) => {
                if let Some((master_window, stack_windows)) = self
                    .managed_windows
                    .iter()
                    .copied()
                    .rev()
                    .collect::<Vec<WindowId>>()
                    .split_first()
                    .map(|(l, r)| (l.clone(), r.to_vec()))
                {
                    if stack_windows.is_empty() {
                        layout.arrange_windows(
                            &mut self.display,
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
                            &mut self.display,
                            Rectangle {
                                x: 0,
                                y: 0,
                                width: self.screen_width / 2 + layout.window_pad / 2,
                                height: self.screen_height,
                            },
                            &[master_window],
                        )?;

                        layout.arrange_windows(
                            &mut self.display,
                            Rectangle {
                                x: self.screen_width as i16 / 2 - layout.window_pad as i16 / 2,
                                y: 0,
                                width: self.screen_width / 2 + layout.window_pad / 2,
                                height: self.screen_height,
                            },
                            &stack_windows,
                        )?;
                    }
                    self.display.flush()?;
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
        if let Some(_) = self.find_managed_window(window) {
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
        let root = self.display.default_screen().root;
        let tree = request_blocking!(self.display, requests::QueryTree { window: root })?;
        for window in tree.children {
            self.manage_window(window)?;
        }
        self.display.flush()?;

        Ok(())
    }

    fn handle_event(&mut self, event: SomeEvent) -> Result<(), Error> {
        match event {
            SomeEvent::ConfigureRequest(event) => {
                let attributes = ConfigureWindowAttributes::from(&event);
                self.display.send_request(&requests::ConfigureWindow {
                    window: event.window,
                    attributes,
                })?;
                self.display.flush()?;
            }
            SomeEvent::CreateNotify(_event) => {
                // self.manage_window(event.window)?;
                // self.display.flush()?;
            }
            SomeEvent::MapRequest(event) => {
                self.display.send_request(&requests::MapWindow {
                    window: event.window,
                })?;
                self.manage_window(event.window)?;
                self.arrange_windows()?;
                self.display.flush()?;
            }
            SomeEvent::DestroyNotify(event) => {
                self.unmanage_window(event.window)?;
            }
            SomeEvent::UnknownEvent(event) => {
                dbg!(event);
            }
            _ => {}
        }

        Ok(())
    }
}

pub fn go() -> Result<(), Error> {
    let mut wm = JustWindows::setup()?;
    wm.restore_windows()?;

    std::process::Command::new("xterm").spawn()?;
    std::process::Command::new("xterm").spawn()?;
    std::process::Command::new("xterm").spawn()?;

    loop {
        for error in wm.display.errors() {
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

        while let Some(event) = wm.display.next_event()? {
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
