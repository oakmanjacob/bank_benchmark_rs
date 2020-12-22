extern crate rand;
use rand::distributions::{Distribution, Uniform};

mod bucket;

use bucket::Bucket;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Instant;


fn main() {
	// Configure parameters
	let stripe_count: u32 = 256;
	let account_count: u32 = 1024;
	let command_count: u32 = 1024;
	let thread_count: u32 = 4;

	// Set up locks with contained buckets
	let mut locks = Vec::new();
	for i in 0..stripe_count {
		locks.push(RwLock::new(Bucket::<i32, f64>::new()));
		let mut bucket = locks.last().unwrap().write().unwrap();

		for j in (i..account_count).step_by(stripe_count as usize) {
			bucket.insert(j as i32, 100000.0 / account_count as f64);
		}
	}
	
	// Pregenerate randomness
	let mut commands = Vec::new();
	let mut rng = rand::thread_rng();
	let command_range = Uniform::from(0..20);
	let account_range = Uniform::from(0..account_count);
	let deposit_range = Uniform::from(0..10000);
	
	for _ in 0..command_count {
		commands.push((
			command_range.sample(&mut rng) == 0,
			account_range.sample(&mut rng) as i32,
			account_range.sample(&mut rng) as i32,
			deposit_range.sample(&mut rng) as f64 / 10.00));
	}

	// Set up values as arcs
	let locks = Arc::new(locks);
	let commands = Arc::new(commands);

	// Start timer
	let start = Instant::now();
	let mut threads = Vec::new();

	let mut begin = 0 as usize;
	let mut end = (command_count/thread_count) as usize;
	let mut spare = (command_count % thread_count) as usize;

	for _ in 0..thread_count {
		let locks = Arc::clone(&locks);
		let commands = Arc::clone(&commands);

		// Distribute remainder evenly
		if spare > 0 {
			end += 1;
			spare -= 1;
		}

		threads.push(thread::spawn(move || {
			do_work(&locks, &commands[begin..end])
		}));

		// Shift slice window
		begin = end;
		end += (command_count/thread_count) as usize;
	}

	// Reclaim Threads and Find Max Time Used
	let mut max = 0;
	while threads.len() > 0 {
		match threads.pop() {
			Some(thread) => {
				let val = thread.join().unwrap();
				if val > max {
					max = val;
				}
			},
			None => {},
		}
	}

	println!("Max {}", max);
	println!("Total {}", start.elapsed().as_micros());
}

fn do_work(locks: &Arc<Vec<RwLock<Bucket<i32,f64>>>>, commands: &[(bool, i32, i32, f64)]) -> u128 {
	let start = std::time::Instant::now();

	for command in commands.iter() {
		if command.0 {
			// Check total balance
			let mut sum = 0.0;
			let mut buckets = Vec::new();

			// Aquire locks pessimistically
			for lock in locks.iter() {
				buckets.push(lock.read().unwrap());
			}

			// Aquire locks optimistically
			// let mut locked = false;
			// while !locked {
			// 	locked = true;

			// 	while buckets.len() > 0 {
			// 		buckets.pop();
			// 	}

			// 	for lock in locks.iter() {
			// 		match lock.try_read() {
			// 			Ok(bucket) => buckets.push(bucket),
			// 			Err(_) => {
			// 				locked = false;
			// 				break;
			// 			}
			// 		}
			// 	}
			// }

			while buckets.len() > 0 {
				let bucket = buckets.pop().unwrap();
				for account in bucket.iter() {
					sum += account.1;
				}
			}

			println!("Total Balance {}", sum);
		}
		else {
			// Transfer
			let from_bucket;
			let to_bucket;

			let from_bucket_id = command.1 as usize % locks.len();
			let to_bucket_id = command.2 as usize % locks.len();

			// Forced order of aquisition to avoid deadlock
			if from_bucket_id != to_bucket_id {
				if from_bucket_id < to_bucket_id {
					from_bucket = Rc::new(RefCell::new(locks[from_bucket_id].write().unwrap()));
					to_bucket = Rc::new(RefCell::new(locks[to_bucket_id].write().unwrap()));
				}
				else {
					to_bucket = Rc::new(RefCell::new(locks[to_bucket_id].write().unwrap()));
					from_bucket = Rc::new(RefCell::new(locks[from_bucket_id].write().unwrap()));
				}
			}
			else {
				if command.1 == command.2 {
					println!("Same Account");
					continue;
				}

				from_bucket = Rc::new(RefCell::new(locks[from_bucket_id].write().unwrap()));
				to_bucket = Rc::clone(&from_bucket);
			}

			// Get current balances and double check accounts actually exist
			let from_account_balance;
			match from_bucket.borrow().get(command.1) {
				Some(&(_,balance)) => from_account_balance = balance,
				None => continue
			}

			let to_account_balance;
			match to_bucket.borrow().get(command.2) {
				Some(&(_,balance)) => to_account_balance = balance,
				None => continue
			}

			// Make the transfer
			from_bucket.borrow_mut().update(command.1, from_account_balance - command.3);
			to_bucket.borrow_mut().update(command.2, to_account_balance + command.3);
		}
	}

	start.elapsed().as_micros()
}