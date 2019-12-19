#[cfg(test)]
mod black_box;

#[cfg(test)]
use self::bench::*;

#[cfg(test)]
mod bench {
	use bencher::Bencher;
	use bencher::benchmark_group;
	pub use bencher::benchmark_main;
	use super::black_box::black_box;

	benchmark_group![loops, loop100, loop1000, loop10000, loop100000, loop1000000, loop10000000];

	fn loop100(lo: &mut Bencher) {
		lo.iter(|| for _ in 0..100 {
			black_box();
		});
	}

	fn loop1000(lo: &mut Bencher) {
		lo.iter(|| for _ in 0..1000 {
			black_box();
		});
	}

	fn loop10000(lo: &mut Bencher) {
		lo.iter(|| for _ in 0..10000 {
			black_box();
		});
	}

	fn loop100000(lo: &mut Bencher) {
		lo.iter(|| for _ in 0..100000 {
			black_box();
		});
	}

	fn loop1000000(lo: &mut Bencher) {
		lo.iter(|| for _ in 0..1000000 {
			black_box();
		});
	}

	fn loop10000000(lo: &mut Bencher) {
		lo.iter(|| for _ in 0..10000000 {
			black_box();
		});
	}
}

#[cfg(not(test))]
mod bench {
	#[macro_export]
	macro_rules! benchmark_main {
		($ident:ident) => {};
	}
}

benchmark_main! {
	loops
}
