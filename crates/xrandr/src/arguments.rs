use crate::{
    Filter, Gamma, Mode, ModeAction, Monitor, Name, NameKind, Output, Reflection, Relation,
    Rotation, Transform, WidthHeight, XRRModeInfo,
};
use justshow_x11::{extensions::render::Fixed, OrNone};
use std::str::FromStr;

#[derive(Debug)]
pub enum InvalidCliArgs {
    InvalidFlag(String),
    MissingArgument(String),
    CouldNotParse(String, String),
    MustBeAfterOutput(String),
    NegativeF32(String, f32),
}

#[derive(Debug)]
pub struct Args {
    pub program_name: String,
    pub display_name: Option<String>,
    pub help: bool,
    pub verbose: bool,
    pub dry_run: bool,
    pub grab_server: bool,
    pub current: bool,
    pub setit: bool,
    pub setit_1_2: bool,
    pub version: bool,
    pub toggle_x: bool,
    pub toggle_y: bool,
    pub action_requested: bool,
    pub modeit: bool,
    pub query: bool,
    pub query_1_2: bool,
    pub query_1: bool,
    pub properties: bool,
    pub provider_name: Option<Name>,
    pub output_source_provider_name: Option<Name>,
    pub provsetoffsink: bool,
    pub offload_sink_provider_name: Option<Name>,
    pub list_providers: bool,
    pub provsetoutsource: bool,
    pub width_height: Option<WidthHeight<u32>>,
    pub size: Option<u32>,
    pub rate: Option<f32>,
    pub screen: Option<u32>,
    pub rot: Option<Rotation>,
    pub all_outputs: Vec<Output>,
    pub focused_output: Option<usize>,
    pub no_primary: bool,
    pub framebuffer: Option<WidthHeight<u32>>,
    pub framebuffer_physical: Option<WidthHeight<u32>>,
    pub dpi: Option<f32>,
    pub dpi_output_name: Option<String>,
    pub automatic: bool,
    pub modes: Vec<Mode>,
    pub list_monitors: bool,
    pub list_active_monitors: bool,
    pub monitors: Vec<Monitor>,
    pub monitorit: bool,
}

impl Args {
    fn find_output(&mut self, name: Name) -> Option<&mut Output> {
        for (idx, output) in self.all_outputs.iter_mut().enumerate() {
            let common = name.kind & output.output.kind;

            if common.has(NameKind::XID) && name.xid == output.output.xid {
                self.focused_output = Some(idx);
                return Some(output);
            }

            if common.has(NameKind::STRING) && name.string == output.output.string {
                self.focused_output = Some(idx);
                return Some(output);
            }

            if common.has(NameKind::INDEX) && name.index == output.output.index {
                self.focused_output = Some(idx);
                return Some(output);
            }
        }

        self.focused_output = None;
        None
    }

    fn find_output_by_name(&mut self, name: String) -> Option<&mut Output> {
        let output_name = Name {
            kind: NameKind::STRING,
            string: name,
            xid: OrNone::none(),
            index: 0,
        };
        self.find_output(output_name)
    }

    fn find_focused_output(&mut self) -> Option<&mut Output> {
        let idx = self.focused_output?;
        Some(&mut self.all_outputs[idx])
    }

    fn add_output(&mut self) -> &mut Output {
        let new_output = Output::new();
        let idx = self.all_outputs.len();
        self.all_outputs.push(new_output);
        self.focused_output = Some(idx);
        &mut self.all_outputs[idx]
    }

