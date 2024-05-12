use crate::{backend::Backend, Event, Result, BYTES_PER_PIXEL};
use just_shared_memory::SharedMemory;
use just_x11::{
    events::EventType,
    extensions::mit_shm::{self, ShmSegId},
    requests::{GContextSettings, WindowCreationAttributes},
    Drawable, GContextId, WindowClass, WindowId, WindowVisual, XDisplay,
};

struct MitShmCanvas {
    mem: SharedMemory,
    width: u32,
    height: u32,
    shmseg: ShmSegId,
}

impl MitShmCanvas {
    fn new(width: u32, height: u32, shmseg: ShmSegId) -> Self {
        let mem = SharedMemory::zeroed(width * height * BYTES_PER_PIXEL);

        Self {
            mem,
            width,
            height,
            shmseg,
        }
    }

    fn mem_mut(&mut self) -> &mut [u8] {
        unsafe { self.mem.data_mut() }
    }
}

pub(crate) struct X11MitShmBackend {
    display: XDisplay,
    mit_shm_major_opcode: u8,
    canvas: MitShmCanvas,
    window: WindowId,
    gc: GContextId,
}

impl Backend for X11MitShmBackend {
    fn new(_title: &str) -> Result<Self> {
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

        let canvas = Self::attach_new_shm_seg(&mut display, mit_shm_major_opcode, 800, 600)?;

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

        Ok(Self {
            display,
            mit_shm_major_opcode,
            canvas,
            window,
            gc,
        })
    }

    fn flush_window(&mut self) -> Result<()> {
        self.display.send_extension_request(
            &mit_shm::requests::PutImage {
                drawable: Drawable::Window(self.window),
                gc: self.gc,
                total_width: self.canvas.width as u16,
                total_height: self.canvas.height as u16,
                src_x: 0,
                src_y: 0,
                src_width: self.canvas.width as u16,
                src_height: self.canvas.height as u16,
                dst_x: 0,
                dst_y: 0,
                depth: 24,
                format: 2,     // ZPixmap
                send_event: 0, // FIXME: Should be bool
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
                    if res > self.canvas.width {
                        self.canvas.width
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
                    if res > self.canvas.height {
                        self.canvas.height
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
                            new_width: event.width as u32,
                            new_height: event.height as u32,
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
                            x: x_to_u32!(event.event_x),
                            y: y_to_u32!(event.event_y),
                        });
                    }
                }
                _event => {}
            }
        }

        Ok(events)
    }

    fn resize(&mut self, new_width: u32, new_height: u32) -> Result<()> {
        if new_width * new_height * BYTES_PER_PIXEL <= self.canvas.mem.size() {
            self.canvas.width = new_width;
            self.canvas.height = new_height;
            return Ok(());
        }

        self.display.send_extension_request(
            &mit_shm::requests::Detach {
                shmseg: self.canvas.shmseg,
            },
            self.mit_shm_major_opcode,
        )?;

        let new_canvas = Self::attach_new_shm_seg(
            &mut self.display,
            self.mit_shm_major_opcode,
            new_width,
            new_height,
        )?;
        self.display.flush()?;
        let old_canvas = core::mem::replace(&mut self.canvas, new_canvas);
        unsafe { old_canvas.mem.free() }

        Ok(())
    }

    fn size(&self) -> (u32, u32) {
        (self.canvas.width, self.canvas.height)
    }

    fn buf_mut(&mut self) -> &mut [u8] {
        self.canvas.mem_mut()
    }
}

impl X11MitShmBackend {
    fn attach_new_shm_seg(
        display: &mut XDisplay,
        mit_shm_major_opcode: u8,
        width: u32,
        height: u32,
    ) -> Result<MitShmCanvas> {
        let new_shmseg = ShmSegId::from(display.id_allocator().allocate_id());
        let new_canvas = MitShmCanvas::new(width, height, new_shmseg);
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

// TODO: Detect closing by WM
