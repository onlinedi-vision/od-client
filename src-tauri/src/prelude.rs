#[cfg(debug_assertions)]
pub(crate) fn debug_only_print(msg: &dyn std::fmt::Debug) {
    println!("{:?}", msg);
}

#[cfg(not(debug_assertions))]
pub(crate) fn debug_only_print(_msg: &dyn std::fmt::Debug) {}
