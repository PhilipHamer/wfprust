use winapi::shared::ntdef::UCHAR;
use winapi::shared::basetsd::ULONG_PTR;

#[allow(non_camel_case_types)]
pub type KSPIN_LOCK = ULONG_PTR;
#[allow(non_camel_case_types)]
pub type PKSPIN_LOCK = *mut KSPIN_LOCK;
#[allow(non_camel_case_types)]
type PKLOCK_QUEUE_HANDLE = *mut KLOCK_QUEUE_HANDLE;
type KIRQL = UCHAR;

#[repr(C)]
#[allow(non_snake_case)]
struct KSPIN_LOCK_QUEUE {
    pub Next: *mut KSPIN_LOCK_QUEUE,
    pub Lock: PKSPIN_LOCK,
}

#[repr(C)]
#[allow(non_snake_case)]
struct KLOCK_QUEUE_HANDLE {
    pub LockQueue: KSPIN_LOCK_QUEUE,
    pub OldIrql: KIRQL,
}

extern "system" {
    fn KeInitializeSpinLock(spin_lock: PKSPIN_LOCK);
    fn KeAcquireInStackQueuedSpinLock(spin_lock: PKSPIN_LOCK, lock_handle: PKLOCK_QUEUE_HANDLE);
    fn KeReleaseInStackQueuedSpinLock(lock_handle: PKLOCK_QUEUE_HANDLE);
}

pub struct SpinLock {
    _sl_dont_use: KSPIN_LOCK,
    pub spin_lock_ptr: PKSPIN_LOCK,
    lock_handle: KLOCK_QUEUE_HANDLE,
    acquired: bool,
}

impl SpinLock {
    pub fn new() -> Self {
        let mut spin_lock = SpinLock {
            _sl_dont_use: 0,
            spin_lock_ptr: 0 as _,
            lock_handle: unsafe { core::mem::zeroed() },
            acquired: false,
        };
        unsafe { KeInitializeSpinLock(&mut spin_lock._sl_dont_use) }
        spin_lock.spin_lock_ptr = &mut spin_lock._sl_dont_use;
        spin_lock
    }
    pub fn from_existing(existing: PKSPIN_LOCK) -> Self {
        SpinLock {
            _sl_dont_use: 0,
            spin_lock_ptr: existing,
            lock_handle: unsafe { core::mem::zeroed() },
            acquired: false,
        }
    }
    pub fn acquire(&mut self) {
        unsafe { KeAcquireInStackQueuedSpinLock(self.spin_lock_ptr, &mut self.lock_handle) };
        self.acquired = true;
    }
    pub fn release(&mut self) {
        if self.acquired {
            unsafe { KeReleaseInStackQueuedSpinLock(&mut self.lock_handle) };
            self.acquired = false;
        }
    }
}

impl Drop for SpinLock {
    fn drop(&mut self) {
        self.release();
    }
}
