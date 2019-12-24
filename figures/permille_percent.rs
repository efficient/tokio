#![crate_type = "dylib"]

#[no_mangle]
extern "C" fn transform_x(x: i32) -> f64 {
	let x: f64 = x.into();
	x / 10.0
}
