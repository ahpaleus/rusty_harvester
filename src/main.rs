mod rng;

use std::error::Error;
use std::fs::File;
use std::fs;
use std::collections::{BTreeSet, HashSet};
use std::io::{Read, Write, BufReader, BufRead};
use std::process::{Command, Stdio};
use rng::Rng;
use std::env;
use std::time::Instant;
use std::path::Path;

fn read_file_to_vec(file_to_open: String) -> Vec<u8> {
	// Opening a corpus file
    let mut file_handler = File::open(file_to_open).unwrap();
	
	// Vector for bytes
	let mut data = Vec::new();

	// Moving input (corpus file) to Vectors
	let _ = file_handler.read_to_end(&mut data).unwrap();

	// Return vector
	return data
}

fn fuzz(mut data_to_fuzz: Vec<u8>) -> Vec<u8> {
	// Create a new rng
	let rng = Rng::new();
	
	// Setting up fuzz_factor (percentage of mutated bytes)
	let fuzz_factor = 0.03;

	// Amount of bytes to fuzz bitflips
	let mut amount_of_byte = (data_to_fuzz.len() as f64 * fuzz_factor as f64) as usize;

	println!("-- DEBUG -- amount_of_byte1: {}", amount_of_byte);

	if amount_of_byte > 0 {
		amount_of_byte = (rng.rand() % amount_of_byte) as usize;
	} else {
		amount_of_byte = 1;
	}

	// Running through the bytes amount_of_byte many times
	for _ in 0..amount_of_byte {
		let random_byte = rng.rand() % data_to_fuzz.len();

		// Bitflipping mutation
		let values = vec![1, 2, 4, 8, 16, 32, 64, 128];
		data_to_fuzz[random_byte] = values[rng.rand() % 8];
	}

	return data_to_fuzz
}

fn read_lines<P>(filename: P) 
	-> std::io::Result<std::io::Lines<BufReader<File>>>
	where P: AsRef<Path>, {
		let file = File::open(filename)?;
		
		return Ok(BufReader::new(file).lines())
}

