extern crate inger;

#[allow(dead_code, nonstandard_style)]
mod png;
#[allow(dead_code, nonstandard_style)]
mod pthread;

use png::png_image;
use std::convert::TryInto;
use std::ffi::c_void;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;
use std::os::raw::c_uint;
use std::ptr::null;
use std::ptr::null_mut;
use test::Bencher;

const TIMEOUT_US: u64 = 10_000;

#[bench]
fn direct(lo: &mut impl Bencher) {
	let (mut img, src, mut dest) = alloc_bufs().unwrap();
	lo.iter(|| unsafe {
		img.begin_read_from_memory(&src).unwrap();
		img.finish_read(&mut dest).unwrap();
	});
}

#[bench]
fn thread(lo: &mut impl Bencher) {
	use pthread::CLOCK_REALTIME;
	use pthread::PTHREAD_CANCEL_ASYNCHRONOUS;
	use pthread::clock_gettime;
	use pthread::pthread_cancel;
	use pthread::pthread_create;
	use pthread::pthread_setcanceltype;
	use pthread::pthread_timedjoin_np;
	use pthread::pthread_t;
	use pthread::timespec;
	use std::ops::AddAssign;

	let mut bufs = alloc_bufs().unwrap();
	let bufs: *mut _ = &mut bufs;
	let bufs: *mut c_void = bufs as _;
	let tout: i64 = TIMEOUT_US.try_into().unwrap();
	let tout = timespec {
		tv_nsec: tout * 1_000,
		tv_sec: 0,
	};
	let mut tid = pthread_t::default();
	unsafe {
		pthread_setcanceltype(PTHREAD_CANCEL_ASYNCHRONOUS.try_into().unwrap(), null_mut());
	}
	lo.iter(|| unsafe {
		let mut ts = timespec::default();
		clock_gettime(CLOCK_REALTIME as _, &mut ts);
		ts += &tout;
		pthread_create(&mut tid, null(), Some(main), bufs);
		if pthread_timedjoin_np(tid, null_mut(), &ts) != 0 {
			pthread_cancel(tid);
		}
	});

	unsafe extern fn main(img_src_dest: *mut c_void) -> *mut c_void {
		let img_src_dest: *mut (png_image, Box<_>, Box<_>) = img_src_dest as _;
		let (img, src, dest) = &mut *img_src_dest;
		img.begin_read_from_memory(src).unwrap();
		img.finish_read(dest).unwrap();
		null_mut()
	}

	impl AddAssign<&timespec> for timespec {
		fn add_assign(&mut self, other: &timespec) {
			let nsec = self.tv_nsec + other.tv_nsec;
			self.tv_sec += nsec / 1_000_000_000;
			self.tv_nsec += nsec % 1_000_000_000;
		}
	}
}

#[bench]
fn process(lo: &mut impl Bencher) {
	use png::__sigset_t as sigset_t;
	use png::pid_t;
	use png::timespec;
	use std::ffi::c_void;
	use std::os::raw::c_int;
	extern {
		fn exit(_: c_int) -> !;
		fn fork() -> pid_t;
		fn kill(_: pid_t, _: c_int) -> c_int;
		fn sigaddset(_: *mut sigset_t, _: c_int);
		fn sigemptyset(_: *mut sigset_t);
		fn signal(_: c_int, _: extern fn(c_int)) -> extern fn(c_int);
		fn sigtimedwait(_: *const sigset_t, _: Option<&mut c_void>, _: *const timespec) -> c_int;
		fn waitpid(_: pid_t, _: Option<&mut c_int>, _: c_int) -> pid_t;
	}

	const SIGCHLD: c_int = 17;
	const SIGKILL: c_int = 9;
	let (mut img, src, mut dest) = alloc_bufs().unwrap();
	let tout: i64 = TIMEOUT_US.try_into().unwrap();
	let tout = timespec {
		tv_nsec: tout * 1_000,
		tv_sec: 0,
	};
	let mut chld = sigset_t::default();
	unsafe {
		sigemptyset(&mut chld);
		sigaddset(&mut chld, SIGCHLD);
		signal(SIGCHLD, handler);
	}
	lo.iter(|| unsafe {
		let pid = fork();
		if pid == 0 {
			img.begin_read_from_memory(&src).unwrap();
			img.finish_read(&mut dest).unwrap();
			exit(0);
		} else {
			if sigtimedwait(&chld, None, &tout) != SIGCHLD {
				kill(pid, SIGKILL);
			}
			waitpid(pid, None, 0);
		}
	});

	extern fn handler(_: c_int) {}
}

#[bench]
fn preempt(lo: &mut impl Bencher) {
	use inger::launch;
	use std::cell::UnsafeCell;
	use std::ops::Deref;
	use std::ops::DerefMut;
	use std::sync::mpsc::channel;
	use std::thread::spawn;

	let bufs = UnsafeCell::from(alloc_bufs().unwrap());
	let (send, recv) = channel();
	let reaper = spawn(move || while let Some(fun) = recv.recv().unwrap() {
		drop(fun);
	});
	lo.iter(|| {
		let bufs = AssertSend (unsafe {
			unbound(&bufs)
		});
		let fun = launch(move || unsafe {
			let (img, src, dest) = &mut *bufs.get();
			img.begin_read_from_memory(src).unwrap();
			img.finish_read(dest).unwrap();
		}, TIMEOUT_US).unwrap();
		if fun.is_continuation() {
			send.send(fun.into()).unwrap();
		}
	});
	send.send(None).unwrap();
	reaper.join().unwrap();

	struct AssertSend<T> (T);

	unsafe impl<T> Send for AssertSend<T> {}

	impl<T> Deref for AssertSend<T> {
		type Target = T;

		fn deref(&self) -> &Self::Target {
			let Self (this) = self;
			this
		}
	}

	impl<T> DerefMut for AssertSend<T> {
		fn deref_mut(&mut self) -> &mut Self::Target {
			let Self (this) = self;
			this
		}
	}

	unsafe fn unbound<'a, 'b, T>(t: &'a T) -> &'b T {
		(|t: *const T| &*t)(t)
	}
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
	use std::fs::File;
	use std::io::Read;

	let mut file = File::open(file).ret("open()")?;
	let src = file.metadata().ret("fstat()")?.len().try_into().ret("u64::try_into()")?;
	let mut src = Vec::with_capacity(src);
	file.read_to_end(&mut src).ret("fread()")?;
	Ok(src.into_boxed_slice())
}

fn dest(img: &png_image) -> Result<Box<[u8]>, String> {
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
