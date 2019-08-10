extern crate nom_tutorial;

/// Prints a list of mounted filesystems similar to calling `mount` with no arguments.
fn main() -> std::result::Result<(), std::boxed::Box<dyn std::error::Error>> {
	for mount in nom_tutorial::mounts()? {
		println!("{}", mount?);
	}
	Ok(())
}
