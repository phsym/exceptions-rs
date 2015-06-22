//! Crate for emulating some exception-like behaviour. It gives the capability to have a 
//! detailed stack trace in errors, making it easier to trace where and why failures happened.
use std::string::ToString;
use std::io;
use std::io::{stdout, stderr, Write};
use std::fmt;

/// Represent an entry in a stack trace
pub struct StackEntry {
	/// The file where the trace was recorded
	pub file: &'static str,
	/// The line in the file
	pub line: u32,
	/// The code expression which caused the `Throwable`
	pub expr: &'static str
}

pub trait Throwable {
	/// Push stack trace information
	fn push_stack(&mut self, file: &'static str, line: u32, expr: &'static str);
	
	/// Get the stack trace
	fn get_stack_trace(&self) -> &Vec<StackEntry>;
	
	/// Return the message explaining what cause the Throwable` to be raised
	fn get_message(&self) -> &String;
	
	/// Print the stack trace to stdout. Code should instead call the macro `print_stack_trace!`
	#[allow(unused_must_use)] // Ignore if writing to stderr fails
	fn print_stack_trace(&self) {
		stdout().flush(); // Flush stdout to prevent mixes of stoud and stderr
		let mut err = stderr();
		writeln!(err, "{}", self.get_message());
		for s in self.get_stack_trace() {
			writeln!(err, "\tat {} [{}:{}]", s.expr, s.file, s.line); 
		}
		err.flush();
	}
}

/// Trait implented by types that can be converted
/// into a type implementing `Throwable`
pub trait IntoThrowable<T: Throwable> {
	/// Convert `self` into the Throwable `T`
	fn into_throwable(self) -> T;
}

impl <T: Throwable> IntoThrowable<T> for T {
	fn into_throwable(self) -> T {
		return self;
	}
}


pub struct Exception {
	message: String,
	stack: Vec<StackEntry>
}

impl Exception {
	pub fn new(message: String) -> Exception {
		return Exception{message: message, stack: Vec::new()};
	}
}

impl Throwable for Exception {
	fn push_stack(&mut self, file: &'static str, line: u32, expr: &'static str) {
		self.stack.insert(0, StackEntry{file: file, line: line, expr: expr});
	}
	
	fn get_stack_trace(&self) -> &Vec<StackEntry> {
		return &self.stack;
	}
	
	fn get_message(&self) -> &String {
		return &self.message;
	}
}

impl <'r> IntoThrowable<Exception> for &'r str {
	fn into_throwable(self) -> Exception {
		return Exception::new(self.to_string());
	}
}

impl IntoThrowable<Exception> for String {
	fn into_throwable(self) -> Exception {
		return Exception::new(self);
	}
}

impl IntoThrowable<Exception> for fmt::Error {
	fn into_throwable(self) -> Exception {
		return Exception::new("Fmt Exception".to_string());
	}
}

impl IntoThrowable<Exception> for io::Error {
	fn into_throwable(self) -> Exception {
		return Exception::new(self.to_string());
	}
}

//impl <E: error::Error> IntoThrowable<Exception> for E {
//	fn into_throwable(self) -> Exception {
//		return Exception::new(self.description().to_string());
//	}
//}

//impl <E: ToString> IntoThrowable<Exception> for E {
//	fn into_throwable(self) -> Exception {
//		return Exception::new(self.to_string());
//	}
//}

#[macro_export]
macro_rules! try {
	($expr:expr) => (
		match $expr {
			std::result::Result::Ok(e) => e,
			std::result::Result::Err(e) => {
				let mut th = e.into_throwable();
				th.push_stack(file!(), line!(), stringify!($expr));
				return std::result::Result::Err(th);
			},
		}
	);
	($($expr:expr); *;) => ($(try!($expr)); *;)
}

#[macro_export]
macro_rules! throw {
	($expr:expr) => (
		{
			let mut e = $expr.into_throwable();
			e.push_stack(file!(), line!(), stringify!(throw!($expr)));
			return std::result::Result::Err(e);
		}
	)
}

#[macro_export]
macro_rules! print_stack_trace {
	($expr:expr) => (
		{
			$expr.push_stack(file!(), line!(), stringify!(print_stack_trace!($expr)));
			$expr.print_stack_trace();
		}
	)
}

#[macro_export]
macro_rules! catch {
	($expr:expr) => (
		match $expr {
			std::result::Result::Ok(e) => std::result::Result::Ok(e),
			std::result::Result::Err(e) => {
				let mut th = e.into_throwable();
				th.push_stack(file!(), line!(), stringify!($expr));
				std::result::Result::Err(th)
			},
		}
	);
	
	($($expr:expr); *;) => (
		{
			let mut result;
			loop {
				$(
					result = catch!($expr);
					match result {
						std::result::Result::Err(..) => break,
						_ => {}
					}
				)*
				break;
			}
			result
		}
	)
}