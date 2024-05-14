use crate::{backend::Backend, Event, Result, Vector2, BYTES_PER_PIXEL};
use core::cmp;
use just_shared_memory::SharedMemory;
use just_x11::{
    atoms::AtomId,
    events::EventType,
    extensions::mit_shm::{self, ShmSegId},
    replies::String8,
    requests::{GContextSettings, PutImageFormat, WindowCreationAttributes},
    Drawable, GContextId, WindowClass, WindowId, WindowVisual, XDisplay,
};

// TODO: This should use double buffering

struct MitShmCanvas {
    mem: SharedMemory,
    size: Vector2<u32>,
    shmseg: ShmSegId,
}

impl MitShmCanvas {
    #[inline]
    fn new(size: Vector2<u32>, shmseg: ShmSegId) -> Self {
        let mem = SharedMemory::zeroed(size.x * size.y * BYTES_PER_PIXEL);

        Self { mem, size, shmseg }
    }

    #[inline]
    fn mem_mut(&mut self) -> &mut [u8] {
        unsafe { self.mem.data_mut() }
    }

    #[inline]
    fn mem(&self) -> &[u8] {
        unsafe { self.mem.data() }
    }
}

pub(crate) struct X11MitShmBackend {
    display: XDisplay,
    mit_shm_major_opcode: u8,
    canvas: MitShmCanvas,
    window: WindowId,
    gc: GContextId,
    wm_delete_window: AtomId,
}

impl X11MitShmBackend {
    pub(crate) fn new(title: &str) -> Result<Self> {
        use just_x11::requests;

        let mut display = XDisplay::open()?;
        let mit_shm = {
            let pending_reply = display.send_request(&requests::QueryExtension {
                name: mit_shm::EXTENSION_NAME.to_vec(),
            })?;
            let reply = display.await_pending_reply(pending_reply)?;
            reply.unwrap()
        };

        // TODO: Graceful error
        assert!(mit_shm.present);

        let mit_shm_major_opcode = mit_shm.major_opcode;

        let canvas_size = Vector2 { x: 800, y: 600 };
        let canvas = Self::attach_new_shm_seg(&mut display, mit_shm_major_opcode, canvas_size)?;

        // create window

        let window = {
            let window_id = WindowId::from(display.id_allocator().allocate_id());
            let window_attributes = WindowCreationAttributes::new().set_event_mask(
                EventType::KEY_PRESS
                    | EventType::KEY_RELEASE
                    | EventType::BUTTON_PRESS
                    | EventType::BUTTON_RELEASE
                    | EventType::POINTER_MOTION
                    | EventType::STRUCTURE_NOTIFY,
            );
            let create_window = requests::CreateWindow {
                depth: 24,
                wid: window_id,
                parent: display.screens()[0].root,
                x: 0,
                y: 0,
                width: 600,
                height: 800,
                border_width: 0,
                window_class: WindowClass::CopyFromParent,
                visual: WindowVisual::CopyFromParent,
                attributes: window_attributes,
            };
            display.send_request(&create_window)?;
            window_id
        };

        let gc = {
            let gc_id = GContextId::from(display.id_allocator().allocate_id());
            display.send_request(&requests::CreateGC {
                cid: gc_id,
                drawable: Drawable::Window(window),
                values: GContextSettings::new(),
            })?;
            gc_id
        };

        display.send_request(&requests::MapWindow { window })?;
        display.flush()?;

        // setup window closing "handler"

        let wm_protocols = {
            let pending = display.send_request(&requests::InternAtom {
                only_if_exists: false,
                name: String8::from_bytes(b"WM_PROTOCOLS".to_vec()).unwrap(),
            })?;
            display.flush()?;
            display.await_pending_reply(pending)?.unwrap().atom
        };

        let wm_delete_window = {
            let pending = display.send_request(&requests::InternAtom {
                only_if_exists: false,
                name: String8::from_bytes(b"WM_DELETE_WINDOW".to_vec()).unwrap(),
            })?;
            display.flush()?;
            display.await_pending_reply(pending)?.unwrap().atom
        };

        display.send_request(&requests::ChangeProperty {
            mode: requests::ChangePropertyMode::Replace,
            window,
            property: wm_protocols,
            type_: AtomId::ATOM,
            format: requests::ChangePropertyFormat::Format32,
            data: wm_delete_window.to_le_bytes().to_vec(),
        })?;

        // set window name

        let wm_name = {
            let pending = display.send_request(&requests::InternAtom {
                only_if_exists: false,
                name: String8::from_bytes(b"WM_NAME".to_vec()).unwrap(),
            })?;
            display.flush()?;
            display.await_pending_reply(pending)?.unwrap().atom
        };

        display.send_request(&requests::ChangeProperty {
            mode: requests::ChangePropertyMode::Replace,
            window,
            property: wm_name,
            type_: AtomId::STRING,
            format: requests::ChangePropertyFormat::Format8,
            data: title.as_bytes().to_vec(),
        })?;

        display.flush()?;

        Ok(Self {
            display,
            mit_shm_major_opcode,
            canvas,
            window,
            gc,
            wm_delete_window,
        })
    }
}

