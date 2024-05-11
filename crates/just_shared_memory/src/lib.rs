#![no_std]
use core::slice;

#[link(name = "shmutils")]
extern "C" {
    pub(crate) fn shmutils_create(size: u32) -> i32;
    pub(crate) fn shmutils_get_ptr(shmid: i32) -> *mut u8;
    pub(crate) fn shmutils_free_remove(shmid: i32, shmaddr: *mut u8);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedMemoryId(i32);

impl SharedMemoryId {
    #[inline(always)]
    pub fn inner(self) -> i32 {
        self.0
    }
}

/// `sys/shm.h` shared memroy
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SharedMemory {
    size: u32,
    id: SharedMemoryId,
    data: *mut u8,
}

impl SharedMemory {
    /// Create new zeroed System V shared memory region
    #[inline(always)]
    pub fn zeroed(size: u32) -> Self {
        unsafe {
            let id = shmutils_create(size);
            let data = shmutils_get_ptr(id);
            data.write_bytes(0, size as usize);
            return Self {
                size,
                id: SharedMemoryId(id),
                data,
            };
        }
    }

    #[inline]
    pub fn id(&self) -> SharedMemoryId {
        self.id
    }

    #[inline]
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Get underlying data
    /// # SAFETY
    /// - Shared memory was not free before
    #[inline(always)]
    pub unsafe fn data(&self) -> &[u8] {
        slice::from_raw_parts_mut(self.data, self.size as usize)
    }

    /// Get underlying data
    /// # SAFETY
    /// - Shared memory was not free before
    #[inline(always)]
    pub unsafe fn data_mut(&mut self) -> &mut [u8] {
        slice::from_raw_parts_mut(self.data, self.size as usize)
    }

    /// Get pointer to the underlying data
    #[inline]
    pub fn data_raw(&self) -> *mut u8 {
        self.data
    }

    /// Free shared memory
    /// # SAFETY
    /// - Shared memory was not free before
    /// - No one else is reading shared memory
    #[inline(always)]
    pub unsafe fn free(self) {
        shmutils_free_remove(self.id.inner(), self.data);
    }
}

#[test]
#[allow(dropping_references)]
fn roundtrip_raw() {
    unsafe {
        let size = 64;

        let shm = shmutils_create(size);

        let ptr = shmutils_get_ptr(shm);
        ptr.write_bytes(0, size as usize);
        let buf = slice::from_raw_parts_mut(ptr, size as usize);
        assert_eq!(buf, &[0; 64]);
        let _ = drop(buf);

        shmutils_free_remove(shm, ptr);
    }
}

#[test]
fn wrapper() {
    unsafe {
        let mut shared = SharedMemory::zeroed(64);
        let data_mut = shared.data_mut();
        data_mut[0] = 1;
        data_mut[1] = 2;
        data_mut[2] = 3;

        let data = shared.data();
        assert_eq!(data.len(), 64);
        assert_eq!(&data[0..4], &[1, 2, 3, 0]);

        shared.free();
    }
}
