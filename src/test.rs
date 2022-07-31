#![allow(dead_code)]

use winapi::shared::ntdef::*;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use crate::spinlock::*;
use crate::kthread::*;
use crate::llist::*;
use crate::containing_record;
use crate::log;

const LIMIT: u32 = u32::MAX;
const NUM_THREADS: u32 = 15;
const MAX_PER_THREAD: u32 = LIMIT / NUM_THREADS;

struct ThreadData {
    ptr: *mut u32,
    lck: PKSPIN_LOCK,
}

extern "system" fn worker_func(data: PVOID) -> () {

    log!("in worker_func");

    if !data.is_null() {

        let thread_data = unsafe{&mut *(data as *mut ThreadData)};
        let ptr = thread_data.ptr;
        let lck = thread_data.lck;

        let mut i: u32 = 0;
        while i < MAX_PER_THREAD {
            let mut spin_lock = SpinLock::from_existing(lck);
            spin_lock.acquire();
            unsafe { *ptr = *ptr + 1; }
            i = i + 1;
        }
    }

    unsafe {
        PsTerminateSystemThread(STATUS_SUCCESS);
    }
}

extern "system" fn master_func(_: PVOID) -> () {
    let mut the_counter: u32 = 0;
    let spin_lock = SpinLock::new();
    let mut thread_data = ThreadData {
        ptr: &mut the_counter as *mut u32,
        lck: spin_lock.spin_lock_ptr,
    };
    
    {
        // create worker threads
        let threads: [Option<KThread>; NUM_THREADS as usize] = array_init::array_init(
            |_| KThread::new_low_priority(worker_func, &mut thread_data as *mut ThreadData as PVOID));

        for t in threads.iter() {
            match t {
                None => { log!("failed to spawn worker thread"); },
                _ => { },
            }
        }
        // TODO...
    }

    log!("done with threads, the counter is {}", the_counter);

    unsafe {
        PsTerminateSystemThread(STATUS_SUCCESS);
    }
}

static mut HELPER_THREAD: Option<KThread> = None;

pub fn test_spin_lock() -> NTSTATUS {

    log!("limit is {}, num threads is {}, max per thread is {}, total should be {}", 
        LIMIT, NUM_THREADS, MAX_PER_THREAD, NUM_THREADS * MAX_PER_THREAD);

    unsafe { 
        HELPER_THREAD = KThread::new_low_priority(master_func, 0 as _);
        match HELPER_THREAD {
            None => -1,
            _ => STATUS_SUCCESS,
        }
    }
}

// TEST linked list

#[repr(C)]
struct Container {
    pub item1: u8,
    pub item2: u16,
    pub item3: i32,
    pub entry: LIST_ENTRY,
    pub item4: u64,
}

fn init_container(i1: u8, i2: u16, i3: i32, i4: u64) -> Container {
    Container {
        item1: i1,
        item2: i2,
        item3: i3,
        entry: LIST_ENTRY { Flink: 0 as _, Blink: 0 as _ },
        item4: i4,
    }
}

fn check_container(container: &Container, i1: u8, i2: u16, i3: i32, i4: u64) -> bool {
    container.item1 == i1 && container.item2 == i2 && container.item3 == i3 && container.item4 == i4
}

pub fn test_linked_list() {

    let mut head: LIST_ENTRY = unsafe { core::mem::zeroed() };
    let mut list = List::init(&mut head);

    if !list.is_empty() {
        log!("list should be empty at the start");
    }

    let mut c1 = init_container(1, 2, 3, 4);
    list.insert_head(&mut c1.entry);

    let mut c2 = init_container(5, 6, 7, 8);
    list.insert_head(&mut c2.entry);

    let _t = list.remove_tail(); // 1234

    if list.is_empty() {
        log!("list should not be empty here");
    }

    let mut c3 = init_container(9, 10, 11, 12);
    list.insert_tail(&mut c3.entry);

    let h = list.remove_head(); // 5678
    let t = list.remove_tail(); // 9101112

    if !list.is_empty() {
        log!("list should be empty at the end");
    }

    let ch = containing_record!(h, Container, entry);
    let ct = containing_record!(t, Container, entry);

    if !check_container(ch, 5, 6, 7, 8) {
        log!("container ch is bad");
    }

    if !check_container(ct, 9, 10, 11, 12) {
        log!("container ct is bad");
    }

    log!("done checking list");
}
