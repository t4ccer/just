use justshow_x11::{
    atoms::{self, AtomId},
    bitmask,
    error::Error,
    events::EventType,
    replies::{self, String8},
    requests::{self, ConfigureWindowAttributes, NoReply, WindowCreationAttributes},
    Drawable, OrNone, PendingReply, PixmapId, ResourceId, WindowId, XDisplay,
};
use std::{collections::HashMap, mem};

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

pub struct X11Connection {
    display: XDisplay,
    known_atoms: HashMap<AtomId, String8>,
}

impl X11Connection {
    pub fn new(display: XDisplay) -> Self {
        X11Connection {
            display,
            known_atoms: HashMap::new(),
        }
    }

    pub fn display(&self) -> &XDisplay {
        &self.display
    }

    pub fn display_mut(&mut self) -> &mut XDisplay {
        &mut self.display
    }

    pub fn select_input(
        &mut self,
        window: WindowId,
        events: EventType,
    ) -> Result<PendingReply<NoReply>, Error> {
        self.display
            .send_request(&requests::ChangeWindowAttributes {
                window,
                attributes: WindowCreationAttributes::new().set_event_mask(events),
            })
    }

    pub fn default_screen(&self) -> justshow_x11::Screen {
        self.display.screens()[0].clone()
    }

    pub fn set_border_width(
        &mut self,
        window: WindowId,
        border_width: u16,
    ) -> Result<PendingReply<NoReply>, Error> {
        self.display.send_request(&requests::ConfigureWindow {
            window,
            attributes: ConfigureWindowAttributes::new().set_border_width(border_width),
        })
    }

    pub fn set_border_color(
        &mut self,
        window: WindowId,
        border_color: u32,
    ) -> Result<PendingReply<NoReply>, Error> {
        self.display
            .send_request(&requests::ChangeWindowAttributes {
                window,
                attributes: WindowCreationAttributes::new().set_border_pixel(border_color),
            })
    }

    pub fn map_window(&mut self, window: WindowId) -> Result<PendingReply<NoReply>, Error> {
        self.display.send_request(&requests::MapWindow { window })
    }

    pub fn get_atom_name(&mut self, atom: AtomId) -> Result<String8, Error> {
        if let Some(atom_name) = self.known_atoms.get(&atom) {
            return Ok(atom_name.clone());
        }

        let r = request_blocking!(self.display, requests::GetAtomName { atom })?;

        self.known_atoms.insert(atom, r.name.clone());
        Ok(r.name)
    }

    pub fn flush(&mut self) -> Result<(), Error> {
        self.display.flush()
    }

    pub fn set_focus(&mut self, window: WindowId) -> Result<(), Error> {
        let hints = self.get_wm_hints(window)?;
        dbg!(hints);
        Ok(())
    }

    pub fn get_window_geometry(&mut self, window: WindowId) -> Result<replies::GetGeometry, Error> {
        request_blocking!(
            self.display,
            requests::GetGeometry {
                drawable: Drawable::Window(window)
            }
        )
    }

    pub fn query_tree(&mut self, window: WindowId) -> Result<replies::QueryTree, Error> {
        request_blocking!(self.display, requests::QueryTree { window })
    }

    pub fn get_wm_hints(&mut self, window: WindowId) -> Result<WindowManagerHints, Error> {
        const NUM_PROP_WMHINTS_ELEMENTS: usize = mem::size_of::<WindowManagerHints>() / 4;

        let reply = request_blocking!(
            self.display,
            requests::GetProperty {
                delete: false,
                window,
                property: atoms::predefined::WM_HINTS,
                type_: atoms::predefined::WM_HINTS,
                long_offset: 0,
                long_length: NUM_PROP_WMHINTS_ELEMENTS as u32,
            }
        )?;

        assert!(reply.type_ == atoms::predefined::WM_HINTS);
        assert!(reply.length_of_value == NUM_PROP_WMHINTS_ELEMENTS as u32);

        let raw: [u8; NUM_PROP_WMHINTS_ELEMENTS * 4] = reply.value.try_into().unwrap();
        let raw: [u32; NUM_PROP_WMHINTS_ELEMENTS] = unsafe { mem::transmute(raw) };

        assert!(raw[1] == 0 || raw[1] == 1);

        let hints: WindowManagerHints = unsafe { mem::transmute(raw) };
        Ok(hints)
    }
}

bitmask! {
    #[repr(u32)]
    bitmask WindowManagerHintsFlags {
        /// User specified x, y
        USER_SPECIFIED_POSITION = 0x001,

        /// User specified width, height
        USER_SPECIFIED_SIZE = 0x002,

        /// Program specified position
        PROGRAM_SPECIFIED_POSITION = 0x004,

        /// Program specified size
        PROGRAM_SPECIFIED_SIZE = 0x008,

        /// Program specified minimum size
        PROGRAM_SPECIFIED_MINSIZE = 0x010,

        /// Program specified maximum size
        PROGRAM_SPECIFIED_MAXSIZE = 0x020,

        /// Program specified resize increments
        PROGRAM_SPECIFIED_RESIZE_INCREMENTS = 0x040,

        /// Program specified min and max aspect ratios
        PROGRAM_SPECIFIED_ASPECT = 0x080,

        /// Program specified base for incrementing
        PROGRAM_SPECIFIED_BASESIZE = 0x100,

        /// Program specified window gravity
        PROGRAM_SPECIFIED_WINDOW_GRAVITY = 0x200,
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct WindowManagerHints {
    pub flags: WindowManagerHintsFlags,
    pub input: bool,
    pub initial_state: i32,
    pub icon_pixmap: OrNone<PixmapId>,
    pub icon_window: OrNone<WindowId>,
    pub icon_x: i32,
    pub icon_y: i32,
    pub icon_mask: OrNone<ResourceId>,
    pub window_group: u32,
}
