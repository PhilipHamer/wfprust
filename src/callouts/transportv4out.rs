use winapi::shared::guiddef::GUID;
use winapi::km::fwp::fwpsk::*;
use winapi::shared::ntdef::{PVOID, PCVOID};
use winapi::km::ndis::*;
use crate::callouts::common::*;
use winapi::shared::ws2def::IPPROTO_TCP;

pub const CALLOUT_OUTBOUND_TRANSPORTV4_GUID: GUID = GUID {
    Data1: 0x4b073c6c,
    Data2: 0x3113,
    Data3: 0x474c,
    Data4: [0xba, 0xa2, 0xdc, 0xef, 0xf1, 0xb8, 0xec, 0x18],
};

pub const FILTER_OUTBOUND_TRANSPORTV4_GUID: GUID = GUID {
    Data1: 0x92dcfb30,
    Data2: 0x431b,
    Data3: 0x4bea,
    Data4: [0xb7, 0x7d, 0x4e, 0x2f, 0x8b, 0x36, 0xfa, 0xcc],
};

pub const SUBLAYER_OUTBOUND_TRANSPORTV4_GUID: GUID = GUID {
    Data1: 0xe89125fd, 
    Data2: 0x66a3, 
    Data3: 0x4dd0, 
    Data4: [0xae, 0x9d, 0x45, 0xeb, 0xe, 0x1d, 0x64, 0x4a],
};

// The above guids we make ourself, but this one is from Microsoft.
pub const FWPM_LAYER_OUTBOUND_TRANSPORT_V4: GUID = GUID {
    Data1: 0x09e61aea,
    Data2: 0xd214,
    Data3: 0x46e2,
    Data4: [0x9b, 0x21, 0xb2, 0x6b, 0x0b, 0x2f, 0x28, 0xc8],
};

const FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_PROTOCOL: u32 = 0;
const _FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_LOCAL_ADDRESS: u32 = 1;
const _FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_LOCAL_ADDRESS_TYPE: u32 = 2;
const _FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_REMOTE_ADDRESS: u32 = 3;
const _FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_LOCAL_PORT: u32 = 4;
const _FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_REMOTE_PORT: u32 = 5;

pub extern "system" fn outbound_transport_classify_v4(
    in_fixed_values: *const FWPS_INCOMING_VALUES0,
    _in_meta_values:*const FWPS_INCOMING_METADATA_VALUES0,
    layer_data: PVOID,
    _classify_context: PCVOID,
    _filter: *const FWPS_FILTER1,
    _flow_context: u64,
    classify_out:*mut FWPS_CLASSIFY_OUT0,
) -> () {

    if in_fixed_values.is_null() {
        return;
    }
    if unsafe {(*in_fixed_values).incomingValue.is_null() } {
        return;
    }

    let protocol: u8;
    /*let local_port: u16;
    let remote_port: u16;
    let local_addr: u32;
    let remote_addr: u32;*/
    unsafe {
        protocol = *(*(*in_fixed_values).incomingValue.offset(FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_PROTOCOL as isize)).value.u.uint8();
        /*local_addr = *(*(*in_fixed_values).incomingValue.offset(FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_LOCAL_ADDRESS as isize)).value.u.uint32();
        remote_addr = *(*(*in_fixed_values).incomingValue.offset(FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_REMOTE_ADDRESS as isize)).value.u.uint32();
        local_port = *(*(*in_fixed_values).incomingValue.offset(FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_LOCAL_PORT as isize)).value.u.uint16();
        remote_port = *(*(*in_fixed_values).incomingValue.offset(FWPS_FIELD_OUTBOUND_TRANSPORT_V4_IP_REMOTE_PORT as isize)).value.u.uint16();*/
    }

    if protocol != IPPROTO_TCP as u8 {
        return;
    }

    if layer_data.is_null() {
        return;
    }

    let action = inspect_packets(layer_data as PNET_BUFFER_LIST, |pkt| {
        let mut verdict = PacketAction::Allow;
        if pkt.get_size() < 35 {
            return verdict;
        }
        if pkt.byte_at(12) & 0b11110000 == 0b01010000 && 
            pkt.byte_at(20) == b'G' && 
            pkt.byte_at(21) == b'E' && 
            pkt.byte_at(22) == b'T' &&
            pkt.byte_at(23) == b' ' &&
            pkt.byte_at(24) == b'/' &&
            pkt.byte_at(25) == b'm' &&
            pkt.byte_at(26) == b'a' &&
            pkt.byte_at(27) == b'p' &&
            pkt.byte_at(28) == b's' &&
            pkt.byte_at(29) == b' ' &&
            pkt.byte_at(30) == b'H' &&
            pkt.byte_at(31) == b'T' &&
            pkt.byte_at(32) == b'T' &&
            pkt.byte_at(33) == b'P' {
                verdict = PacketAction::Block;
        }
        verdict
    });

    match action {
        PacketAction::Block if !classify_out.is_null() => { unsafe { (*classify_out).actionType = FWP_ACTION_BLOCK } }
        _ => {}
    }

}
