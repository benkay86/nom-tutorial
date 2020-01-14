//! Example crate demonstrating how to use nom to parse `/proc/mounts`.  Browse crates.io for sys-mount, proc-mounts, and libmount for more stable, usable crates.

// Needed to use traits associated with std::io::BufReader.
use std::io::BufRead;
use std::io::Read;

/// Describes a mounted filesystem, see `man 8 mount` for more details.
#[derive(Clone, Default, Debug)]
pub struct Mount {
	/// The device from which the filesystem is mounted, e.g. /dev/sda1
	pub device: std::string::String,
	/// Where in the root filesystem the device is mounted, e.g. /mnt/disk
	pub mount_point: std::string::String,
	/// The filesystem type, e.g. ext4
	pub file_system_type: std::string::String,
	/// A vector of mount options, e.g. ["ro", "nosuid"]
	/// Note: This could also be implemented as a set (e.g. std::collections::HashSet)
	pub options: std::vec::Vec<std::string::String>,
}

/// Implements `Display` for `Mount` to simulate behavior of Unix mount command.
/// 
/// # Examples
/// ```
/// # use nom_tutorial::Mount;
/// # use std::string::String;
/// let mount = Mount {
/// 	device: String::from("/dev/sda1"),
/// 	mount_point: String::from("/mnt/disk"),
/// 	file_system_type: String::from("ext4"),
/// 	options: vec![String::from("ro"), String::from("nosuid")]
/// };
/// assert!(mount.to_string() == "/dev/sda1 on /mnt/disk type ext4 (ro,nosuid)");
/// ```
impl std::fmt::Display for Mount {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} on {} type {} ({})", self.device, self.mount_point, self.file_system_type, self.options.join(","))
	}
}

/// Structure that accesses `/proc/mounts` and iterates over the contained mounts.
/// 
/// You can generate an instance by calling [Mounts::new()] or the convenience method [mounts()].  Instantiation may fail if `/proc/mounts` does not exist or you do not have access to read it.  You can access each individual mount through an iterator with [Mounts::into_iter()](std::iter::IntoIterator::into_iter) for a consuming iterator or [Mounts::iter_mut()] for a mutable iterator.  Note that there is no immutable borrowed iterator `Mounts::iter()`.  An instance of `Mounts` really isn't useful for anything except iterating over the contained mounts.
/// # Examples
/// 
/// ```
/// # use nom_tutorial;
/// for mount in nom_tutorial::mounts().unwrap() {
///   println!("{}", mount.unwrap());
/// }
pub struct Mounts {
	buf_reader: std::io::BufReader<std::fs::File>
}

impl Mounts {
	/// Returns a new Mounts instance.  You can also call [mounts()] for convenience.
	pub fn new() -> std::result::Result<Mounts, std::io::Error> {
		let file = std::fs::File::open("/proc/mounts")?;
		Ok( Mounts { buf_reader: std::io::BufReader::new(file) } )
	}
}

impl IntoIterator for Mounts {
	type Item = std::result::Result<Mount, std::boxed::Box<dyn std::error::Error>>;
	type IntoIter = MountsIntoIterator;
	
	/// Consuming iterator, used similarly to mutable iterator.  See [Mounts::iter_mut()] for example.
	fn into_iter(self) -> Self::IntoIter {
		MountsIntoIterator { lines: self.buf_reader.lines() }
	}
}

impl<'a> IntoIterator for &'a mut Mounts {
	type Item = std::result::Result<Mount, std::boxed::Box<dyn std::error::Error>>;
	type IntoIter = MountsIteratorMut<'a>;
	
	/// Mutable iterator, see [Mounts::iter_mut()].
	fn into_iter(self) -> Self::IntoIter {
		MountsIteratorMut { lines: self.buf_reader.by_ref().lines() }
	}
}

/// Consuming iterator for [Mounts].
pub struct MountsIntoIterator {
	lines: std::io::Lines<std::io::BufReader<std::fs::File>>
}

impl std::iter::Iterator for MountsIntoIterator {
	type Item = std::result::Result<Mount, std::boxed::Box<dyn std::error::Error>>;
	
