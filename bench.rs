#[allow(dead_code, nonstandard_style)]
mod png;

use png::png_image;
use std::ffi::c_void;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::os::raw::c_uint;
use std::ptr::null;
use std::ptr::null_mut;
use test::Bencher;

#[bench]
fn direct(lo: &mut impl Bencher) {
	let (mut img, src, mut dest) = alloc_bufs().unwrap();
	lo.iter(|| unsafe {
		img.begin_read_from_memory(&src).unwrap();
		img.finish_read(&mut dest).unwrap();
	});
}

fn alloc_bufs() -> Result<(png_image, Box<[u8]>, Box<[u8]>), String> {
	use std::env::var;

	let filename = var("FILENAME").ret("FILENAME")?;
	let src = src(&filename)?;
	let img = unsafe {
		png_image::new(&src)
	}?;
	let dest = dest(&img)?;
	Ok((img, src, dest))
}

fn src(file: &str) -> Result<Box<[u8]>, String> {
	use std::convert::TryInto;
	use std::fs::File;
	use std::io::Read;

	let mut file = File::open(file).ret("open()")?;
	let src = file.metadata().ret("fstat()")?.len().try_into().ret("u64::try_into()")?;
	let mut src = Vec::with_capacity(src);
	file.read_to_end(&mut src).ret("fread()")?;
	Ok(src.into_boxed_slice())
}

fn dest(img: &png_image) -> Result<Box<[u8]>, String> {
	use std::convert::TryInto;

	assert!(img.format != 0);

	let dest = img.size().try_into().ret("c_uint::try_into()")?;
	Ok(vec![0; dest].into_boxed_slice())
}

impl png_image {
	unsafe fn new(src: &[u8]) -> Result<Self, String> {
		let mut this = png_image::default();
		this.begin_read_from_memory(&src)?;
		Ok(this)
	}

	#[inline]
	unsafe fn begin_read_from_memory(&mut self, src: &[u8]) -> Result<(), String> {
		use png::PNG_FORMAT_RGB;
		use png::PNG_IMAGE_VERSION;
		use png::png_image_begin_read_from_memory;

		self.version = PNG_IMAGE_VERSION;
		self.opaque = null_mut();

		let buf: *const c_void = src.as_ptr() as _;
		if png_image_begin_read_from_memory(self, buf, src.len()) == 0 {
			Err(&*self).ret("png_image_begin_read_from_memory()")?;
		}
		self.format = PNG_FORMAT_RGB;

		Ok(())
	}

	#[inline]
	unsafe fn finish_read(&mut self, dest: &mut [u8]) -> Result<(), String> {
		use png::png_image_finish_read;

		let buf: *mut c_void = dest.as_mut_ptr() as _;
		if png_image_finish_read(self, null(), buf, 0, null_mut()) == 0 {
			Err(self).ret("png_image_finish_read()")?;
		}

		Ok(())
	}

	#[inline]
	fn size(&self) -> c_uint {
		(if self.format&0x08 != 0 { 1 } else { ((self.format & 0x04) >> 2)+1 })*self.height*((if self.format&0x08 != 0 { 1 } else { (self.format&(0x02|0x01))+1 }) * self.width)
	}
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