    pub fn from_cli(mut raw_args: impl Iterator<Item = String>) -> Result<Self, InvalidCliArgs> {
        let program_name = raw_args.next().unwrap();

        let mut args = Args {
            program_name,
            display_name: None,
            help: false,
            verbose: false,
            dry_run: false,
            grab_server: true,
            current: false,
            setit: false,
            setit_1_2: false,
            version: false,
            toggle_x: false,
            toggle_y: false,
            action_requested: false,
            query: false,
            query_1: false,
            query_1_2: false,
            properties: false,
            provider_name: None,
            list_providers: false,
            width_height: None,
            size: None,
            rate: None,
            screen: None,
            rot: None,
            all_outputs: Vec::new(),
            focused_output: None,
            no_primary: false,
            framebuffer: None,
            framebuffer_physical: None,
            dpi_output_name: None,
            dpi: None,
            automatic: false,
            modes: Vec::new(),
            output_source_provider_name: None,
            provsetoutsource: false,
            provsetoffsink: false,
            offload_sink_provider_name: None,
            list_monitors: false,
            list_active_monitors: false,
            monitors: Vec::new(),
            monitorit: false,
            modeit: false,
        };

        loop {
            let Some(arg) = raw_args.next() else {
                break;
            };

            macro_rules! get_next_arg {
                () => {
                    match raw_args.next() {
                        Some(value) => value,
                        None => return Err(InvalidCliArgs::MissingArgument(arg)),
                    }
                };
            }

            macro_rules! parse {
                ($t:ident, $value:expr) => {
                    match $t::from_str($value.as_str()) {
                        Ok(parsed) => parsed,
                        Err(_) => return Err(InvalidCliArgs::CouldNotParse(arg, $value)),
                    }
                };
            }

            macro_rules! after_output {
                () => {
                    match args.find_focused_output() {
                        Some(output) => output,
                        None => return Err(InvalidCliArgs::MustBeAfterOutput(arg)),
                    }
                };
            }

            match arg.as_str() {
                "--display" | "-display" | "-d" => {
                    let display_name = get_next_arg!();
                    args.display_name = Some(display_name);
                }
                "-help" | "--help" => args.help = true,
                "--verbose" => args.verbose = true,
                "--dryrun" => {
                    args.dry_run = true;
                    args.verbose = true;
                }
                "--nograb" => args.grab_server = false,
                "--current" => args.current = true,
                "--size" | "-s" => {
                    let size = get_next_arg!();

                    if let Ok(wh) = WidthHeight::from_str(&size) {
                        args.width_height = Some(wh);
                        args.setit = true;
                        args.action_requested = true;
                        continue;
                    }

                    if let Ok(s) = u32::from_str(&size) {
                        args.size = Some(s);
                        args.setit = true;
                        args.action_requested = true;
                        continue;
                    }

                    return Err(InvalidCliArgs::CouldNotParse(arg, size));
                }
                "--refresh" | "--rate" | "-r" => {
                    let rate = get_next_arg!();
                    let rate = Some(parse!(f32, rate));
                    args.rate = rate;
                    args.setit = true;

                    if let Some(config_output) = args.find_focused_output() {
                        config_output.refresh = rate;
                        args.setit_1_2 = true;
                    }
                }
                "--version" | "-v" => {
                    args.version = true;
                    args.action_requested = true;
                }
                "-x" => {
                    args.toggle_x = true;
                    args.setit = true;
                }
                "-y" => {
                    args.toggle_y = true;
                    args.setit = true;
                }
                "--screen" => {
                    let screen = get_next_arg!();
                    args.screen = Some(parse!(u32, screen));
                }
                "--query" | "-q" => {
                    args.query = true;
                }
                "--orientation" | "-o" => {
                    let dirind = get_next_arg!();
                    args.rot = Some(parse!(Rotation, dirind));
                    args.setit = true;
                    args.action_requested = true;
                }
                "--properties" | "--madprops" | "--props" | "--prop" => {
                    args.query_1_2 = true;
                    args.properties = true;
                    args.action_requested = true;
                }
                "--output" => {
                    let output_name = get_next_arg!();
                    if args.find_output_by_name(output_name.clone()).is_none() {
                        let new_output = args.add_output();
                        new_output
                            .output
                            .set(output_name, NameKind::STRING | NameKind::XID);
                    }

                    args.setit_1_2 = true;
                    args.action_requested = true;
                }
                "--filter" => {
                    let output = after_output!();
                    let filter = get_next_arg!();
                    output.filter = Some(parse!(Filter, filter));
                }
                "--crtc" => {
                    let output = after_output!();
                    let crtc = get_next_arg!();
                    output.crtc.set(crtc, NameKind::INDEX | NameKind::XID);
                }
                "--mode" => {
                    let output = after_output!();
                    let mode = get_next_arg!();
                    output.mode.set(mode, NameKind::STRING | NameKind::XID);
                }
                "--preferred" => {
                    let output = after_output!();
                    output.mode.kind |= NameKind::PREFERRED;
                }
                "--pos" => {
                    let output = after_output!();
                    let pos = get_next_arg!();
                    let pos = parse!(WidthHeight, pos);
                    output.pos = Some(pos);
                }
                "--rotation" | "--rotate" => {
                    let output = after_output!();
                    let rotation = get_next_arg!();
                    output.rotation = Some(parse!(Rotation, rotation));
                }
                "--reflection" | "--reflect" => {
                    let output = after_output!();
                    let reflection = get_next_arg!();
                    output.reflection = Some(parse!(Reflection, reflection));
                }
                "--left-of" => {
                    let output = after_output!();
                    let relative_to = get_next_arg!();
                    output.relation = Some(Relation::LeftOf(relative_to));
                }
                "--right-of" => {
                    let output = after_output!();
                    let relative_to = get_next_arg!();
                    output.relation = Some(Relation::RightOf(relative_to));
                }
                "--above" => {
                    let output = after_output!();
                    let relative_to = get_next_arg!();
                    output.relation = Some(Relation::Above(relative_to));
                }
                "--below" => {
                    let output = after_output!();
                    let relative_to = get_next_arg!();
                    output.relation = Some(Relation::Below(relative_to));
                }
                "--same-as" => {
                    let output = after_output!();
                    let relative_to = get_next_arg!();
                    output.relation = Some(Relation::SameAs(relative_to));
                }
                "--panning" => todo!(),
                "--gamma" => {
                    let output = after_output!();
                    let gamma = get_next_arg!();
                    output.gamma = Some(parse!(Gamma, gamma));
                }
                "--brightness" => {
                    let output = after_output!();
                    let brightness = get_next_arg!();
                    output.brightness = Some(parse!(f32, brightness));
                    args.setit_1_2 = true;
                }
                "--primary" => {
                    let output = after_output!();
                    output.primary = true;
                }
                "--noprimary" => {
                    args.no_primary = true;
                    args.setit_1_2 = true;
                }
                "--set" => {
                    let output = after_output!();
                    let name = get_next_arg!();
                    let value = get_next_arg!();
                    output.props.insert(name, value);
                }
                "--scale" => {
                    let output = after_output!();
                    let scale = get_next_arg!();
                    let scale: WidthHeight<f32> = parse!(WidthHeight, scale);

                    output.transform.transform.matrix[0][0] = Fixed::from(scale.width);
                    output.transform.transform.matrix[1][1] = Fixed::from(scale.height);
                    output.transform.transform.matrix[2][2] = Fixed::from(1.0);

                    let filter = if scale.width != 1.0 || scale.height != 1.0 {
                        Filter::Bilinear
                    } else {
                        Filter::Nearest
                    };
                    output.transform.filter = Some(filter);
                    output.transform.params = Vec::new();
                }
                "--scale-from" => {
                    let output = after_output!();
                    let scale = get_next_arg!();
                    let scale: WidthHeight<u32> = parse!(WidthHeight, scale);

                    output.scale_from = Some(scale);
                }
                "--transform" => {
                    let output = after_output!();
                    let transform = get_next_arg!();

                    output.transform = Transform::new();

                    if transform.as_str() == "none" {
                        continue;
                    }

                    let mut input = transform.as_str().split(',');
                    for y in 0..output.transform.transform.matrix.len() {
                        for x in 0..output.transform.transform.matrix[0].len() {
                            let raw = input.next().ok_or(InvalidCliArgs::CouldNotParse(
                                arg.clone(),
                                transform.clone(),
                            ))?;
                            output.transform.transform.matrix[y][x] =
                                Fixed::from(parse!(f32, raw.to_string()))
                        }
                    }
                    if input.next().is_some() {
                        return Err(InvalidCliArgs::CouldNotParse(
                            arg.clone(),
                            transform.clone(),
                        ));
                    }

                    output.transform.filter = Some(Filter::Bilinear);
                    output.transform.params = Vec::new();
                }
                "--off" => {
                    let output = after_output!();
                    output.mode.kind |= NameKind::XID;
                    output.mode.xid = OrNone::none();

                    output.crtc.kind |= NameKind::XID;
                    output.crtc.xid = OrNone::none();
                }
                "--fb" => {
                    let fb = get_next_arg!();
                    let fb = parse!(WidthHeight, fb);
                    args.framebuffer = Some(fb);
                }
                "--fbmm" => {
                    let fb = get_next_arg!();
                    let fb = parse!(WidthHeight, fb);
                    args.framebuffer_physical = Some(fb);
                }
                "--dpi" => {
                    let dpi = get_next_arg!();
                    match f32::from_str(dpi.as_str()) {
                        Err(_) => {
                            args.dpi = None;
                            args.dpi_output_name = Some(arg);
                        }
                        Ok(dpi) => {
                            args.dpi = Some(dpi);
                        }
                    }
                    args.setit_1_2 = true;
                    args.action_requested = true;
                }
                "--auto" => {
                    match args.find_focused_output() {
                        None => args.automatic = true,
                        Some(output) => output.automatic = true,
                    }
                    args.setit_1_2 = true;
                    args.action_requested = true;
                }
                "--q12" => {
                    args.query_1_2 = true;
                }
                "--q1" => {
                    args.query_1 = true;
                }
                "--newmode" => {
                    let name = get_next_arg!();

                    let clock = get_next_arg!();
                    let clock = parse!(f64, clock);

                    let width = get_next_arg!();
                    let width = parse!(u32, width);

                    let h_sync_start = get_next_arg!();
                    let h_sync_start = parse!(u32, h_sync_start);

                    let h_sync_end = get_next_arg!();
                    let h_sync_end = parse!(u32, h_sync_end);

                    let h_total = get_next_arg!();
                    let h_total = parse!(u32, h_total);

                    let height = get_next_arg!();
                    let height = parse!(u32, height);

                    let v_sync_start = get_next_arg!();
                    let v_sync_start = parse!(u32, v_sync_start);

                    let v_sync_end = get_next_arg!();
                    let v_sync_end = parse!(u32, v_sync_end);

                    let v_total = get_next_arg!();
                    let v_total = parse!(u32, v_total);

                    let mode_flags = 0;

                    let mode = Mode {
                        name: Name::empty(),
                        output: Name::empty(),
                        action: ModeAction::Create,
                        mode: XRRModeInfo {
                            id: OrNone::none(),
                            width,
                            height,
                            dot_clock: (clock * 1_000_000.0) as u64,
                            h_sync_start,
                            h_sync_end,
                            h_total,
                            h_skew: 0,
                            v_sync_start,
                            v_sync_end,
                            v_total,
                            name,
                            mode_flags,
                        },
                    };

                    args.modes.push(mode);
                    args.modeit = true;
                    args.action_requested = true;
                }
                "--rmmode" => {
                    let name = get_next_arg!();

                    let mode = Mode {
                        name: Name::new(name, NameKind::STRING | NameKind::XID),
                        output: Name::empty(),
                        action: ModeAction::Destroy,
                        mode: XRRModeInfo::new(),
                    };

                    args.modes.push(mode);
                    args.modeit = true;
                    args.action_requested = true;
                }
                "--addmode" => {
                    let output = get_next_arg!();
                    let name = get_next_arg!();

                    let mode = Mode {
                        name: Name::new(name, NameKind::STRING | NameKind::XID),
                        output: Name::new(output, NameKind::STRING | NameKind::XID),
                        action: ModeAction::Add,
                        mode: XRRModeInfo::new(),
                    };

                    args.modes.push(mode);
                    args.modeit = true;
                    args.action_requested = true;
                }
                "--delmode" => {
                    let output = get_next_arg!();
                    let name = get_next_arg!();

                    let mode = Mode {
                        name: Name::new(name, NameKind::STRING | NameKind::XID),
                        output: Name::new(output, NameKind::STRING | NameKind::XID),
                        action: ModeAction::Delete,
                        mode: XRRModeInfo::new(),
                    };

                    args.modes.push(mode);
                    args.modeit = true;
                    args.action_requested = true;
                }
                "--listproviders" => {
                    args.list_providers = true;
                    args.action_requested = true;
                }
                "--setprovideroutputsource" => {
                    let provider_name = get_next_arg!();
                    args.provider_name = Some(Name::new(
                        provider_name,
                        NameKind::STRING | NameKind::XID | NameKind::INDEX,
                    ));

                    let output_source_provider_name = match raw_args.next() {
                        Some(output_source_provider_name) => Name::new(
                            output_source_provider_name,
                            NameKind::STRING | NameKind::XID | NameKind::INDEX,
                        ),
                        None => Name::new_xid(OrNone::none()),
                    };
                    args.output_source_provider_name = Some(output_source_provider_name);

                    args.action_requested = true;
                    args.provsetoutsource = true;
                }
                "--setprovideroffloadsink" => {
                    let provider_name = get_next_arg!();
                    args.provider_name = Some(Name::new(
                        provider_name,
                        NameKind::STRING | NameKind::XID | NameKind::INDEX,
                    ));

                    let offload_sink_provider_name = match raw_args.next() {
                        Some(output_source_provider_name) => Name::new(
                            output_source_provider_name,
                            NameKind::STRING | NameKind::XID | NameKind::INDEX,
                        ),
                        None => Name::new_xid(OrNone::none()),
                    };
                    args.offload_sink_provider_name = Some(offload_sink_provider_name);

                    args.action_requested = true;
                    args.provsetoffsink = true;
                }
                "--listmonitors" => {
                    args.list_monitors = true;
                    args.action_requested = true;
                }
                "--listactivemonitors" => {
                    args.list_active_monitors = true;
                    args.action_requested = true;
                }
                "--setmonitor" => {
                    let mut monitor = Monitor::new();

                    let mut name = get_next_arg!();
                    if name.starts_with('*') {
                        monitor.primary = true;
                        name = name[1..].to_string();
                    }
                    monitor.name = name;
                    monitor.set = true;

                    let geom = get_next_arg!();
                    if geom.as_str() != "auto" {
                        (|| {
                            let (width, geom) = geom.split_once('/')?;
                            monitor.width = u32::from_str(width).ok()?;

                            let (mmwidth, geom) = geom.split_once('x')?;
                            monitor.mmheight = u32::from_str(mmwidth).ok()?;

                            let (height, geom) = geom.split_once('/')?;
                            monitor.height = u32::from_str(height).ok()?;

                            let (mmheight, geom) = geom.split_once('+')?;
                            monitor.mmheight = u32::from_str(mmheight).ok()?;

                            let (x, y) = geom.split_once('+')?;
                            monitor.x = i32::from_str(x).ok()?;
                            monitor.y = i32::from_str(y).ok()?;

                            Some(())
                        })()
                        .ok_or(InvalidCliArgs::CouldNotParse(arg.clone(), geom))?;
                    }

                    let outputs = get_next_arg!();
                    if outputs.as_str() != "none" {
                        for output_raw_name in outputs.split(',') {
                            let output_name = Name::new(
                                output_raw_name.to_string(),
                                NameKind::STRING | NameKind::XID | NameKind::INDEX,
                            );
                            let mut output = Output::new();
                            output.output = output_name;
                            monitor.outputs.push(output);
                        }
                    }

                    args.monitors.push(monitor);
                    args.monitorit = true;
                    args.action_requested = true;
                }
                "--delmonitor" => {
                    let mut monitor = Monitor::new();
                    monitor.name = get_next_arg!();
                    monitor.set = false;

                    args.monitors.push(monitor);
                    args.monitorit = true;
                    args.action_requested = true;
                }
                _ => return Err(InvalidCliArgs::InvalidFlag(arg)),
            }
        }

        if !args.action_requested {
            args.query = true;
        }

        if args.verbose {
            args.query = true;
            if args.setit && !args.setit_1_2 {
                args.query_1 = true;
            }
        }

        Ok(args)
    }
}
