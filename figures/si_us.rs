#![crate_type = "dylib"]

#[no_mangle]
extern "C" fn transform_y(y: &str) -> f64 {
	let oom = y.find(char::is_alphabetic).unwrap();
	let val: f64 = y[..oom].parse().unwrap();
	match &y[oom..] {
	"us" => val,
	"ms" => val * 1_000.0,
	"s" => val * 1_000_000.0,
	_ => panic!("unrecognized si suffix"),
	}
}
