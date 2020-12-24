mod bucket;

use rand::distributions::{Distribution, Uniform};
use rand::prelude::*;
use rand_pcg::Pcg64;

use std::sync::Arc;
use std::thread;
use std::time::Instant;

use bucket::Accounts;
use bucket::AccountsSeq;

use structopt::StructOpt;

type AccountId = usize;

enum Command {
	Transfer {
		from: AccountId,
		to: AccountId,
		amount: f64,
	},
	CheckTotalBalance,
}

#[derive(StructOpt)]
struct Cli {
	#[structopt(short = "a", long = "account_count", default_value = "1024")]
	account_count: usize,

	#[structopt(short = "s", long = "stripe_count", default_value = "256")]
	stripe_count: usize,

	#[structopt(short = "c", long = "command_count", default_value = "1024")]
	command_count: usize,

	#[structopt(short = "t", long = "thread_count", default_value = "1")]
	thread_count: usize,
}

fn main() {
	let args = Cli::from_args();

	// Generate random commands.
	// Commands will have a 5% change to be a balance command
	// and a 95% change to be a transfer

	let mut rng = Pcg64::seed_from_u64(0);
	let command_range = Uniform::from(0..20);
	let account_range = Uniform::from(0..args.account_count);
	let deposit_range = Uniform::from(0.0..100.0);

	let commands: Arc<[Command]> = (0..args.command_count)
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

	let exec_time;

	if args.thread_count > 1 {
		// Initialize accounts
		let accounts = Arc::new(Accounts::new(args.stripe_count));
		for id in 0..args.account_count {
			accounts.add_account(id, 100000.0 / args.account_count as f64);
		}

		let mut threads = vec![];

		let mut begin = 0;
		let mut end = args.command_count / args.thread_count;
		let mut spare = args.command_count % args.thread_count;

		for _ in 0..args.thread_count {
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
			end += args.command_count / args.thread_count;
		}

		// Reclaim execution
		exec_time = threads
			.into_iter()
			.map(|handle| handle.join().unwrap())
			.max()
			.unwrap();

		println!("Total balance: {}", accounts.sum_up_all_accounts());
	}
	else {
		// Initialize accounts sequential
		let mut account_seq = AccountsSeq::new(args.stripe_count);
		for id in 0..args.account_count {
			account_seq.add_account(id, 100000.0 / args.account_count as f64);
		}
		
		let start = Instant::now();

		for command in commands.iter() {
			match command {
				&Command::Transfer { from, to, amount } => {
					account_seq.transfer(from, to, amount).expect("account doesn't exist");
				},
				Command::CheckTotalBalance => {
					println!("Total balance: {}", account_seq.sum_up_all_accounts());
				}
			}
		}

		exec_time = start.elapsed().as_micros();
		println!("Total balance: {}", account_seq.sum_up_all_accounts());
	}

	println!("Execution time {}", exec_time);
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
				println!("Total balance: {}", accounts.sum_up_all_accounts());
			}
		}
	}

	start.elapsed().as_micros()
}