use winapi::km::ndis::*;
use kernel_alloc::nt::{ExAllocatePool, ExFreePool, PoolType};

pub struct Packet {
    allocated: *mut u8,
    size: usize,
}

impl Packet {

    pub fn new(netbuf: PNET_BUFFER) -> Option<Self> {
        let packet_size = unsafe { *(*netbuf).u1.s().u.DataLength() } as usize;
        if packet_size == 0 {
            return None;
        }
        let mem_packet = unsafe { ExAllocatePool(PoolType::NonPagedPool, packet_size) } as *mut u8;
        if mem_packet.is_null() {
            return None;
        }
        let pkt = Packet{
            allocated: mem_packet,
            size: packet_size,
        };
        let buf_ndis = unsafe { NdisGetDataBuffer(netbuf, packet_size as u32, 0 as _, 1, 0) } as *const u8;
        if buf_ndis.is_null() {
            // non-contiguous
            let buf_ndis = unsafe { NdisGetDataBuffer(netbuf, packet_size as u32, mem_packet as _, 1, 0) } as *const u8;
            if buf_ndis.is_null() {
                // memory is freed
                return None;
            }
        } else {
            // contiguous
            // TODO? unsafe { core::ptr::copy_nonoverlapping(buf_ndis, mem_packet, packet_size); }
            let mut i: usize = 0;
            while i < packet_size {
                unsafe { *(mem_packet).offset(i as isize) = *(buf_ndis).offset(i as isize) };
                i = i + 1;
            }
        }
        Some( pkt )
    }

    #[inline]
    pub fn get_size(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn byte_at(&self, i: isize) -> u8 {
        unsafe { *(self.allocated.offset(i)) }
    }

}

impl Drop for Packet {
    fn drop(&mut self) {
        unsafe { ExFreePool(self.allocated as _) };
    }
}
