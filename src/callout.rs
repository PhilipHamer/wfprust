use winapi::shared::ntdef::{PVOID, PCVOID, NTSTATUS, HANDLE};
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::shared::guiddef::GUID;
use winapi::km::wdm::PDEVICE_OBJECT;
use winapi::km::fwp::{fwpmk::*, fwpsk::*, fwptypes::*};

use crate::string::WideString;
use crate::check_status;

pub type ClassifyFn = extern "system" fn(
    *const FWPS_INCOMING_VALUES0,
    *const FWPS_INCOMING_METADATA_VALUES0,
    PVOID,
    PCVOID,
    *const FWPS_FILTER1,
    u64,
    *mut FWPS_CLASSIFY_OUT0,
) -> ();

extern "system" fn default_notify(
    _notify_type: FWPS_CALLOUT_NOTIFY_TYPE,
    _filter_key: *const GUID,
    _filter: *const FWPS_FILTER1,
) -> NTSTATUS {
    // nothing to do
    STATUS_SUCCESS
}

pub fn install_callout(
    device_obj: PDEVICE_OBJECT,
    engine_handle: HANDLE,
    name: &WideString, 
    description: &WideString,
    callout_key: GUID, 
    sublayer_key: GUID, 
    filter_key: GUID,
    layer: GUID, 
    classify_fn: ClassifyFn) -> NTSTATUS {

    let mut status: NTSTATUS;

    // register
    let callout_reg = FWPS_CALLOUT1 {
        calloutKey: callout_key,
        flags: 0,
        classifyFn: Some(classify_fn),
        notifyFn: Some(default_notify),
        flowDeleteFn: None,
    };
    unsafe { status = FwpsCalloutRegister1(device_obj as _, &callout_reg, 0 as _) };
    check_status!(status, "FwpsCalloutRegister1");

    // add
    let callout_add = FWPM_CALLOUT0 {
        calloutKey: callout_key,
        displayData: FWPM_DISPLAY_DATA0 {
            name: name.as_ptr(),
            description: description.as_ptr(),
        },
        flags: 0,
        providerKey: 0 as _,
        providerData: FWP_BYTE_BLOB { size: 0, data: 0 as _},
        applicableLayer: layer,
        calloutId: 0,
    };
    unsafe { status = FwpmCalloutAdd0(engine_handle, &callout_add, 0 as _, 0 as _) };
    check_status!(status, "FwpmCalloutAdd0");

    // sublayer
    let sublayer = FWPM_SUBLAYER0 {
        subLayerKey: sublayer_key,
        displayData: FWPM_DISPLAY_DATA0 {
            name: name.as_ptr(),
            description: description.as_ptr(),
        },
        flags: 0,
        providerKey: 0 as _,
        providerData: FWP_BYTE_BLOB { size: 0, data: 0 as _},
        weight: u16::MAX,
    };
    unsafe { status = FwpmSubLayerAdd0(engine_handle, &sublayer, 0 as _) };
    check_status!(status, "FwpmSubLayerAdd0");

    // filter
    let fwp_value_empty: FWP_VALUE0_u;
    let fwp_filter: FWPM_FILTER0_u;
    let mut fwp_action: FWPM_ACTION0_u;
    unsafe {
        fwp_value_empty = core::mem::zeroed();
        fwp_filter = core::mem::zeroed();
        fwp_action = core::mem::zeroed();
        *(fwp_action.calloutKey_mut()) = callout_key;
    }

    let filter = FWPM_FILTER0 {
        filterKey: filter_key,
        displayData: FWPM_DISPLAY_DATA0 {
            name: name.as_ptr(),
            description: description.as_ptr(),
        },
        flags: 0,
        providerKey: 0 as _,
        providerData: FWP_BYTE_BLOB { size: 0, data: 0 as _},
        layerKey: layer,
        subLayerKey: sublayer_key,
        weight: FWP_VALUE0 { r#type: FWP_EMPTY, u: fwp_value_empty },
        numFilterConditions: 0,
        filterCondition: 0 as _,
        action: FWPM_ACTION0 { r#type: FWP_ACTION_CALLOUT_UNKNOWN, u: fwp_action },
        u: fwp_filter,
        reserved: 0 as _,
        filterId: 0 as u64,
        effectiveWeight: FWP_VALUE0 { r#type: FWP_EMPTY, u: fwp_value_empty },
    };
    unsafe { status = FwpmFilterAdd0(engine_handle, &filter, 0 as _, 0 as _) };
    check_status!(status, "FwpmFilterAdd0");

    status
}
