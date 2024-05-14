// CLIPPY CONFIG
#![allow(
    clippy::new_without_default,
    clippy::unnecessary_cast,
    clippy::identity_op
)]

use crate::layout::{Layout, VerticalMasterSplit, VerticalStack};
use just_x11::{
    error::Error,
    events::EventType,
    events::SomeEvent,
    keysym::KeySym,
    requests::{self, ConfigureWindowAttributes, GrabMode, KeyCode, KeyModifier},
    xerror::SomeError,
    Rectangle, WindowId, XDisplay,
};
use just_x11_simple::{keys::KeySymbols, X11Connection};
use std::{collections::HashMap, process};

mod layout;

// TODO: FocusNext, FocusPrevious
/// Abstract action type
#[derive(Debug, Clone, Copy)]
enum JustAction {
    KillActive,
    Term,
}

struct KeyBindings {
    bindings: HashMap<KeyCode, JustAction>,
    key_symbols: KeySymbols,
}

impl KeyBindings {
    fn new(key_symbols: KeySymbols) -> Self {
        Self {
            bindings: HashMap::new(),
            key_symbols,
        }
    }

    fn bind_key_sym(
        &mut self,
        display: &mut XDisplay,
        root: WindowId,
        sym: KeySym,
        event: JustAction,
    ) -> Result<(), Error> {
        let key_codes = self.key_symbols.get_keycodes(sym);
        for key in key_codes {
            display.send_request(&requests::GrabKey {
                owner_events: false,
                grab_window: root,
                modifiers: KeyModifier::CONTROL,
                key,
                pointer_mode: GrabMode::Asynchronous,
                keyboard_mode: GrabMode::Asynchronous,
            })?;
            self.bindings.insert(key, event);
        }

        Ok(())
    }

    fn get_action(&self, key_code: KeyCode) -> Option<JustAction> {
        self.bindings.get(&key_code).copied()
    }
}

// struct WindowStack {

// }

struct Workspace {
    layout: Box<dyn Layout>,
    windows: Vec<WindowId>,
}

impl Workspace {
    pub fn new() -> Self {
        let layout = {
            let border_width = 3;
            let window_pad = 10;
            let inactive_border = 0xd0d0d0;
            let active_border = 0x4eb4fa;

            VerticalMasterSplit {
                border_width,
                window_pad,
                inactive_border,
                active_border,
                right: Box::new(VerticalStack {
                    border_width,
                    window_pad,
                    inactive_border,
                    active_border,
                }),
            }
        };

        Self::with_layout(Box::new(layout))
    }

    pub fn with_layout(layout: Box<dyn Layout>) -> Self {
        Self {
            layout,
            windows: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct WmScreen {
    size: Rectangle,
    workspace_idx: usize,
    root: WindowId,
}

struct JustWindows {
    conn: X11Connection,
    managed_windows: Vec<WindowId>,
    active_window: Option<WindowId>,
    bindings: KeyBindings,

    screens: Vec<WmScreen>,
    workspaces: Vec<Workspace>,

    active_workspace: usize,

    /// Processes that we have spawned.
    /// We use it to clean up zombie children as it's a bit more clean and cross-platform than
    /// catching sigchld signal.
    running_children: Vec<process::Child>,
}

impl JustWindows {
    fn setup() -> Result<Self, Error> {
        let mut conn = X11Connection::new(XDisplay::open()?);

        // FIXME: Get this with randr
        let screens = conn
            .display()
            .screens()
            .iter()
            .enumerate()
            .map(|(idx, screen)| WmScreen {
                size: Rectangle {
                    x: 0,
                    y: 0,
                    width: screen.width_in_pixels,
                    height: screen.height_in_pixels,
                },
                root: screen.root,
                workspace_idx: idx,
            })
            .collect::<Vec<_>>();
        dbg!(&screens);

        let workspaces = screens.iter().map(|_| Workspace::new()).collect::<Vec<_>>();

        let key_symbols = KeySymbols::new(conn.display_mut())?;
        let mut bindings = KeyBindings::new(key_symbols);

        for screen in &screens {
            conn.select_input(
                screen.root,
                EventType::SUBSTRUCTURE_REDIRECT
                    | EventType::SUBSTRUCTURE_NOTIFY
                    | EventType::ENTER_WINDOW
                    | EventType::LEAVE_WINDOW
                    | EventType::STRUCTURE_NOTIFY,
            )?;

            bindings.bind_key_sym(
                conn.display_mut(),
                screen.root,
                KeySym::q,
                JustAction::KillActive,
            )?;
            bindings.bind_key_sym(
                conn.display_mut(),
                screen.root,
                KeySym::Return,
                JustAction::Term,
            )?;
        }

        conn.flush()?;

        Ok(Self {
            conn,
            managed_windows: Vec::new(),
            active_window: None,
            bindings,
            running_children: Vec::new(),
            screens,
            workspaces,
            active_workspace: 0,
        })
    }

    fn arrange_windows(&mut self) -> Result<(), Error> {
        for screen in self.screens.clone() {
            let workspace = &self.workspaces[screen.workspace_idx];
            let positioned = workspace.layout.position_windows(
                screen.size,
                self.active_window,
                &workspace.windows,
            );
            dbg!(&positioned);

            positioned.into_iter().try_for_each(|positioned| {
                self.conn
                    .display_mut()
                    .send_request(&requests::ConfigureWindow {
                        window: positioned.window,
                        attributes: positioned.to_attributes(),
                    })?;

                self.conn
                    .set_border_color(positioned.window, positioned.border_color)?;
                Ok::<(), Error>(())
            })?;
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
            self.workspaces[self.active_workspace].windows.push(window);
        }

        Ok(())
    }

    fn unmanage_window(&mut self, window: WindowId) -> Result<(), Error> {
        self.cleanup_running_children()?;
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
            self.set_initial_window_properties(window)?;
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
        Ok(())
    }

    fn cleanup_running_children(&mut self) -> Result<(), Error> {
        let mut ret = Ok(());
        self.running_children
            .retain_mut(|child| match child.try_wait() {
                Ok(None) => true,
                Ok(Some(_)) => false,
                Err(err) => {
                    ret = Err(err);
                    true
                }
            });
        ret?;
        Ok(())
    }

    fn spawn(&mut self, command: &str) -> Result<(), Error> {
        let spawned_process = std::process::Command::new(command).spawn()?;
        self.running_children.push(spawned_process);

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
                // self.conn.set_focus(event.window)?;
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
                } else {
                    dbg!(event.event);
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
            SomeEvent::KeyPress(event) => {
                if let Some(event) = self.bindings.get_action(event.detail) {
                    match event {
                        JustAction::KillActive => {
                            if let Some(active) = self.active_window {
                                self.unmanage_window(active)?;
                                self.conn.kill_window(active)?;
                                self.active_window = None;
                            }
                        }
                        JustAction::Term => {
                            self.spawn("xterm")?;
                        }
                    }
                }
            }
            SomeEvent::MapNotify(_)
            | SomeEvent::CreateNotify(_)
            | SomeEvent::UnmapNotify(_)
            | SomeEvent::MappingNotify(_)
            | SomeEvent::PropertyNotify(_)
            | SomeEvent::KeyRelease(_)
            | SomeEvent::ButtonPress(_) => {}
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

    // wm.spawn("xterm")?;
    // wm.spawn("xterm")?;
    // wm.spawn("xterm")?;
    // wm.spawn("xterm")?;

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
