use crate::{depinfo::DependencyInfo, input::InputFile, output};
use memchr::{memchr2, memmem};
use std::io;

pub fn check_up2date(
	input: InputFile,
	template: &str,
	check_file: &mut dyn io::Read
) -> anyhow::Result<bool> {
	let mut check_buf = Vec::new();
	check_file.read_to_end(&mut check_buf)?;

	let search_key = b" [__cargo_doc2readme_dependencies_info]: ";
	if let Some(search_idx) = memmem::find(&check_buf, search_key) {
		let sub = &check_buf[search_idx + search_key.len()..];
		let end_idx = memchr2(b' ', b'\n', sub).unwrap_or(sub.len());
		let depinfo =
			match DependencyInfo::decode(String::from_utf8(sub[..end_idx].to_vec()).unwrap()) {
				Ok(depinfo) => depinfo,
				Err(_) => {
					// TODO we should print the error here so that the user knows the code isn't up to date because
					// of invalid dependency info
					return Ok(false);
				}
			};

		// ensure the input is up to date
		if !depinfo.check_input(template, &input.rustdoc) {
			// TODO we should print the error here so that the user knows the input has changed
			return Ok(false);
		}

		// ensure that the dependencies that were used in the readme still meet the current required
		// versions. dependencies that are missing in the readme don't matter.
		for (lib_name, (crate_name, version)) in &input.dependencies {
			if !depinfo.check_dependency(crate_name, Some(version), lib_name, true) {
				// TODO we should print the error here so that the user knows the dependency
				// has an incompatible version
				return Ok(false);
			}
		}

		// looks like everything is up to date
		return Ok(true);
	}

	// if no dependency info was available, do a bytewise comparison
	let mut output_buf = Vec::new();
	output::emit(input, template, &mut output_buf)?;
	Ok(output_buf == check_buf)
}
