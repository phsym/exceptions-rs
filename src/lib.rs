//! Crate for emulating some exception-like behaviour. It gives the capability to have a 
//! detailed stack trace in errors, making it easier to trace where and why failures happened.
use std::string::ToString;
use std::io;
use std::io::{stdout, stderr, Write};
use std::fmt;
use std::ops::Deref;

/// Represent an entry in a stack trace
pub struct StackEntry {
	/// The file where the trace was recorded
	pub file: &'static str,
	/// The line in the file
	pub line: u32,
	/// The code expression which caused the `Throwable`
	pub expr: &'static str
}

/// Represent an object that can be thrown and can register the stack informations
/// when beeing propagetd accross the call stack
pub trait Throwable {
	/// Push stack trace information
	fn push_stack(&mut self, file: &'static str, line: u32, expr: &'static str);
	
	/// Get the stack trace
	fn get_stack_trace(&self) -> &Vec<StackEntry>;
	
	/// Return the message explaining what caused the `Throwable` to be raised
	fn get_message(&self) -> &str;
	
	/// Get the `Throwable` cause (if any) that caused this `Throwable` to be thrown
	fn get_cause(&self) -> Option<&Throwable>;
	
	/// Print the stack trace to stdout. Code should instead call the `print_stack_trace!` macro
	#[allow(unused_must_use)] // Ignore if writing to stderr fails
	fn print_stack_trace(&self) {
		stdout().flush(); // Flush stdout to prevent mixes of stoud and stderr
		let mut err = stderr();
		writeln!(err, "{}", self.get_message());
		for s in self.get_stack_trace() {
			writeln!(err, "\tat {} [{}:{}]", s.expr, s.file, s.line); 
		}
		if let Some(cause) = self.get_cause() {
			write!(err, "Caused by: ");
			cause.print_stack_trace();
		} 
		err.flush();
	}
}

impl <T: Throwable+?Sized> Throwable for Box<T> {
	fn push_stack(&mut self, file: &'static str, line: u32, expr: &'static str) {
		(**self).push_stack(file, line, expr);
	}
	
	fn get_stack_trace(&self) -> &Vec<StackEntry> {
		return (**self).get_stack_trace();
	}
	
	fn get_message(&self) -> &str {
		return (**self).get_message();
	}
	
	fn get_cause(&self) -> Option<&Throwable> {
		return (**self).get_cause();
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
	stack: Vec<StackEntry>,
	cause: Option<Box<Throwable>>
}

impl Exception {
	pub fn new(message: String) -> Exception {
		return Exception{message: message, stack: Vec::new(), cause: None};
	}
	
	pub fn new_with_cause<T: Throwable+'static>(message: String, cause: T) -> Exception {
		//FIXME: Take Box<T> or Box<Throwable> as cause argument
		return Exception{message: message, stack: Vec::new(), cause: Some(Box::new(cause))};
	}
}

impl Throwable for Exception {
	fn push_stack(&mut self, file: &'static str, line: u32, expr: &'static str) {
		self.stack.insert(0, StackEntry{file: file, line: line, expr: expr});
	}
	
	fn get_stack_trace(&self) -> &Vec<StackEntry> {
		return &self.stack;
	}
	
	fn get_message(&self) -> &str {
		return &self.message;
	}
	
	fn get_cause(&self) -> Option<&Throwable> {
//		return match self.cause {
//			Some(ref c) => Some(c),
//			None => None
//		};
		if self.cause.is_some() {
			return Some(self.cause.as_ref().unwrap().deref())
		}
		return None;
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
		return Exception::new("Formatting Exception".to_string());
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