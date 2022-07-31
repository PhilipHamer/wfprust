use winapi::km::ndis::*;
use crate::packet::Packet;
use crate::log;

pub type PacketInspectFn = fn(&Packet) -> PacketAction;

pub enum PacketAction {
    Allow,
    Block,
    Log(usize),
}

pub fn inspect_packets(buf_list: PNET_BUFFER_LIST, inspect_packet: PacketInspectFn) -> PacketAction {

    let mut action = PacketAction::Allow;
    let mut buf: PNET_BUFFER;
    let buf_list_next: PNET_BUFFER_LIST;
    unsafe {
        buf = (*buf_list).u1.s().FirstNetBuffer;
        buf_list_next = (*buf_list).u1.s().Next;
    }
    if buf.is_null() {
        return action;
    }
    if !buf_list_next.is_null() {
        return action;
    }
    
    while !buf.is_null() {
        if let Some(pkt) = Packet::new(buf) {
            match inspect_packet(&pkt) {
                PacketAction::Block => { action = PacketAction::Block }
                PacketAction::Log(data) if !matches!(action, PacketAction::Block) => { action = PacketAction::Log(data) }
                _ => { }
            }
        } else {
            log!("packet failed to alloc/parse");
        }
        buf = unsafe { (*buf).u1.s() }.Next;
    }

    action
}
