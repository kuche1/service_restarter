
// #![deny(warnings)]
// like `-Werror`

use std::process::ExitCode;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use std::convert::TryInto;
use std::process::Command;
use std::io::Write;
use std::fs::File;
use std::thread;
use std::time;
use std::env;
use std::fs;

const SERVICE_SLEEP_AFTER_RESTART_SEC:u16 = 40;
// give each service that much time before going on to the next one
// TODO ideally, this would also be cmdline

////
//// get time, compare time
////

fn time_now(utc:u8) -> (u8, u8) {
	let now = SystemTime::now();

	// println!("now: {:?}", now); // {:#?} pretty print ; {:?} regular print

	let duration = now.duration_since(UNIX_EPOCH).unwrap();
	// UNIX_EPOCH -> 1970-01-01 00:00:00 UTC

	let seconds =
		duration.as_secs()
		+ (utc as u64) * 60 * 60 // utc
		+ 60 * 60 * 24 * 365 * 2; // change base from 1970 to 1972 (a leap year)
	let minutes = seconds / 60;
	let hours = minutes / 60;
	let days = hours / 24;
	let years_4 = days / (366*1 + 365*3); // this will break at year 2100 -> https://www.calendar.best/images/leap-years/list-of-leap-years-from-1900-to-3000-600.jpg

	let seconds_since_start_of_day = seconds % (60 * 60 * 24);

	let minutes_since_start_of_day = seconds_since_start_of_day / 60;

	// println!("minutes since start of day {minutes_since_start_of_day}");

	let hour:u8 = (minutes_since_start_of_day / 60).try_into().unwrap();
	let minute:u8 = (minutes_since_start_of_day % 60).try_into().unwrap();

	println!("current time: {}:{}", hour, minute);

	return (hour, minute);
}

// greater than
// A > B
fn time_gt(hour_a:u8, minute_a:u8, hour_b:u8, minute_b:u8) -> bool {
	if hour_a > hour_b {
		return true;
	}
	if hour_a < hour_b {
		return false;
	}

	if minute_a > minute_b {
		return true;
	}
	if minute_a < minute_b {
		return false;
	}

	return false;
}

// greater or equal
// A >= B
fn time_ge(hour_a:u8, minute_a:u8, hour_b:u8, minute_b:u8) -> bool {
	if hour_a > hour_b {
		return true;
	}
	if hour_a < hour_b {
		return false;
	}

	if minute_a > minute_b {
		return true;
	}
	if minute_a < minute_b {
		return false;
	}

	return true;
}

// less than
// A < B
fn time_lt(hour_a:u8, minute_a:u8, hour_b:u8, minute_b:u8) -> bool {
	return !time_ge(hour_a, minute_a, hour_b, minute_b);
}

////
//// sleep
////

fn sleep_ms(time_ms:u64){
	let time_ms = time::Duration::from_millis(time_ms);
	thread::sleep(time_ms);
}

fn sleep_sec(msg:&str, time_sec:u16){
	println!("{}; sleeping for {} seconds", msg, time_sec);
	sleep_ms( (1_000 * time_sec).into() );
}

fn sleep_1hour(msg:&str){
	println!("{}; sleeping for 1 hour", msg);
	sleep_ms(1_000 * 60 * 60);
}

////
//// log
////

fn log(msg:String){
	// TODO put timestamp in file name

	unsafe{
		fs::create_dir_all(ERROR_FOLDER.clone()).unwrap(); // using a mutable static variable is unsafe
	}

	let mut f = File
		::options()
		.append(true)
		.create(true)
		.open(format!("{}/{}", unsafe{&ERROR_FOLDER}, "deleteme"))
		.unwrap();

	writeln!(&mut f, "{}", msg).unwrap();
}

////
//// service commands
////

