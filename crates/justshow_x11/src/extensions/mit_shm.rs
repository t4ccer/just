use crate::utils::impl_resource_id;

pub mod replies;
pub mod requests;

pub const EXTENSION_NAME: [u8; 7] = *b"MIT-SHM";

impl_resource_id!(ShmSegId);
