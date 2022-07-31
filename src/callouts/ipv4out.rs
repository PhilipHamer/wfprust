use winapi::shared::guiddef::GUID;
use winapi::km::fwp::fwpsk::*;
use winapi::shared::ntdef::{PVOID, PCVOID};
use winapi::km::ndis::*;
use crate::callouts::common::*;
use crate::log;
use winapi::shared::ws2def::{IPPROTO_ICMP, IPPROTO_TCP, IPPROTO_UDP};

pub const CALLOUT_OUTBOUND_IPV4_GUID: GUID = GUID {
    Data1: 0x1406f15a,
    Data2: 0x23ba,
    Data3: 0x4c27,
    Data4: [0xa9, 0x41, 0x3f, 0x81, 0x56, 0x9b, 0xe0, 0xd7],
};

pub const FILTER_OUTBOUND_IPV4_GUID: GUID = GUID {
    Data1: 0x791276e3,
    Data2: 0x970f,
    Data3: 0x4164,
    Data4: [0x9d, 0x65, 0x17, 0x59, 0xf3, 0xac, 0xd9, 0xad],
};

pub const SUBLAYER_OUTBOUND_IPV4_GUID: GUID = GUID {
    Data1: 0xd8baf99b, 
    Data2: 0x470f, 
    Data3: 0x4b59, 
    Data4: [0x81, 0x8a, 0x13, 0x2d, 0x83, 0x95, 0xae, 0xb],
};

// The above guids we make ourself, but this one is from Microsoft.
pub const FWPM_LAYER_OUTBOUND_IPPACKET_V4: GUID = GUID {
    Data1: 0x1e5c9fae,
    Data2: 0x8a84,
    Data3: 0x4135,
    Data4: [0xa3, 0x31, 0x95, 0x0b, 0x54, 0x22, 0x9e, 0xcd],
};

const _FWPS_FIELD_OUTBOUND_IPPACKET_V4_IP_LOCAL_ADDRESS: u32 = 0;
const _FWPS_FIELD_OUTBOUND_IPPACKET_V4_IP_REMOTE_ADDRESS: u32 = 2;

pub extern "system" fn outbound_ippacket_classify_v4(
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

    /*
    let local_addr: u32;
    let remote_addr: u32;
    unsafe {
        local_addr = *(*(*in_fixed_values).incomingValue.offset(FWPS_FIELD_OUTBOUND_IPPACKET_V4_IP_LOCAL_ADDRESS as isize)).value.u.uint32();
        remote_addr = *(*(*in_fixed_values).incomingValue.offset(FWPS_FIELD_OUTBOUND_IPPACKET_V4_IP_REMOTE_ADDRESS as isize)).value.u.uint32();
    }
    */

    if layer_data.is_null() {
        return;
    }

    let action = inspect_packets(layer_data as PNET_BUFFER_LIST, |pkt| {
        let mut verdict = PacketAction::Allow;
        if pkt.get_size() < 20 {
            return verdict;
        }
        let protocol = pkt.byte_at(9) as u32;
        match protocol {
            IPPROTO_TCP => { }
            IPPROTO_UDP => { }
            _ => { verdict = PacketAction::Log(protocol as usize); }
        }
        verdict
    });

    match action {
        PacketAction::Block if !classify_out.is_null() => { unsafe { (*classify_out).actionType = FWP_ACTION_BLOCK } }
        PacketAction::Log(data) => {
            match data as u32 {
                IPPROTO_ICMP => { log!("found icmp packet"); }
                _ => { log!("found ip packet with protocol {}", data); }
            }
        }
        _ => {}
    }
}
