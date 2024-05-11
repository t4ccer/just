#![allow(dead_code)] // FIXME: Remove

use crate::arguments::Args;
use just_x11::{
    bitmask,
    connection::{DisplayVar, XConnection},
    error::Error,
    extensions::{
        randr::{self},
        render::{self, Fixed},
    },
    requests, OrNone, ResourceId, XDisplay,
};
use std::{collections::HashMap, env, process::ExitCode, str::FromStr};

mod arguments;

#[derive(Debug, Clone)]
struct Monitor {
    name: String,
    set: bool,
    primary: bool,
    outputs: Vec<Output>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    mmwidth: u32,
    mmheight: u32,
}

impl Monitor {
    fn new() -> Self {
        Self {
            name: String::new(),
            set: false,
            primary: false,
            outputs: Vec::new(),
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            mmwidth: 0,
            mmheight: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Filter {
    Bilinear,
    Nearest,
}

impl FromStr for Filter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bilinear" => Ok(Self::Bilinear),
            "nearest" => Ok(Self::Nearest),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct WidthHeight<T> {
    width: T,
    height: T,
}

impl<T> FromStr for WidthHeight<T>
where
    T: FromStr,
{
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (w, h) = s.split_once('x').ok_or(())?;
        let width = T::from_str(w).map_err(|_| ())?;
        let height = T::from_str(h).map_err(|_| ())?;
        Ok(Self { width, height })
    }
}

#[derive(Debug, Clone, Copy)]
struct Gamma {
    red: f32,
    green: f32,
    blue: f32,
}

impl FromStr for Gamma {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(gamma) = f32::from_str(s) {
            if gamma < 0.0 {
                return Err(());
            }

            return Ok(Gamma {
                red: gamma,
                green: gamma,
                blue: gamma,
            });
        }

        let mut split = s.split(':');
        let red = f32::from_str(split.next().ok_or(())?).map_err(|_| ())?;
        let green = f32::from_str(split.next().ok_or(())?).map_err(|_| ())?;
        let blue = f32::from_str(split.next().ok_or(())?).map_err(|_| ())?;
        if split.next().is_some() {
            Err(())
        } else {
            Ok(Gamma { red, green, blue })
        }
    }
}

#[derive(Debug, Clone)]
struct Transform {
    transform: render::Transform,
    filter: Option<Filter>,
    params: Vec<Fixed>,
}

impl Transform {
    fn new() -> Self {
        Self {
            transform: render::Transform {
                matrix: [
                    [Fixed::from(1.0), Fixed::from(0.0), Fixed::from(0.0)],
                    [Fixed::from(0.0), Fixed::from(1.0), Fixed::from(0.0)],
                    [Fixed::from(0.0), Fixed::from(0.0), Fixed::from(1.0)],
                ],
            },
            filter: None,
            params: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct Output {
    refresh: Option<f32>,
    output: Name,
    filter: Option<Filter>,
    crtc: Name,
    mode: Name,
    pos: Option<WidthHeight<i32>>,
    rotation: Option<Rotation>,
    reflection: Option<Reflection>,
    relation: Option<Relation>,
    gamma: Option<Gamma>,
    brightness: Option<f32>,
    primary: bool,
    props: HashMap<String, String>,
    transform: Transform,
    scale_from: Option<WidthHeight<u32>>,
    automatic: bool,
}

impl Output {
    fn new() -> Self {
        Self {
            refresh: None,
            output: Name::empty(),
            filter: None,
            crtc: Name::empty(),
            mode: Name::empty(),
            pos: None,
            rotation: None,
            reflection: None,
            relation: None,
            gamma: None,
            brightness: None,
            primary: false,
            props: HashMap::new(),
            transform: Transform::new(),
            scale_from: None,
            automatic: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Reflection {
    Normal,
    X,
    Y,
    XY,
}

impl FromStr for Reflection {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" => Ok(Self::Normal),
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            "xy" => Ok(Self::XY),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Rotation {
    Normal,
    Inverted,
    Left,
    Right,
}

impl FromStr for Rotation {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "normal" | "0" => Ok(Self::Normal),
            "inverted" | "1" => Ok(Self::Inverted),
            "left" | "2" => Ok(Self::Left),
            "right" | "3" => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

bitmask! {
    #[repr(u8)]
    bitmask NameKind {
        STRING = 1,
        XID = 2,
        INDEX = 4,
        PREFERRED = 8,
    }
}

#[derive(Debug, Clone)]
struct Name {
    kind: NameKind,
    string: String,
    xid: OrNone<ResourceId>,
    index: u32,
}

impl Name {
    fn new_xid(xid: OrNone<ResourceId>) -> Self {
        let mut ret = Self::empty();
        ret.xid = xid;
        ret.kind |= NameKind::XID;
        ret
    }

    fn new(name: String, valid: NameKind) -> Self {
        let mut ret = Self::empty();
        ret.set(name, valid);
        ret
    }

    fn empty() -> Self {
        Self {
            kind: NameKind::EMPTY_MASK,
            string: String::new(),
            xid: OrNone::none(),
            index: 0,
        }
    }

    fn set(&mut self, name: String, valid: NameKind) {
        if valid.has(NameKind::XID) && name.len() > 2 && &name[..2] == "0x" {
            let name = &name[2..];
            if let Ok(xid) = u32::from_str_radix(name, 16) {
                self.xid = OrNone::new(ResourceId::from(xid));
                self.kind |= NameKind::XID;
                return;
            }
        }

        if valid.has(NameKind::INDEX) {
            if let Ok(index) = u32::from_str(&name) {
                self.index = index;
                self.kind |= NameKind::INDEX;
                return;
            }
        }

        if valid.has(NameKind::STRING) {
            self.string = name;
            self.kind |= NameKind::STRING;
            return;
        }

        panic!()
    }
}

#[derive(Debug, Clone)]
enum Relation {
    LeftOf(String),
    RightOf(String),
    Above(String),
    Below(String),
    SameAs(String),
}

#[derive(Debug, Clone, Copy)]
enum ModeAction {
    Create,
    Destroy,
    Add,
    Delete,
}

#[derive(Debug, Clone)]
struct XRRModeInfo {
    id: OrNone<ResourceId>,
    width: u32,
    height: u32,
    dot_clock: u64,
    h_sync_start: u32,
    h_sync_end: u32,
    h_total: u32,
    h_skew: u32,
    v_sync_start: u32,
    v_sync_end: u32,
    v_total: u32,
    name: String,
    mode_flags: u64,
}

impl XRRModeInfo {
    fn new() -> Self {
        Self {
            id: OrNone::none(),
            width: 0,
            height: 0,
            dot_clock: 0,
            h_sync_start: 0,
            h_sync_end: 0,
            h_total: 0,
            h_skew: 0,
            v_sync_start: 0,
            v_sync_end: 0,
            v_total: 0,
            name: String::new(),
            mode_flags: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct Mode {
    name: Name,
    output: Name,
    action: ModeAction,
    mode: XRRModeInfo,
}

fn run(args: Args) -> Result<(), Error> {
    // dbg!(&args);

    if args.version {
        println!("xrandr program version       {}", env!("CARGO_PKG_VERSION"))
    }

    let mut display = if let Some(display_name) = args.display_name {
        let connection = XConnection::with_display(DisplayVar::from_str(&display_name)?)?;
        XDisplay::with_connection(connection)?
    } else {
        XDisplay::open()?
    };

    let screen = if let Some(screen) = args.screen {
        let no_screens = display.screens().len();
        if screen as usize > no_screens {
            eprintln!(
                "Invalid screen number {} (display has {})",
                screen, no_screens
            );
            panic!();
        }
        screen
    } else {
        0
    };

    let root = display.screens()[screen as usize].root;

    let randr_query = {
        let randr_query_pending = display.send_request(&requests::QueryExtension {
            name: randr::EXTENSION_NAME.to_vec(),
        })?;
        display.flush()?;
        display.await_pending_reply(randr_query_pending)?.unwrap()
    };
    if !randr_query.present {
        eprintln!("RandR extension missing\n");
        panic!();
    }

    macro_rules! send_randr_request {
        ($request:expr) => {{
            let pending = display.send_extension_request($request, randr_query.major_opcode)?;
            display.await_pending_reply(pending)?.unwrap()
        }};
    }

    let randr_version = send_randr_request!(&randr::requests::QueryVersion {
        major_version: randr::SUPPORTED_MAJOR,
        minor_version: randr::SUPPORTED_MINOR,
    });

    if args.version {
        println!(
            "Server reports RandR version {}.{}",
            randr_version.major_version, randr_version.minor_version
        );
    }

    let has_1_5 = randr_version.major_version > 1
        || (randr_version.major_version == 1 && randr_version.minor_version >= 5);
    assert!(has_1_5, "RandR version below 1.5 not supported"); // TODO: Add support

    // TODO: has_1_2 check
    if args.modeit || true {
        let _screen_size_range =
            send_randr_request!(&randr::requests::GetScreenSizeRange { window: root });

        let _screen_resources =
            send_randr_request!(&randr::requests::GetScreenResourcesCurrent { window: root });

        // for crtc in screen_resources.crtcs.iter().copied() {
        //     let crtc_info = send_randr_request!(&randr::requests::GetCrtcInfo {
        //         crtc,
        //         timestamp: screen_resources.config_timestamp,
        //     });
        //     dbg!(crtc_info);
        // }

        for mode in &args.modes {
            match mode.action {
                ModeAction::Create => todo!(),
                ModeAction::Destroy => todo!(),
                ModeAction::Add => todo!(),
                ModeAction::Delete => todo!(),
            }
        }
    }

    Ok(())
}

fn main() -> ExitCode {
    match Args::from_cli(env::args()) {
        Ok(args) => match run(args) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("xrandr: {}", err);
                ExitCode::FAILURE
            }
        },
        Err(err) => {
            eprintln!("xrandr: {:?}", err); // TODO: Formatter
            ExitCode::FAILURE
        }
    }
}
