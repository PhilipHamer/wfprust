#![no_std]
#![feature(alloc_error_handler)]

extern crate alloc;

#[macro_use]
extern crate memoffset;

use crate::callouts::ipv4out::*;
use crate::callout::install_callout;
use crate::callouts::transportv4out::*;
use core::panic::PanicInfo;
use winapi::km::wdm::*;
use winapi::km::fwp::fwpmk::*;
use winapi::shared::ntdef::{PUNICODE_STRING, HANDLE, NTSTATUS};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::shared::rpcdce::RPC_C_AUTHN_DEFAULT;

pub mod log;
pub mod string;
pub mod ntfail;
pub mod callout;
pub mod callouts;
pub mod packet;
pub mod spinlock;
pub mod kthread;
pub mod llist;
mod test;

/// When using the alloc crate it seems like it does some unwinding. Adding this
/// export satisfies the compiler but may introduce undefined behaviour when a
/// panic occurs.
#[no_mangle]
pub extern "system" fn __CxxFrameHandler3(_: *mut u8, _: *mut u8, _: *mut u8, _: *mut u8) -> i32 { unimplemented!() }

#[global_allocator]
static GLOBAL: kernel_alloc::KernelAlloc = kernel_alloc::KernelAlloc;

/// Explanation can be found here: https://github.com/Trantect/win_driver_example/issues/4
#[export_name = "_fltused"]
static _FLTUSED: i32 = 0;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! { loop {} }

// DriverEntry
#[no_mangle]
pub extern "system" fn driver_entry(driver_obj: PDRIVER_OBJECT, _reg_path: PUNICODE_STRING) -> NTSTATUS {

    log!("loading driver");

    let mut status: NTSTATUS;

    // TEMP test of spin lock
    //status = test::test_spin_lock();
    //check_status!(status, "test_spin_lock");
    // TEMP test of linked list
    //test::test_linked_list();

    // Create device object
    const FILE_DEVICE_SECURE_OPEN: u32 = 0x00000100;
    let mut device_obj: PDEVICE_OBJECT = 0 as _;
    unsafe { status = IoCreateDevice(driver_obj, 0, 0 as _, DEVICE_TYPE::FILE_DEVICE_NETWORK, FILE_DEVICE_SECURE_OPEN, 0, &mut device_obj) };
    check_status!(status, "IoCreateDevice");

    // WFP engine open
    let mut engine_handle: HANDLE = 0 as _;
    unsafe { status = FwpmEngineOpen0(0 as _, RPC_C_AUTHN_DEFAULT, 0 as _, 0 as _, &mut engine_handle) };
    check_status!(status, "FwpmEngineOpen0");

    install_callout(
        device_obj,
        engine_handle,
        wide_string!("outbound_ipv4"), 
        wide_string!("outbound ipv4 thingy"),
        CALLOUT_OUTBOUND_IPV4_GUID,
        SUBLAYER_OUTBOUND_IPV4_GUID,
        FILTER_OUTBOUND_IPV4_GUID,
        FWPM_LAYER_OUTBOUND_IPPACKET_V4, 
        outbound_ippacket_classify_v4);

    install_callout(
        device_obj,
        engine_handle,
        wide_string!("outbound_transport v4"), 
        wide_string!("outbound transport v4 thingy"),
        CALLOUT_OUTBOUND_TRANSPORTV4_GUID,
        SUBLAYER_OUTBOUND_TRANSPORTV4_GUID,
        FILTER_OUTBOUND_TRANSPORTV4_GUID,
        FWPM_LAYER_OUTBOUND_TRANSPORT_V4, 
        outbound_transport_classify_v4);

    // WFP engine close
    //unsafe { status = FwpmEngineClose0(engine_handle) };
    //check_status!(status, "FwpmEngineClose0");

    log!("driver loaded");

    STATUS_SUCCESS
}
