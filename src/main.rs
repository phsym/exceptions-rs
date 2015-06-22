#[macro_use] extern crate exceptions;
use exceptions::*;

fn test1() -> Result<(), Exception> {
	println!("test1()");
	try!(test0());
	throw!("This is an exception");
}

fn test2() -> Result<(), Exception> {
	println!("test2()");
	try!(test1());
	return Ok(());
}

fn test3() -> Result<(), Exception> {
	println!("test3()");
	try! {
		test0();
		test0();
		test2();
	}
	try!(test2());
	return Ok(());
}

fn test0() -> Result<(), &'static str> {
	println!("test0()");
	return Ok(());
//	return Err("This is an error");
}

fn nothing() {
	return;
}

use std::any::Any;
fn main() {
	let a = nothing();
	println!("{:?}", (&a as &Any).is::<&Result<(), &Throwable>>());
    println!("Hello, world!");
    match catch!(test3(); test0();) {
    	Err(mut e) => print_stack_trace!(e),
    	_ => ()
    };
    println!("End");
}



