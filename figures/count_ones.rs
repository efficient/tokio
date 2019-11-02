#![crate_type = "dylib"]

#[no_mangle]
extern "C" fn transform_x(x: usize) -> usize {
	x.count_ones() as _
}