fn main() -> Result<(), Box<dyn Error>> {
	let rng = Rng::new();

	let args: Vec<String> = env::args().collect();

	if args.len() != 3 {
		println!("Usage: ./fuzzer <input file> <binary>");
		return Ok(());
	}

	// Parse arguments
	let filename = &args[1];
	let binary = &args[2];

	println!("Filename: {}", filename);

	// Create directory for crashes, before - deletes
	let _crashes = match fs::remove_dir_all("crashes") {
	Ok(crashes) => crashes,
		_ => println!("crashes dir does not exist, I am creating.")
	};
	fs::create_dir_all("crashes")?;

	// Opening up a corpus file
	let corpus_handler = &read_file_to_vec(filename.to_string());

	println!("Length of corpus file: {} bytes", corpus_handler.len());

	// Count fuzz_cases & crashes for a statistical reason
	let mut fuzz_cases 	= 0;
	let mut crashes		= 0;

	// Unique crashes
	let mut unique_crashes = HashSet::new();

	// Store current time
	let it = Instant::now();

	// Store Intel PIN variables
	let intelpin_path = "pin";

	// Create resource for coverage
	let mut coverage = BTreeSet::new();
	
	// Create directory for queue mutated files
	let queue_dir = "queue";
	
	let _remove = match fs::remove_dir_all(queue_dir) {
		Ok(remove) => remove,
		_ => println!("{} dir does not exist, I am creating.", queue_dir)
	};

	// Hashset with filenames which produces new coverage
	let mut files_new_coverage = HashSet::new();

	let mut vector_with_coveragefiles = Vec::new();

	// Create directory queue
	fs::create_dir_all("queue")?;

	// Copy mutated0 to queue
	fs::copy(filename.to_string(), queue_dir.to_owned()+"/id0")?;

	// Insert id0 (basic corpus file) to hashset
	files_new_coverage.insert("id0".to_string());
	vector_with_coveragefiles.push("id0".to_string());

	// Infinite loop -> TODO: limit to number_of_iterations
	loop {
		// Create file to mutate
		let mut mutated_handler = File::create("mutated")?;

		// Random file to use and mutate
		let random_file = vector_with_coveragefiles[rng.rand() 
			% vector_with_coveragefiles.len()].clone();

		// println!("random_file: {}", random_file);

		let mutated_vector = fuzz(
			read_file_to_vec
			("queue/".to_owned()+&random_file.to_string()));

		// Write to created file
		mutated_handler.write(&mutated_vector)?;

		// Executing binary under Intel PIN
		let process = Command::new(intelpin_path)
			.arg("-t")
			.arg("intelpin-module/coverage.dylib")
			.arg("--")
			.arg(binary)
			.arg("mutated")
			.stdout(Stdio::null())
			.stderr(Stdio::null())
			.spawn()
			.expect("Failed to execute child under Intel PIN");

		let ecode = process.wait_with_output().expect("failed to wait on child");

		// Fuzz cases per second information
		let elapsed = (Instant::now() - it).as_secs_f64();

		print!("Fuzz case\t{}\t|", fuzz_cases);
		print!("{:12.2} fuzz cases/second", fuzz_cases as f64 / elapsed);
		print!("\t|  {} crash ({} unique)", crashes, unique_crashes.len());
		print!("\t|  {} coverage\n", coverage.len());

		fuzz_cases += 1;

		// Checking exit status to grab segmenation faults
		if ecode.status.success() {
			// println!("Run correctly! (Return status 0)\t");
			()

		} else {
			// On Unix, this will return None if the process was terminated 
			// by a signal; std::os::unix provides an extension trait for 
			// extracting the signal and other details from the ExitStatus.
			// `man signal`:
			// 11    SIGSEGV      create core image    segmentation violation
			match ecode.status.code() {
				// Catch segmentation fault (signal 11)
				Some(139) | Some(11) => {
					// Obtain crash's PC (first line from interceptsegv.out)
					let intelpin_file = File::open("interceptsegv.out")?;
					let mut intelpin_file_buf = BufReader::new(intelpin_file);
					let mut byte_vec: Vec<u8> = Vec::new();
					let _my_bytes = intelpin_file_buf.read_until(b'\n', &mut byte_vec);

					let pc_address = std::str::from_utf8(&byte_vec).unwrap().trim_matches('\n');
					
					// Save crash
					if !unique_crashes.contains(pc_address){
						fs::copy("mutated", "crashes/SIGSEGV_PC_".to_owned() + pc_address)?;
					}

					unique_crashes.insert(pc_address.to_string());
					crashes += 1;

					()
				}

				// Catch any other signal
				_ => {
					// println!("\tOther signal.., {:?}", ecode.status.signal());
					()
				}
			}
		}

		let coverage_before = coverage.len();

		// Coverage module
		if let Ok(lines) = read_lines("./interceptsegv.out") {
			for line in lines {
				if let Ok(ip) = line {
					// avoid first line while crash without <-
					if ip.contains(" <- ") {
						let edge: Vec<usize> = ip.split(" <- ")
							.map(|x| x.parse::<usize>().unwrap()).collect();
						let new_path = edge[0] ^ edge[1];

						if !coverage.contains(&new_path) {
							coverage.insert(new_path);
						}
					}
				}
			}
		}

		if coverage_before < coverage.len() {
			// Copy file which creates new coverage to queue dir
			let new_file = "id".to_owned() + &fuzz_cases.to_string();

			fs::copy("mutated", queue_dir.to_string() + "/"
				+ &new_file)?;

			// Add name of the file to the hashset with files which produces
			// new coverage
			files_new_coverage.insert(new_file.to_string());
			vector_with_coveragefiles.push(new_file.to_string());		
		}
	}
}


/*
TODO:
- Other mutations
- Threads, improving performance
- GNU plots for fun
- Input from stdin
- Argument parsing, checking pin existence in $PATH
- Timeout of executing binary
*/


