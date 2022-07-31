use winapi::shared::ntdef::*;

// containing type must be #[repr(C)]
// TODO make it more solid by using language features (traits? generics? more macros?)

pub struct List {
    head: PLIST_ENTRY,
}

#[macro_export]
macro_rules! containing_record {
    ($head: ident, $parent: path, $field: tt) => {
        // TODO offset_of! is UB so find a replacement such as 
        // https://users.rust-lang.org/t/how-can-i-get-the-offset-of-a-field-in-a-repr-c-struct-as-a-constant/40166/5
        unsafe { &mut (*(($head as *const u8).offset(-(offset_of!($parent, $field) as isize)) as *mut $parent)) }
        // see CONTAINING_RECORD macro in ntdef.h
    };
}

impl List {

    pub fn init(head: PLIST_ENTRY) -> Self {
        let list = List { head: head };
        unsafe { Self::InitializeListHead(list.head); }
        list
    }

    pub fn remove_head(&mut self) -> PLIST_ENTRY {
        unsafe { Self::RemoveHeadList(self.head) }
    }

    pub fn remove_tail(&mut self) -> PLIST_ENTRY {
        unsafe { Self::RemoveTailList(self.head) }
    }

    pub fn insert_head(&mut self, entry: PLIST_ENTRY) {
        unsafe { Self::InsertHeadList(self.head, entry) }
    }

    pub fn insert_tail(&mut self, entry: PLIST_ENTRY) {
        unsafe { Self::InsertTailList(self.head, entry) }
    }

    pub fn is_empty(&mut self) -> bool {
        unsafe { Self::IsListEmpty(self.head) }
    }

    #[allow(non_snake_case)]
    unsafe fn IsListEmpty(head: PLIST_ENTRY) -> bool {
        (*head).Flink == head
    }

    #[allow(non_snake_case)]
    unsafe fn InitializeListHead(head: PLIST_ENTRY) {
        (*head).Blink = head;
        (*head).Flink = head;
    }

    #[allow(non_snake_case)]
    unsafe fn RemoveHeadList(head: PLIST_ENTRY) -> PLIST_ENTRY {
        let entry = (*head).Flink;
        let next = (*entry).Flink;
        assert!((*entry).Blink == head && (*next).Blink == entry);
        (*head).Flink = next;
        (*next).Blink = head;
        entry
    }

    #[allow(non_snake_case)]
    unsafe fn RemoveTailList(head: PLIST_ENTRY) -> PLIST_ENTRY {
        let entry = (*head).Blink;
        let prev_entry = (*entry).Blink;
        assert!((*entry).Flink == head && (*prev_entry).Flink == entry);
        (*head).Blink = prev_entry;
        (*prev_entry).Flink = head;
        entry
    }

    #[allow(non_snake_case)]
    unsafe fn InsertHeadList(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
        let next = (*head).Flink;
        assert!((*next).Blink == head);
        (*entry).Flink = next;
        (*entry).Blink = head;
        (*next).Blink = entry;
        (*head).Flink = entry;
    }

    #[allow(non_snake_case)]
    unsafe fn InsertTailList(head: PLIST_ENTRY, entry: PLIST_ENTRY) {
        let prev_entry = (*head).Blink;
        assert!((*prev_entry).Flink == head);
        (*entry).Flink = head;
        (*entry).Blink = prev_entry;
        (*prev_entry).Flink = entry;
        (*head).Blink = entry;
    }
}
