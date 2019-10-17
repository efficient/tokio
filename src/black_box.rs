#[inline]
pub fn black_box() {
	#[link(name = "black_box")]
	extern {
		fn black_box();
	}
	unsafe {
		black_box();
	}
}
