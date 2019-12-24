#![crate_type = "dylib"]

#[no_mangle]
extern "C" fn transform_x(x: i32) -> f64 {
	x.count_ones().into()
}
