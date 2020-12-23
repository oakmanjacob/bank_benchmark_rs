extern crate rand;
extern crate rand_pcg;

mod bucket;

use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use rand_pcg::Pcg64;

use std::sync::Arc;
use std::thread;
use std::time::Instant;

use bucket::Accounts;
use bucket::AccountsSeq;

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
	let command_count = 1024;
	let thread_count = 1;

	// Initialize accounts
	let accounts = Arc::new(Accounts::new(stripe_count));
	for id in 0..account_count {
		accounts.add_account(id, 100000.0 / account_count as f64);
	}

	// Generate random commands.
	// Commands will have a 5% change to be a balance command
	// and a 95% change to be a transfer

	let mut rng = Pcg64::seed_from_u64(0);
	let command_range = Uniform::from(0..20);
	let account_range = Uniform::from(0..account_count);
	let deposit_range = Uniform::from(0.0..100.0);

	let commands: Arc<[Command]> = (0..command_count)
		.map(|_| {
			if command_range.sample(&mut rng) == 0 {
				Command::CheckTotalBalance
			} else {
				Command::Transfer {
					from: account_range.sample(&mut rng),
					to: account_range.sample(&mut rng),
					amount: deposit_range.sample(&mut rng),
				}
			}
		})
		.collect();

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

	// Reclaim execution
	let con_time = threads
		.into_iter()
		.map(|handle| handle.join().unwrap())
		.max()
		.unwrap();

	// Initialize accounts sequential
	let mut account_seq = AccountsSeq::new(stripe_count);
	for id in 0..account_count {
		account_seq.add_account(id, 100000.0 / account_count as f64);
	}
	
	let start = Instant::now();

	for command in commands.iter() {
		match command {
			&Command::Transfer { from, to, amount } => {
				account_seq.transfer(from, to, amount).expect("account doesn't exist");
			},
			Command::CheckTotalBalance => {
				println!("Total Balance {}", account_seq.sum_up_all_accounts());
			}
		}
	}

	let seq_time = start.elapsed().as_micros();

	println!("Concurrent time {}", con_time);
	println!("Sequential time {}", seq_time);
	println!("Total balance: {}", accounts.sum_up_all_accounts());
}

/// Do work on a thread given an accounts object and
/// a list of commands to work through.
/// Commands are pregenerated to seperate out randomness
fn do_work(accounts: &Accounts, commands: &[Command]) -> u128 {
	let start = std::time::Instant::now();

	for command in commands {
		match command {
			&Command::Transfer { from, to, amount } => {
				accounts.transfer(from, to, amount).expect("account doesn't exist");
			},
			Command::CheckTotalBalance => {
				println!("Total Balance {}", accounts.sum_up_all_accounts());
			}
		}
	}

	start.elapsed().as_micros()
}