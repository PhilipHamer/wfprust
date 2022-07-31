use winapi::shared::ntdef::*;
use winapi::km::wdm::{KPROCESSOR_MODE, STANDARD_RIGHTS_ALL, KeWaitForSingleObject};
use winapi::shared::ntstatus::STATUS_SUCCESS;

#[allow(non_camel_case_types)]
pub type PKSTART_ROUTINE = extern "system" fn(PVOID) -> ();

// we treat these as opaque
#[allow(non_camel_case_types)]
type POBJECT_TYPE = PVOID;
#[allow(non_camel_case_types)]
type PKTHREAD = PVOID;
#[allow(non_camel_case_types)]
type POBJECT_HANDLE_INFORMATION = PVOID;

extern "system" {

    static PsThreadType: *const POBJECT_TYPE;

    fn ZwClose(Handle: HANDLE) -> NTSTATUS;

    fn PsCreateSystemThread(
        ThreadHandle: PHANDLE,
        DesiredAccess: ULONG,
        ObjectAttributes: POBJECT_ATTRIBUTES,
        ProcessHandle: HANDLE,
        ClientId: PVOID,
        StartRoutine: PKSTART_ROUTINE,
        StartContext: PVOID,
    ) -> NTSTATUS;

    pub fn PsTerminateSystemThread(ExitStatus: NTSTATUS) -> NTSTATUS;

    fn ObReferenceObjectByHandle(
        Handle: HANDLE,
        DesiredAccess: ULONG,
        ObjectType: POBJECT_TYPE,
        AccessMode: KPROCESSOR_MODE,
        Object: *mut PVOID,
        HandleInformation: POBJECT_HANDLE_INFORMATION,
    ) -> NTSTATUS;

    fn ObDereferenceObject(a: PVOID) -> ();

    fn KeSetBasePriorityThread(Thread: PKTHREAD, Increment: LONG) -> LONG;
}

pub struct KThread {
    kthread: PKTHREAD,
}

impl KThread {

    pub fn new_low_priority(start_routine: PKSTART_ROUTINE, data: PVOID) -> Option<Self> {
        Self::common_new(start_routine, data, true)
    }
    pub fn new(start_routine: PKSTART_ROUTINE, data: PVOID) -> Option<Self> {
        Self::common_new(start_routine, data, false)
    }
    fn common_new(start_routine: PKSTART_ROUTINE, data: PVOID, low_priority: bool) -> Option<Self> {
        let mut kthread = KThread {
            kthread: 0 as _,
        };

        // init object attributes
        let mut obj_attr = OBJECT_ATTRIBUTES {
            Length: 0,
            RootDirectory: 0 as _,
            ObjectName: 0 as _,
            Attributes: 0,
            SecurityDescriptor: 0 as _,
            SecurityQualityOfService: 0 as _,
        };
        unsafe { InitializeObjectAttributes(&mut obj_attr, 0 as _, OBJ_KERNEL_HANDLE, 0 as _, 0 as _) ; }

        // create thread
        let mut handle: HANDLE = 0 as _;
        let status_create = unsafe { PsCreateSystemThread(
            &mut handle,
            0,
            &mut obj_attr,
            0 as _,
            0 as _,
            start_routine,
            data,
        ) };
        if status_create != STATUS_SUCCESS {
            return None;
        }

        // obref it
        let status_obref = unsafe { ObReferenceObjectByHandle(
            handle, 
            STANDARD_RIGHTS_ALL, 
            *PsThreadType, 
            KPROCESSOR_MODE::KernelMode, 
            &mut kthread.kthread, 
            0 as _,
        ) };

        // we can close handle (kthread is dropped later)
        let status_close = unsafe { ZwClose(handle) };
        if status_close != STATUS_SUCCESS {
            return None;
        }
        if status_obref != STATUS_SUCCESS {
            return None;
        }

        if low_priority {
            unsafe { KeSetBasePriorityThread(kthread.kthread, -1); }
        }

        Some(kthread)
    }

}

impl Drop for KThread {
    fn drop(&mut self) {
        if !self.kthread.is_null() {
            // TODO check return value? set a timeout?
            unsafe { KeWaitForSingleObject(self.kthread, 0, KPROCESSOR_MODE::KernelMode, false, None) };
            unsafe { ObDereferenceObject(self.kthread) };
        }
    }
}
