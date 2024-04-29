use crate::{
    extensions::mit_shm::ShmSegId, requests::write_le_bytes, Drawable, GContextId, PixmapId,
    ToLeBytes,
};

pub mod opcodes;

macro_rules! impl_xrequest_with_response {
    ($r:tt) => {
        impl $crate::requests::XRequestBase for $r {
            type Reply = super::replies::$r;

            #[inline(always)]
            fn reply_type() -> Option<crate::replies::ReplyType> {
                Some(crate::replies::ReplyType::ExtensionMitShm(
                    super::replies::ReplyType::$r,
                ))
            }
        }

        impl $crate::requests::XExtensionRequest for $r {}
    };
}

macro_rules! impl_xrequest_without_response {
    ($r:tt) => {
        impl $crate::requests::XRequestBase for $r {
            type Reply = $crate::requests::NoReply;

            #[inline(always)]
            fn reply_type() -> Option<$crate::replies::ReplyType> {
                None
            }
        }

        impl $crate::requests::XExtensionRequest for $r {}
    };
}

#[derive(Debug, Clone)]
pub struct QueryVersion;

impl ToLeBytes for QueryVersion {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::QUERY_VERSION);
        write_le_bytes!(w, 1u16); // request length

        Ok(())
    }
}

impl_xrequest_with_response!(QueryVersion);

#[derive(Debug, Clone)]
pub struct Attach {
    pub shmseg: ShmSegId,
    pub shmid: u32,
    pub read_only: bool,
}

impl ToLeBytes for Attach {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::ATTACH);
        write_le_bytes!(w, 4u16); // request length
        write_le_bytes!(w, self.shmseg);
        write_le_bytes!(w, self.shmid);
        write_le_bytes!(w, self.read_only as u8);
        write_le_bytes!(w, 0u8); // pad
        write_le_bytes!(w, 0u16); // pad

        Ok(())
    }
}

impl_xrequest_without_response!(Attach);

#[derive(Debug, Clone)]
pub struct Detach {
    pub shmseg: ShmSegId,
}

impl ToLeBytes for Detach {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::DETACH);
        write_le_bytes!(w, 2u16); // request length
        write_le_bytes!(w, self.shmseg);

        Ok(())
    }
}

impl_xrequest_without_response!(Detach);

#[derive(Debug, Clone)]
pub struct PutImage {
    pub drawable: Drawable,
    pub gc: GContextId,
    pub total_width: u16,
    pub total_height: u16,
    pub src_x: u16,
    pub src_y: u16,
    pub src_width: u16,
    pub src_height: u16,
    pub dst_x: i16,
    pub dst_y: i16,
    pub depth: u8,
    pub format: u8,
    pub send_event: u8,
    pub bpad: u8,
    pub shmseg: ShmSegId,
    pub offset: u32,
}

impl ToLeBytes for PutImage {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::PUT_IMAGE);
        write_le_bytes!(w, 10u16); // request length
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.gc);
        write_le_bytes!(w, self.total_width);
        write_le_bytes!(w, self.total_height);
        write_le_bytes!(w, self.src_x);
        write_le_bytes!(w, self.src_y);
        write_le_bytes!(w, self.src_width);
        write_le_bytes!(w, self.src_height);
        write_le_bytes!(w, self.dst_x);
        write_le_bytes!(w, self.dst_y);
        write_le_bytes!(w, self.depth);
        write_le_bytes!(w, self.format);
        write_le_bytes!(w, self.send_event);
        write_le_bytes!(w, self.bpad);
        write_le_bytes!(w, self.shmseg);
        write_le_bytes!(w, self.offset);

        Ok(())
    }
}

impl_xrequest_without_response!(PutImage);

#[derive(Debug, Clone)]
pub struct GetImage {
    pub drawable: Drawable,
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
    pub plane_mask: u32,
    pub format: u8,
    pub shmseg: ShmSegId,
    pub offset: u32,
}

impl ToLeBytes for GetImage {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::GET_IMAGE);
        write_le_bytes!(w, 8u16); // request length
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.x);
        write_le_bytes!(w, self.y);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);
        write_le_bytes!(w, self.plane_mask);
        write_le_bytes!(w, self.format);
        write_le_bytes!(w, 0u8); // pad
        write_le_bytes!(w, 0u8); // pad
        write_le_bytes!(w, 0u8); // pad
        write_le_bytes!(w, self.shmseg);
        write_le_bytes!(w, self.offset);

        Ok(())
    }
}

impl_xrequest_with_response!(GetImage);

#[derive(Debug, Clone)]
pub struct CreatePixmap {
    pub pid: PixmapId,
    pub drawable: Drawable,
    pub width: u16,
    pub height: u16,
    pub depth: u8,
    pub shmseg: ShmSegId,
    pub offset: u32,
}

impl ToLeBytes for CreatePixmap {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::CREATE_PIXMAP);
        write_le_bytes!(w, 7u16); // request length
        write_le_bytes!(w, self.pid);
        write_le_bytes!(w, self.drawable);
        write_le_bytes!(w, self.width);
        write_le_bytes!(w, self.height);
        write_le_bytes!(w, self.depth);
        write_le_bytes!(w, self.shmseg);
        write_le_bytes!(w, self.offset);

        Ok(())
    }
}

impl_xrequest_without_response!(CreatePixmap);

#[derive(Debug, Clone)]
pub struct AttachFd {
    pub shmseg: ShmSegId,
    pub read_only: bool,
}

impl ToLeBytes for AttachFd {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::ATTACH_FD);
        write_le_bytes!(w, 3u16); // request length
        write_le_bytes!(w, self.shmseg);
        write_le_bytes!(w, self.read_only as u8);
        write_le_bytes!(w, 0u8); // pad
        write_le_bytes!(w, 16u8); // pad

        Ok(())
    }
}

impl_xrequest_without_response!(AttachFd);

#[derive(Debug, Clone)]
pub struct CreateSegment {
    pub shmseg: ShmSegId,
    pub size: u32,
    pub read_only: bool,
}

impl ToLeBytes for CreateSegment {
    fn to_le_bytes(&self, w: &mut impl std::io::Write) -> std::io::Result<()> {
        write_le_bytes!(w, opcodes::CREATE_SEGMENT);
        write_le_bytes!(w, 4u16); // request length
        write_le_bytes!(w, self.shmseg);
        write_le_bytes!(w, self.size);
        write_le_bytes!(w, self.read_only as u8);
        write_le_bytes!(w, 0u8); // pad
        write_le_bytes!(w, 16u8); // pad

        Ok(())
    }
}

impl_xrequest_with_response!(CreateSegment);
