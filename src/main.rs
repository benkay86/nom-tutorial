extern crate nom_tutorial;
use nom_tutorial::BoxError;

/// Prints a list of mounted filesystems similar to calling `mount` with no arguments.
fn main() -> std::result::Result<(), BoxError> {
	for mount in nom_tutorial::mounts()? {
		println!("{}", mount?);
	}
	Ok(())
}