	/// Returns the next line in `/proc/mounts` as a [Mount].  If there is a problem reading or parsing `/proc/mounts` returns an error.  See [Mounts::iter_mut()] for an analagous example using a mutable iterator.
	fn next(&mut self) -> std::option::Option<Self::Item> {
		match self.lines.next() {
			Some(line) => match line {
				Ok(line) => match parsers::parse_line(&line[..]) {
					Ok( (_, m) ) => Some(Ok(m)),
					Err(_) => Some(Err(ParseError::default().into()))
				},
				Err(e) => Some(Err(e.into()))
			}
			None => None
		}
	}
}

/// Mutable iterator for `Mounts`.
pub struct MountsIteratorMut<'a> {
	lines: std::io::Lines<&'a mut std::io::BufReader<std::fs::File>>
}

impl<'a> std::iter::Iterator for MountsIteratorMut<'a> {
	type Item = std::result::Result<Mount, std::boxed::Box<dyn std::error::Error>>;
	
	// Returns the next line in `/proc/mounts` as a [Mount].  See [Mounts::iter_mut()] for an example.
	fn next(&mut self) -> std::option::Option<Self::Item> {
		match self.lines.next() {
			Some(line) => match line {
				Ok(line) => match parsers::parse_line(&line[..]) {
					Ok( (_, m) ) => Some(Ok(m)),
					Err(_) => Some(Err(ParseError::default().into()))
				},
				Err(e) => Some(Err(e.into()))
			}
			None => None
		}
	}
}

impl<'a> Mounts {
	// There is no non-mutable iterator.
	
	/// Mutable iterator.
	/// # Examples
	/// ```
	/// # use nom_tutorial;
	/// let mut iter = nom_tutorial::mounts().expect("Couldn't access /proc/mounts.").into_iter();
	/// match iter.next() {
	/// 	Some(m) => match m {
	/// 		Ok(m) => eprintln!("Here is a mounted filesystem: {}", m),
	/// 		Err(e) => eprintln!("There was an error parsing the next line in /proc/mounts: {}", e)
	/// 	},
	/// 	None => eprintln!("There are no mounted filesystems.")
	/// }
	/// ```
	pub fn iter_mut(&'a mut self) -> MountsIteratorMut<'a> {
		self.into_iter()
	}
}

/// The nom crate's error types do not cleanly implement std::error::Error.  This structure is a custom error type that implements Error.  In this very basic implementation of the Display trait we simply indicate that a parsing error has occurred without going into details.
#[derive(Default)]
pub struct ParseError;
impl std::fmt::Display for ParseError {
	/// Indicate that a parsing error occured.
	/// # Examples
	/// ```
	/// # use nom_tutorial::ParseError;
	/// assert_eq!(format!("{}", ParseError::default()), "A parsing error occurred.")
	/// 
	/// ```
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "A parsing error occurred.")
	}
}
impl std::fmt::Debug for ParseError {
	/// Indicate that a parsing error occurred.  In this very simple implementation, the debug output is the same as the display output (i.e. there is no additional information to add), so we can just call the `fmt()` method we implemented for `Display`.
	/// # Examples
	/// ```
	/// # use nom_tutorial::ParseError;
	/// assert_eq!(format!("{:?}", ParseError::default()), "A parsing error occurred.")
	/// 
	/// ```
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		<ParseError as std::fmt::Display>::fmt(self, f)
	}
}
impl std::error::Error for ParseError { }

// Encapsulate individual nom parsers in a private submodule.  The `pub(self)` keyword allows the inner method [parsers::parse_line()] to be called by code within this module, but not my users of our crate.
pub(self) mod parsers {
	use super::Mount;
	
	// Extract a string that does not contain whitespace (space or tab).  Anything else goes.
	fn not_whitespace(i: &str) -> nom::IResult<&str, &str> {
		nom::bytes::complete::is_not(" \t")(i)
	}
	
	// Replace the sequence 040 with a space.
	fn escaped_space(i: &str) -> nom::IResult<&str, &str> {
		nom::combinator::value(" ", nom::bytes::complete::tag("040"))(i)
	}
	
	// Replace the escaped sequence \ with a \.  The inner parser `nom::character::complete::char` returns a `char` instead of a `&str`, so we wrap it in a `nom::combinator::recognize`, which returns that `char` as an `&str` if the inner parser succeeds, and returns an error if the inner parser fails.
	fn escaped_backslash(i: &str) -> nom::IResult<&str, &str> {
		nom::combinator::recognize(nom::character::complete::char('\\'))(i)
	}
	
