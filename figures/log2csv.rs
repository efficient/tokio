#![feature(drain_filter)]
#![feature(linkage)]

use std::borrow::Cow;
use std::fs;

fn main() -> Result<(), Cow<'static, str>> {
	let args = args()?;
	let series: Vec<_> = args.series.iter().map(|series| &series.name[..]).collect();
	println!("{},{}", args.title, args.y_label);
	println!("{},{}", args.x_label, series.join(","));

	let script: Option<_> = args.script.as_ref().map(|script| &script[..]);
	for index in 0..args.series[0].files.len() {
		let x = args.series[0].files[index].x_num();
		print!("{}", transform(x));
		for file in args.series.iter().map(|series| &series.files[index]) {
			debug_assert!(file.x_num() == x);
			print!(",{}", mean(&file, script)?);
		}
		println!();
	}

	Ok(())
}

fn transform(x: usize) -> usize {
	use std::mem::transmute;
	extern "C" {
		#[linkage = "extern_weak"]
		static transform_x: *const usize;
	}

	unsafe {
		if transform_x.is_null() {
			x
		} else {
			let transform: extern "C" fn(usize) -> usize = transmute(transform_x);
			transform(x)
		}
	}
}

fn mean(file: &File, script: Option<&str>) -> Result<f64, String> {
	use std::io::BufReader;
	use std::io::Read;
	use std::process::Command;
	use std::process::Stdio;

	let mut lines;
	if let Some(script) = script {
		let sed = Command::new("sed").arg("-ne").arg(script).arg(&file.name)
			.stderr(Stdio::inherit()).output().map_err(|or| format!("{}", or))?;
		if ! sed.status.success() {
			Err(format!(
				"sed: exited with failing status ({})",
				sed.status.code().map(|code|
					format!("{}", code)
				).unwrap_or(String::from("signaled")),
			))?;
		}

		lines = String::from_utf8(sed.stdout).map_err(|or| format!("{}", or))?;
	} else {
		lines = String::new();

		let mut reader = BufReader::new(&file.file);
		reader.read_to_string(&mut lines).map_err(|or|
			format!("{}: {}", file.name, or)
		)?;
	}

	let numer: f64 = lines.lines().map(|line| {
		let line: f64 = line.parse().unwrap();
		line
	}).sum();
	let denom: f64 = lines.lines().count() as _;

	Ok(numer / denom)
}

struct Args {
	title: String,
	x_label: String,
	y_label: String,
	script: Option<String>,
	series: Vec<Series>,
}

struct Series {
	name: String,
	files: Vec<File>,
}

struct File {
	file: fs::File,
	name: String,
}

impl File {
	const DELIM: char = '_';

	fn open(name: String) -> Result<Self, String> {
		Self::try_series(&name).ok_or_else(||
			format!("{}: not {}-delimited", name, Self::DELIM)
		)?;
		let this = Self {
			file: fs::File::open(&name).map_err(|or| format!("{}: {}", name, or))?,
			name,
		};
		Self::try_parse(this.x()).ok_or_else(||
			format!("{}: nonnumeric x coordinate '{}'", this.name, this.x())
		)?;
		Ok(this)
	}

	fn basename(&self) -> &str {
		self.name.split('/').last().unwrap_or(&self.name)
	}

	fn series(&self) -> &str {
		Self::try_series(&self.name).unwrap()
	}

	fn x(&self) -> &str {
		self.basename().split(Self::DELIM).next().unwrap()
	}

	fn x_num(&self) -> usize {
		Self::try_parse(self.x()).unwrap()
	}

	fn try_parse(num: &str) -> Option<usize> {
		let mut numb = num.split("0x").last().unwrap();
		if numb == num {
			numb = num.split("0X").last().unwrap();
		}
		let radix = if numb == num { 10 } else { 16 };
		usize::from_str_radix(numb, radix).ok()
	}

	fn try_series(name: &str) -> Option<&str> {
		name.split(Self::DELIM).skip(1).next()
	}
}

fn args() -> Result<Args, Cow<'static, str>> {
	use std::env::args;

	fn usage(prog: &str) -> String {
		format!("\
			USAGE: {} -s <series>[,<series>]... [-l <series label>[,<series label>]... \
			[-n <sed -n script>] [-t <title>] [-x <label>] [-y <label>] <filenames>...\
		", prog)
	}

	let mut labels = None;
	let mut series = None;
	let mut script = None;
	let mut title = Cow::from("");
	let mut x_label = Cow::from("");
	let mut y_label = Cow::from("");

	let mut args = args().peekable();
	let prog = args.next().unwrap_or_else(||
		module_path!().split(':').next().unwrap().to_owned()
	);
	while args.peek().map_or(None, |switch| switch.chars().next())
		.map_or(false, |head| head == '-') {
		let arg = args.next().unwrap();
		let subarg = Cow::from(
			args.next().ok_or_else(|| format!("{}: switch requires subargument", arg))?
		);
		match &arg[..] {
			"-l" => labels = Some(subarg),
			"-n" => script = Some(subarg.into_owned()),
			"-s" => series = Some(subarg),
			"-t" => title = subarg,
			"-x" => x_label = subarg,
			"-y" => y_label = subarg,
			_ => Err(usage(&prog))?,
		}
	}

	let series = series.ok_or_else(|| usage(&prog))?;
	let series = series.split(',');
	let labels = if let Some(labels) = &labels {
		let labels = labels.split(',');
		if labels.clone().count() != series.clone().count() {
			Err("-l: list must have the same number of elements as that of -s")?;
		}
		labels
	} else {
		series.clone()
	};

	if args.len() == 0 {
		Err(usage(&prog))?;
	}
	let files: Vec<_> = args.map(|filename| File::open(filename)).collect();
	if let Some(Err(or)) = files.iter().find(|error| error.is_err()) {
		Err(or.clone())?;
	}

	let mut files: Vec<_> = files.into_iter().map(|file| file.unwrap()).collect();
	let mut series: Vec<_> = series.zip(labels).map(|(series, name)| Series {
		name: name.to_owned(),
		files: files.drain_filter(|file| file.series() == series).collect(),
	}).collect();
	for series in &mut series {
		series.files.sort_unstable_by_key(|file| file.x_num());
	}

	Ok(Args {
		title: title.into_owned(),
		x_label: x_label.into_owned(),
		y_label: y_label.into_owned(),
		script,
		series,
	})
}
