// Write code here.
//
// To see what the code looks like after macro expansion:
//     $ cargo expand
//
// To run the code:
//     $ cargo run

#[cfg(feature = "run-builder")]
mod runner {
	use derive_builder::Builder;

	#[derive(Builder)]
	pub struct Command {
		executable: String,
		#[builder(each = "arg")]
		args: Vec<String>,
		#[builder(each = "env")]
		env: Vec<String>,
		current_dir: Option<String>,
	}

	pub fn run() {
		let command = Command::builder()
			.executable("cargo".to_owned())
			.arg("build".to_owned())
			.arg("--release".to_owned())
			.build()
			.unwrap();

		assert_eq!(command.executable, "cargo");
		assert_eq!(command.args, vec!["build", "--release"]);
	}
}

#[cfg(feature = "run-seq")]
mod runner {
	use seq::seq;

	pub fn run() {
		macro_rules! expand_to_nothing {
			($arg:literal) => {
				// nothing
			};
		}

		seq!(N in 0..4 {
			expand_to_nothing!(N);
		});
	}
}

#[cfg(feature = "run-sorted")]
mod runner {
	#[sorted::check]
	fn f(bytes: &[u8]) -> Option<u8> {
		#[sorted]
		match bytes {
			[] => Some(0),
			[a] => Some(*a),
			[a, b] => Some(a + b),
			_other => None,
		}
	}
}

fn main() {
	runner::run();
}
