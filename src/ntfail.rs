#[macro_export]
macro_rules! check_status {
    ($status: ident, $funcname: expr) => {
        if $status != STATUS_SUCCESS {
            $crate::log!("{} failed (status={:#x})", $funcname, $status);
            return $status;
        }
    };
}
