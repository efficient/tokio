#[allow(dead_code, nonstandard_style)]
mod png;

use png::PNG_FORMAT_RGB;
use png::PNG_IMAGE_VERSION;
use png::png_image;
use png::png_image_begin_read_from_memory;
use png::png_image_finish_read;
use std::borrow::Cow;
use std::convert::TryInto;
use std::env::args;
use std::ffi::c_void;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::fs::File;
use std::io::Read;
use std::os::raw::c_uint;
use std::process::exit;
use std::ptr::null;
use std::ptr::null_mut;
use std::time::SystemTime;

fn main() -> Result<(), Cow<'static, str>> {
	let mut args = args();
	if args.len() != 2 {
		println!("USAGE: {} <filename>", args.next().ok_or("args.next()")?);
		exit(1);
	}

	let file = args.skip(1).next().ok_or("args.skip(1).next()")?;
	let mut file = File::open(file).ret("open()")?;
	let src = file.metadata().ret("fstat()")?.len().try_into().ret("u64::try_into()")?;
	let mut src = Vec::with_capacity(src);
	file.read_to_end(&mut src).ret("fread()")?;

	let mut img = png_image::default();
	img.version = PNG_IMAGE_VERSION;
	img.opaque = null_mut();

	let buf: *const c_void = src.as_ptr() as _;
	let then = SystemTime::now();
	if unsafe {
		png_image_begin_read_from_memory(&mut img, buf, src.len())
	} == 0 {
		Err(img).ret("png_image_begin_read_from_memory()")?;
	}
	println!("begin: {} us", us_since(&then)?);
	img.format = PNG_FORMAT_RGB;

	if img.warning_or_error != 0 {
		eprintln!("{}", img);
	}

	let dest = png_image_size(&img).try_into().ret("c_uint::try_into()")?;
	let mut dest = vec![0; dest].into_boxed_slice();
	let buf: *mut c_void = dest.as_mut_ptr() as _;
	let then = SystemTime::now();
	if unsafe {
		png_image_finish_read(&mut img, null(), buf, 0, null_mut())
	} == 0 {
		Err(img).ret("png_image_finish_read()")?;
	}
	println!("finish: {} us", us_since(&then)?);

	if img.warning_or_error != 0 {
		eprintln!("{}", img);
	}

	Ok(())
}

#[inline]
fn us_since(then: &SystemTime) -> Result<u128, String> {
	Ok(then.elapsed().ret("SystemTime::elapsed()")?.as_micros())
}

#[inline]
fn png_image_size(image: &png_image) -> c_uint {
	(if image.format&0x08 != 0 { 1 } else { ((image.format & 0x04) >> 2)+1 })*image.height*((if image.format&0x08 != 0 { 1 } else { (image.format&(0x02|0x01))+1 }) * image.width)
}

trait Ret {
	type Urn;

	fn ret(self, _: &str) -> Self::Urn;
}

impl<T, U: Display> Ret for Result<T, U> {
	type Urn = Result<T, String>;

	fn ret(self, messeng: &str) -> Self::Urn {
		self.map_err(|or| format!("{}: {}", messeng, or))
	}
}

impl Display for png_image {
	fn fmt(&self, form: &mut Formatter) -> Result<(), Error> {
		use std::ffi::CStr;

		let message: *const _ = &self.message[..];
		let message: *const [u8] = message as _;
		let message = unsafe {
			&*message
		};
		let message = CStr::from_bytes_with_nul(message).or(Err(Error))?;
		write!(form, "{}", message.to_str().or(Err(Error))?)
	}
}