fn service_restart_if_running(name:&str){
	println!("working on {}", name);

	let cmd = Command
		::new("systemctl")
		.arg("try-restart")
		.arg(&name)
		.output()
		.unwrap();

	if cmd.status.success() {

		sleep_sec("restarted, giving service time to breathe", SERVICE_SLEEP_AFTER_RESTART_SEC);

	}else{

		let generic_msg = format!("could not restart service: {}", name);

		println!("{generic_msg}");

		let mut msg = String::from("");

		msg += &generic_msg;

		if cmd.stdout.len() > 0 {
			msg += "\n\n";
			msg += "stdout:\n";
			msg += "```\n";
			msg += &String::from_utf8(cmd.stdout.clone()).unwrap();
			msg += "```";
		}

		if cmd.stderr.len() > 0 {
			msg += "\n\n";
			msg += "stderr:\n";
			msg += "```\n";
			msg += &String::from_utf8(cmd.stderr.clone()).unwrap();
			msg += "```";
		}

		log(msg);

	}

	println!();
}

////
//// main
////

static mut ERROR_FOLDER: String = String::new();
// yeah, a global variable, I know

fn main() -> ExitCode {
	let args: Vec<String> = env::args().collect();

	let arg_idx = 0;

	// parse error folder

	let arg_idx = arg_idx + 1;

	if args.len() <= arg_idx {
		println!("missing argument: error_folder (string)");
		return ExitCode::FAILURE;
	}

	unsafe{
		ERROR_FOLDER = args[arg_idx].clone(); // unsing mutable static variables is unsafe
	}

	// parse timezone

	let arg_idx = arg_idx + 1;

	if args.len() <= arg_idx {
		println!("missing argument: timezone_utc (u8)");
		return ExitCode::FAILURE;
	}

	let timezone = &args[arg_idx];

	let timezone: u8 = match timezone.parse() {
		Ok(val) => val,
		Err(_err) => {
			println!("invalid u8 value: {timezone}");
			return ExitCode::FAILURE;
		},
	};

	// parse restart hour

	let arg_idx = arg_idx + 1;

	if args.len() <= arg_idx {
		println!("missing argument: restart_time_hour (u8)");
		return ExitCode::FAILURE;
	}

	let restart_time_hour = &args[arg_idx];

	let restart_time_hour: u8 = match restart_time_hour.parse() {
		Ok(val) => val,
		Err(_err) => {
			println!("invalid u8 value: {restart_time_hour}");
			return ExitCode::FAILURE;
		},
	};

	// parse restart minute

	let arg_idx = arg_idx + 1;

	if args.len() <= arg_idx {
		println!("missing argument: restart_time_minute (u8)");
		return ExitCode::FAILURE;
	}

	let restart_time_minute = &args[arg_idx];

	let restart_time_minute: u8 = match restart_time_minute.parse() {
		Ok(val) => val,
		Err(_err) => {
			println!("invalid u8 value: {restart_time_minute}");
			return ExitCode::FAILURE;
		},
	};

	// parse services to restart

	let arg_idx = arg_idx + 1;

	if args.len() <= arg_idx {
		println!("missing arguments: services_to_restart (array of string - all remaining arguments)");
		return ExitCode::FAILURE;
	}

	let services_to_restart = &args[arg_idx..];

	// wait for the right time

	loop{
		let (hour, minute) = time_now(timezone);
		if time_lt(hour, minute, restart_time_hour, restart_time_minute) {
			break
		}
		sleep_1hour("too late to restart"); // TODO ideally, this would also be cmdline
	}

	loop{
		let (hour, minute) = time_now(timezone);
		if time_gt(hour, minute, restart_time_hour, restart_time_minute) {
			break
		}
		sleep_1hour("too early to restart");
	}

	println!("time to restart");
	println!();

	// restart

	for service in services_to_restart {
		service_restart_if_running(service);
	}

	println!("unreachable code reached");
	log("reached end of function (which should have been impossible)".to_string());
	return ExitCode::FAILURE;

	// return ExitCode::SUCCESS;
}