impl Backend for X11MitShmBackend {
    fn flush_window(&mut self) -> Result<()> {
        self.display.send_extension_request(
            &mit_shm::requests::PutImage {
                drawable: Drawable::Window(self.window),
                gc: self.gc,
                total_width: self.canvas.size.x as u16,
                total_height: self.canvas.size.y as u16,
                src_x: 0,
                src_y: 0,
                src_width: self.canvas.size.x as u16,
                src_height: self.canvas.size.y as u16,
                dst_x: 0,
                dst_y: 0,
                depth: 24,
                format: PutImageFormat::ZPixmap,
                send_event: false, // should be true for double buffering tracking?
                bpad: 0,
                shmseg: self.canvas.shmseg,
                offset: 0,
            },
            self.mit_shm_major_opcode,
        )?;
        self.display.flush()?;

        Ok(())
    }

    fn events(&mut self) -> Result<Vec<Event>> {
        use just_x11::events::SomeEvent;

        macro_rules! x_to_u32 {
            ($original:expr) => {
                if $original < 0 {
                    0
                } else {
                    let res = $original as u32;
                    if res > self.canvas.size.x {
                        self.canvas.size.x
                    } else {
                        res
                    }
                }
            };
        }

        macro_rules! y_to_u32 {
            ($original:expr) => {
                if $original < 0 {
                    0
                } else {
                    let res = $original as u32;
                    if res > self.canvas.size.y {
                        self.canvas.size.y
                    } else {
                        res
                    }
                }
            };
        }

        let mut events = Vec::new();

        // TODO: Keyboard events
        for event in self.display.events()? {
            match event {
                SomeEvent::ConfigureNotify(event) => {
                    if event.event == self.window {
                        events.push(Event::Resize {
                            new_size: Vector2 {
                                x: event.width as u32,
                                y: event.height as u32,
                            },
                        });
                    }
                }
                SomeEvent::ButtonPress(event) => {
                    // FIXME: event.root and event.event should be WindowId
                    if event.event == self.window.into() {
                        events.push(Event::ButtonPress {
                            button: event.detail.raw(),
                        });
                    }
                }
                SomeEvent::ButtonRelease(event) => {
                    if event.event == self.window.into() {
                        events.push(Event::ButtonRelease {
                            button: event.detail.raw(),
                        });
                    }
                }
                SomeEvent::MotionNotify(event) => {
                    if event.event == self.window {
                        events.push(Event::PointerMotion {
                            position: Vector2 {
                                x: x_to_u32!(event.event_x),
                                y: y_to_u32!(event.event_y),
                            },
                        });
                    }
                }
                SomeEvent::ClientMessage(event) => {
                    let val = u32::from_le_bytes([
                        event.data[0],
                        event.data[1],
                        event.data[2],
                        event.data[3],
                    ]);
                    if val == self.wm_delete_window.into() {
                        events.push(Event::Shutdown);
                    }
                }
                _event => {}
            }
        }

        Ok(events)
    }

    fn resize(&mut self, new_size: Vector2<u32>) -> Result<()> {
        let old_buf = self.canvas.mem_mut().to_vec();
        let old_size = self.canvas.size;

        if new_size.x * new_size.y * BYTES_PER_PIXEL <= self.canvas.mem.size() {
            self.canvas.mem_mut().fill(0);
            self.canvas.size = new_size;
        } else {
            self.display.send_extension_request(
                &mit_shm::requests::Detach {
                    shmseg: self.canvas.shmseg,
                },
                self.mit_shm_major_opcode,
            )?;

            let new_canvas =
                Self::attach_new_shm_seg(&mut self.display, self.mit_shm_major_opcode, new_size)?;
            self.display.flush()?;
            let old_canvas = core::mem::replace(&mut self.canvas, new_canvas);
            unsafe { old_canvas.mem.free() }
        }

        let new_buf = self.canvas.mem_mut();
        for y in 0..cmp::min(new_size.y, old_size.y) {
            for x in 0..cmp::min(new_size.x, old_size.x) {
                let new_offset = (new_size.x * y + x) as usize * BYTES_PER_PIXEL as usize;
                let old_offset = (old_size.x * y + x) as usize * BYTES_PER_PIXEL as usize;
                new_buf[new_offset + 0] = old_buf[old_offset + 0];
                new_buf[new_offset + 1] = old_buf[old_offset + 1];
                new_buf[new_offset + 2] = old_buf[old_offset + 2];
                new_buf[new_offset + 3] = old_buf[old_offset + 3];
            }
        }

        Ok(())
    }

    #[inline]
    fn size(&self) -> Vector2<u32> {
        self.canvas.size
    }

    #[inline]
    fn buf_mut(&mut self) -> &mut [u8] {
        self.canvas.mem_mut()
    }

    #[inline]
    fn buf(&self) -> &[u8] {
        self.canvas.mem()
    }
}

impl X11MitShmBackend {
    fn attach_new_shm_seg(
        display: &mut XDisplay,
        mit_shm_major_opcode: u8,
        size: Vector2<u32>,
    ) -> Result<MitShmCanvas> {
        let new_shmseg = ShmSegId::from(display.id_allocator().allocate_id());
        let new_canvas = MitShmCanvas::new(size, new_shmseg);
        display.send_extension_request(
            &mit_shm::requests::Attach {
                shmseg: new_shmseg,
                shmid: new_canvas.mem.id().inner() as u32,
                read_only: false,
            },
            mit_shm_major_opcode,
        )?;
        Ok(new_canvas)
    }
}