	// Replace all instances of \040 in a string with a space.
	// Replace \\ with a \.
	fn transform_escaped(i: &str) -> nom::IResult<&str, std::string::String> {
		nom::bytes::complete::escaped_transform(nom::bytes::complete::is_not("\\"), '\\', nom::branch::alt((escaped_backslash, escaped_space)))(i)
	}
	
	// Parse the options of a mount into a comma separated vector of strings.  The options string is terminated by a whitespace.
	// Here we use `nom::combinator::map_parser` to extract the full whitespace-terminated options string and then pass it in to `transform_escaped` to process escaped characters.  Then the transformed string is split into a comma-delimited vector of strings by `nom::multi::separated_list`.
	fn mount_opts(i: &str) -> nom::IResult<&str, std::vec::Vec<std::string::String>> {
		nom::multi::separated_list(nom::character::complete::char(','), nom::combinator::map_parser(nom::bytes::complete::is_not(", \t"),transform_escaped))(i)
	}
	
	// Parse a line from `/proc/mounts` into a Mount struct.  This is perhaps the most complex looking parser, but it is actually one of the simplest because we build upon each of the parsers defined above.  Let's break it down parser by parser:
	// # `nom::combinator::all_consuming` generates an error if there is any leftover input.  This will force nom to generate an error if there is unexpected input at the end of a line in `/proc/mounts`, for example:
	// ```ignore
	// /dev/sda1 /mnt/disk ext4 defaults 0 0 this_last_part_shouldn't_be_here
	// ```
	// 
	// `nom::sequence::tuple` generates a `Result<Ok(remaining_input: &str, output_tuple), Error>`.  Although it looks complicated, we can very easily destructure that tuple.  Each sub/inner parser we pass to `nom::sequence::tuple` generates its own element within the tuple.  We can ignore the whitespace by matching it with `_` and destructure the other elements of the tuple as the variabels we are insterested such as `device`, `mount_point`, etc.  If everything goes as planned we return a new instance of the mount `Mount` structure populated with the variables we destructured from the tuple.
	// ```ignore
	// let (device, _, mount_point /*, ...*/)  =  /* tuple returned by all_consuming(tuple()) --> */  ("/dev/sda1", " ", "/mnt/disk" /*, ...*/);
	// let mount = Mount { device: device.to_string(), mount_point: mount_point.to_string() /*, ...*/ };
	// ```
	pub fn parse_line(i: &str) -> nom::IResult<&str, Mount> {
		match nom::combinator::all_consuming(nom::sequence::tuple((
			nom::combinator::map_parser(not_whitespace, transform_escaped), // device
			nom::character::complete::space1,
			nom::combinator::map_parser(not_whitespace, transform_escaped), // mount_point
			nom::character::complete::space1,
			not_whitespace, // file_system_type
			nom::character::complete::space1,
			mount_opts, // options
			nom::character::complete::space1,
			nom::character::complete::char('0'),
			nom::character::complete::space1,
			nom::character::complete::char('0'),
			nom::character::complete::space0,
		)))(i) {
				Ok((remaining_input, (
				device,
				_, // whitespace
				mount_point,
				_, // whitespace
				file_system_type,
				_, // whitespace
				options,
				_, // whitespace
				_, // 0
				_, // whitespace
				_, // 0
				_, // optional whitespace
			))) => {
				Ok((remaining_input, Mount { 
					device: device,
					mount_point: mount_point,
					file_system_type: file_system_type.to_string(),
					options: options
				}))
			}
			Err(e) => Err(e)
		}
	}
	
	// Alternative version of `parse_line()` above that performs the same
	// function using a different style.  Rather than parsing the entire line at
	// once with one big `nom::sequence::tuple` we break the parsing up into
	// multiple separate statements.  Each statement runs a parser that returns
	// an `Ok(remaining_input, value)`.  At the end of each statement we have
	// the `?` operator, which unwraps the result and returns early with an
	// error if parsing failed.  The remaining input from each parser is used as
	// the input of each subsequent parser.  Values are assigned to temporary
	// variables that are used to construct a `Mount` object at the end of the
	// function.  Values that are not needed are discarded by assigning to `_`. 
	#[allow(unused)]
	pub fn parse_line_alternate(i: &str) -> nom::IResult<&str, Mount> {
		let (i, device) = nom::combinator::map_parser(not_whitespace, transform_escaped)(i)?; // device
		let (i, _) = nom::character::complete::space1(i)?;
		let (i, mount_point) = nom::combinator::map_parser(not_whitespace, transform_escaped)(i)?; // mount_point
		let (i, _) = nom::character::complete::space1(i)?;
		let (i, file_system_type) = not_whitespace(i)?; // file_system_type
		let (i, _) = nom::character::complete::space1(i)?;
		let (i, options) = mount_opts(i)?; // options
		let (i, _) = nom::combinator::all_consuming(nom::sequence::tuple((
			nom::character::complete::space1,
			nom::character::complete::char('0'),
			nom::character::complete::space1,
			nom::character::complete::char('0'),
			nom::character::complete::space0
		)))(i)?;
		Ok((i, Mount {
			device: device,
			mount_point: mount_point,
			file_system_type: file_system_type.to_string(),
			options:options
		}))
	}
	
