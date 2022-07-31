#[macro_export]
macro_rules! log {
    ($string: expr) => {
        kernel_print::kernel_println!(concat!("wfprust: ", $string));
    };
    ($string: expr, $($arg:tt)*) => {
        kernel_print::kernel_println!(concat!("wfprust: ", $string), $($arg)*);
    };
}
