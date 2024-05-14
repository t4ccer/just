// CLIPPY CONFIG
#![allow(
    clippy::new_without_default,
    clippy::unnecessary_cast,
    clippy::identity_op
)]

use just_x11::{
    error::Error,
    extensions::mit_shm::{self, ShmSegId},
    requests, Drawable, XDisplay,
};
use std::{
    fs::File,
    io::{self, BufWriter, Write},
    path::Path,
};

// Port of https://gist.github.com/rexim/2febe9f5a5376b476d33d5d16590ecfd

fn bits_per_pixel(depth: u8) -> u8 {
    if depth <= 4 {
        4
    } else if depth <= 8 {
        8
    } else if depth <= 16 {
        16
    } else {
        32
    }
}

fn save_as_ppm<P>(img: &[u8], width: u16, height: u16, filepath: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    let mut f = BufWriter::new(File::create(filepath)?);
    writeln!(f, "P6")?;
    writeln!(f, "{} {}", width, height)?;
    writeln!(f, "255")?;

    for y in 0..(height as usize) {
        for x in 0..(width as usize) {
            f.write_all(&[img[(y * width as usize + x) * 4 + 2]])?;
            f.write_all(&[img[(y * width as usize + x) * 4 + 1]])?;
            f.write_all(&[img[(y * width as usize + x) * 4 + 0]])?;
        }
    }

    f.flush()
}

fn go() -> Result<(), Error> {
    let mut display = XDisplay::open()?;

    macro_rules! send_request {
        ($request:expr) => {{
            let pending_reply = display.send_request(&($request))?;
            let reply = display.await_pending_reply(pending_reply)?;
            reply.unwrap()
        }};
    }

    let mit_shm = send_request!(requests::QueryExtension {
        name: mit_shm::EXTENSION_NAME.to_vec()
    });

    assert!(mit_shm.present);

    macro_rules! send_mit_shm_request_reply {
        ($request:expr) => {{
            let pending = display.send_extension_request($request, mit_shm.major_opcode)?;
            display.await_pending_reply(pending)?.unwrap()
        }};
    }

    macro_rules! send_mit_shm_request {
        ($request:expr) => {{
            display.send_extension_request($request, mit_shm.major_opcode)?;
        }};
    }

    let version = send_mit_shm_request_reply!(&mit_shm::requests::QueryVersion);

    let root_window = display.screens()[0].root;

    println!(
        "SHM Version {}.{}, Pixmaps supported: {}",
        version.major_version,
        version.minor_version,
        if version.shared_pixmaps {
            "yesu"
        } else {
            "nah"
        }
    );

    let depth = 24;

    let geometry = send_request!(requests::GetGeometry {
        drawable: Drawable::Window(root_window)
    });

    // NOTE: This is not 100% correct, should compute bytes per scanline using pixmap_format
    let size = geometry.width as u32 * geometry.height as u32 * (bits_per_pixel(depth) as u32 / 8);
    let shm = just_shared_memory::SharedMemory::zeroed(size);

    let shmseg = ShmSegId::from(display.id_allocator().allocate_id());

    send_mit_shm_request!(&mit_shm::requests::Attach {
        shmseg,
        shmid: shm.id().inner() as u32,
        read_only: false,
    });

    send_mit_shm_request_reply!(&mit_shm::requests::GetImage {
        drawable: Drawable::Window(root_window),
        x: 0,
        y: 0,
        width: geometry.width,
        height: geometry.height,
        plane_mask: -1, // AllPlanes
        format: 2,      // ZPixmap
        shmseg,
        offset: 0,
    });

    unsafe {
        save_as_ppm(
            shm.data(),
            geometry.width,
            geometry.height,
            "screenshot.ppm",
        )
        .unwrap();
    }

    send_mit_shm_request!(&mit_shm::requests::Detach { shmseg });

    unsafe { shm.free() }

    Ok(())
}

fn main() {
    go().unwrap();
}