	#[cfg(test)]
	mod tests {
		use super::*;
		
		// Extracts a string that does not contain whitespace, i.e. comma or tab.
		#[test]
		fn test_not_whitespace() {
			assert_eq!(not_whitespace("abcd efg"), Ok((" efg", "abcd")));
			assert_eq!(not_whitespace("abcd\tefg"), Ok(("\tefg", "abcd")));
			assert_eq!(not_whitespace(" abcdefg"), Err(nom::Err::Error((" abcdefg", nom::error::ErrorKind::IsNot))));
		}
		
		// Converts 040 to a space.  Does not actually recognize a literal space.
		#[test]
		fn test_escaped_space() {
			assert_eq!(escaped_space("040"), Ok(("", " ")));
			assert_eq!(escaped_space(" "), Err(nom::Err::Error((" ", nom::error::ErrorKind::Tag))));
		}
		
		// Converts `char` \ to `&str` \.
		#[test]
		fn test_escaped_backslash() {
			assert_eq!(escaped_backslash("\\"), Ok(("", "\\")));
			assert_eq!(escaped_backslash("not a backslash"), Err(nom::Err::Error(("not a backslash", nom::error::ErrorKind::Char))));
		}
		
		// Recognizes each escape sequence and transfoms it to the escaped literal.
		// For example, each \040 is transformed into a space.
		#[test]
		fn test_transform_escaped() {
			assert_eq!(transform_escaped("abc\\040def\\\\g\\040h"), Ok(("", std::string::String::from("abc def\\g h"))));
			assert_eq!(transform_escaped("\\bad"), Err(nom::Err::Error(("bad", nom::error::ErrorKind::Tag))));
		}
		
		// Parses a comma separated list of mount options, which might contain spaces.
		#[test]
		fn test_mount_opts() {
			assert_eq!(mount_opts("a,bc,d\\040e"), Ok(("", vec!["a".to_string(), "bc".to_string(), "d e".to_string()])));
		}
		
		// Parses a line from /proc/mounts
		#[test]
		fn test_parse_line() {
			let mount1 = Mount{
				device: "device".to_string(),
				mount_point: "mount_point".to_string(),
				file_system_type: "file_system_type".to_string(),
				options: vec!["options".to_string(), "a".to_string(), "b=c".to_string(), "d e".to_string()]
			};
			let (_, mount2) = parse_line("device mount_point file_system_type options,a,b=c,d\\040e 0 0").unwrap();
			assert_eq!(mount1.device, mount2.device);
			assert_eq!(mount1.mount_point, mount2.mount_point);
			assert_eq!(mount1.file_system_type, mount2.file_system_type);
			assert_eq!(mount1.options, mount2.options);
		}
		
		// Parses a line from /proc/mounts
		#[test]
		fn test_parse_line_alternate() {
			let mount1 = Mount{
				device: "device".to_string(),
				mount_point: "mount_point".to_string(),
				file_system_type: "file_system_type".to_string(),
				options: vec!["options".to_string(), "a".to_string(), "b=c".to_string(), "d e".to_string()]
			};
			let (_, mount2) = parse_line_alternate("device mount_point file_system_type options,a,b=c,d\\040e 0 0").unwrap();
			assert_eq!(mount1.device, mount2.device);
			assert_eq!(mount1.mount_point, mount2.mount_point);
			assert_eq!(mount1.file_system_type, mount2.file_system_type);
			assert_eq!(mount1.options, mount2.options);
		}
	}
}

/// Convenienve method equivalent to `Mounts::new()`.
pub fn mounts() -> std::result::Result<Mounts, std::io::Error> {
	Mounts::new()
}
