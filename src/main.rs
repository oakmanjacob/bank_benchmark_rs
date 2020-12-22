extern crate rand;
use rand::distributions::{Distribution, Uniform};

mod bucket;

use bucket::Bucket;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::Instant;

fn main() {
	// Configure parameters
	let stripe_count: u32 = 256;
	let account_count: u32 = 256;
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
	for _ in 0..thread_count {
		let locks = Arc::clone(&locks);
		let commands = Arc::clone(&commands);

		threads.push(thread::spawn(move || {
			do_work(&locks, &commands)
		}));
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

fn do_work(locks: &Arc<Vec<RwLock<Bucket<i32,f64>>>>, commands: &Arc<Vec<(bool, i32, i32, f64)>>) -> u128 {
	let start = std::time::Instant::now();

	for command in commands.iter() {
		if command.0 {
			// Check total balance
			let mut sum = 0.0;
			let mut buckets = Vec::new();
			for lock in locks.iter() {
				buckets.push(lock.read().unwrap());
			}

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
			let mut from_bucket;
			let mut to_bucket;

			// Forced order of aquisition to avoid deadlock
			if command.1 as usize % locks.len() < command.2 as usize % locks.len() {
				from_bucket = locks[command.1 as usize % locks.len()].write().unwrap();
				to_bucket = locks[command.2 as usize % locks.len()].write().unwrap();
			}
			else if command.1 as usize % locks.len() > command.2 as usize % locks.len() {
				to_bucket = locks[command.2 as usize % locks.len()].write().unwrap();
				from_bucket = locks[command.1 as usize % locks.len()].write().unwrap();
			}
			else if command.1 == command.2 {
				println!("Same Account Issue");
				continue;
			}
			else {
				println!("Same Bucket Issue");
				continue;
			}

			// Get current balances and double check accounts actually exist
			let from_account_balance;
			match from_bucket.get(command.1) {
				Some(&(_,balance)) => from_account_balance = balance,
				None => continue
			}

			let to_account_balance;
			match to_bucket.get(command.2) {
				Some(&(_,balance)) => to_account_balance = balance,
				None => continue
			}

			// Make the transfer
			from_bucket.update(command.1, from_account_balance - command.3);
			to_bucket.update(command.2, to_account_balance + command.3);
		}
	}

	start.elapsed().as_micros()
}