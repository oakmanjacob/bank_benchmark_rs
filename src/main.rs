extern crate rand;
extern crate rand_pcg;

use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use rand_pcg::Pcg64;

mod bucket;

use bucket::Accounts;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

type AccountId = usize;

enum Command {
	Transfer {
		from: AccountId,
		to: AccountId,
		amount: f64,
		},
		CheckTotalBalance,
}

fn main() {
	// Configure parameters
	let stripe_count = 256;
	let account_count = 1024;
	let command_count = 100000;
	let thread_count = 1;

	let accounts = Arc::new(Accounts::new(stripe_count));

	for id in 0..account_count {
		accounts.add_account(id, 100000.0 / account_count as f64)
	}

	// Generate random commands.
	let mut rng = Pcg64::seed_from_u64(0);
	let command_range = Uniform::from(0..20);
		let account_range = Uniform::from(0..account_count);
		
		// Calculate range as integer so it can be converted to 2 decimal places
	let deposit_range = Uniform::from(0..10000);

	let commands: Arc<[Command]> = (0..command_count)
		.map(|_| {
			if command_range.sample(&mut rng) == 0 {
				Command::CheckTotalBalance
			} else {
				Command::Transfer {
					from: account_range.sample(&mut rng),
					to: account_range.sample(&mut rng),
					amount: deposit_range.sample(&mut rng) as f64 / 100.0,
				}
			}
		})
		.collect();

	let start = Instant::now();
	let mut threads = vec![];

	let mut begin = 0;
	let mut end = command_count / thread_count;
	let mut spare = command_count % thread_count;

	for _ in 0..thread_count {
		// Distribute remainder evenly
		if spare > 0 {
			end += 1;
			spare -= 1;
		}

		let accounts = Arc::clone(&accounts);
		let commands = Arc::clone(&commands);

		threads.push(thread::spawn(move || {
					do_work(&accounts, &commands[begin..end])
		}));

		// Shift slice window
		begin = end;
		end += command_count / thread_count;
	}

	let max_time = threads
		.into_iter()
		.map(|handle| handle.join().unwrap())
		.max()
		.unwrap();

	println!("Max time {}", max_time);
	println!("Total {}", start.elapsed().as_micros());
}

fn do_work(accounts: &Accounts, commands: &[Command]) -> u128 {
	let start = std::time::Instant::now();

	for command in commands {
		match command {
			&Command::Transfer { from, to, amount } => {
				// println!("starting transfer");
				accounts.transfer(from, to, amount).expect("account doesn't exist");
				// println!("ending transfer");
			},
			Command::CheckTotalBalance => {
				println!("Total Balance {}", accounts.sum_up_all_accounts());
			}
		}
	}

	start.elapsed().as_micros()
}