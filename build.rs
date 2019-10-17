use std::borrow::Cow;
use std::env::set_current_dir;
use std::env::current_dir;
use std::env::var;
use std::process::Command;

const IN_FILE: &str = "src/black_box.c";

fn main() -> Result<(), Cow<'static, str>> {
	let var = var("OUT_DIR").map_err(or)?;
	let current_dir = current_dir().map_err(or)?;
	set_current_dir(&var).map_err(or)?;

	if ! Command::new("c99")
		.arg("-c")
		.arg("-O2")
		.arg(format!("{}/{}", current_dir.display(), IN_FILE))
		.status()
		.map_err(or)?
		.success() {
		Err("c99")?;
	}
	if ! Command::new("ar")
		.arg("rs")
		.arg("libblack_box.a")
		.arg("black_box.o")
		.status()
		.map_err(or)?
		.success() {
		Err("ar")?;
	}

	println!("cargo:rerun-if-changed={}", IN_FILE);
	println!("cargo:rustc-link-search={}", var);
	Ok(())
}

fn or<T: ToString>(err: T) -> String {
	err.to_string()
}
