use std::num::NonZeroU32;

use crate::{connection::XConnection, error::Error, ResourceId, Window};

#[derive(Debug, Clone)]
pub struct WindowAttributes {
    backing_store: u8,
    visual_id: u32,
    class: u16,
    bit_gravity: u8,
    win_gravity: u8,
    backing_planes: u32,
    backing_pixel: u32,
    save_under: bool,
    map_is_installed: bool,
    map_state: u8,
    override_redirect: bool,
    colormap: u32,
    all_even_masks: u32,
    your_even_masks: u32,
    do_not_propagate_mask: u16,
}

impl WindowAttributes {
    pub(crate) fn from_be_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let backing_store = conn.read_u8()?;
        let _sequence_code = conn.read_be_u16()?;
        let _reply_length = conn.read_be_u32()?;
        let visual_id = conn.read_be_u32()?;
        let class = conn.read_be_u16()?;
        let bit_gravity = conn.read_u8()?;
        let win_gravity = conn.read_u8()?;
        let backing_planes = conn.read_be_u32()?;
        let backing_pixel = conn.read_be_u32()?;
        let save_under = conn.read_bool()?;
        let map_is_installed = conn.read_bool()?;
        let map_state = conn.read_u8()?;
        let override_redirect = conn.read_bool()?;
        let colormap = conn.read_be_u32()?;
        let all_even_masks = conn.read_be_u32()?;
        let your_even_masks = conn.read_be_u32()?;
        let do_not_propagate_mask = conn.read_be_u16()?;
        let _unused = conn.read_be_u16()?;

        Ok(Self {
            backing_store,
            visual_id,
            class,
            bit_gravity,
            win_gravity,
            backing_planes,
            backing_pixel,
            save_under,
            map_is_installed,
            map_state,
            override_redirect,
            colormap,
            all_even_masks,
            your_even_masks,
            do_not_propagate_mask,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Geometry {
    pub depth: u8,
    pub root: Window,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub border_width: u16,
}

impl Geometry {
    pub(crate) fn from_be_bytes(conn: &mut XConnection) -> Result<Self, Error> {
        let depth = conn.read_u8()?;
        let _sequence_code = conn.read_be_u16()?;
        let _reply_length = conn.read_be_u32()?;
        let root = Window(ResourceId {
            value: NonZeroU32::new(conn.read_be_u32()?).unwrap(),
        });
        let x = conn.read_be_i16()?;
        let y = conn.read_be_i16()?;
        let width = conn.read_be_u16()?;
        let height = conn.read_be_u16()?;
        let border_width = conn.read_be_u16()?;
        conn.drain(10)?;

        Ok(Self {
            depth,
            root,
            x,
            y,
            width,
            height,
            border_width,
        })
    }
}

#[derive(Debug, Clone)]
pub enum Reply {
    GetWindowAttributes(WindowAttributes),
    GetGeometry(Geometry),
}

#[derive(Debug, Clone)]
pub enum AwaitingReply {
    NotReceived(ReplyType),
    Received(Reply),
}

#[derive(Debug, Clone, Copy)]
pub enum ReplyType {
    GetWindowAttributes,
    GetGeometry,
}
