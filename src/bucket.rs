use super::AccountId;
use std::sync::RwLock;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountError {
	DoesNotExist,
}

pub struct Accounts {
	buckets: Box<[RwLock<Bucket<usize, f64>>]>,
}

impl Accounts {
	/// Constructor with takes a number of buckets which corresponds to the number of stripes.
	pub fn new(buckets: usize) -> Self {
		let buckets: Vec<_> = (0..buckets).map(|_| RwLock::new(Bucket::new())).collect();
		
		Self {
			buckets: buckets.into_boxed_slice(),
		}
	}

	/// Move an amount of money from one account to another atomically.
	/// There are not guarentees here that accounts will not go into negative balances.
	pub fn transfer(&self, from: AccountId, to: AccountId, amount: f64) -> Result<(), AccountError> {
		if to == from {
			return Ok(())
		}
		
		let from_bucket_idx = from % self.buckets.len();
		let to_bucket_idx = to % self.buckets.len();

		if from_bucket_idx == to_bucket_idx {
			// Same bucket.
			let mut bucket = self.buckets[from_bucket_idx].write().unwrap();

			let from_amount = bucket.get(from).ok_or(AccountError::DoesNotExist)?.1;
			let to_amount = bucket.get(to).ok_or(AccountError::DoesNotExist)?.1;
			assert!(bucket.update(from, from_amount - amount));
			assert!(bucket.update(to, to_amount + amount));
		} else {
			// Different buckets.
			// We have to be careful here, we must always lock in the same order to prevent deadlocks.
			let (mut from_bucket, mut to_bucket) = if from_bucket_idx < to_bucket_idx {
				let from_bucket = self.buckets[from_bucket_idx].write().unwrap();
				let to_bucket = self.buckets[to_bucket_idx].write().unwrap();
				(from_bucket, to_bucket)
			} else {
				let to_bucket = self.buckets[to_bucket_idx].write().unwrap();
				let from_bucket = self.buckets[from_bucket_idx].write().unwrap();
				(from_bucket, to_bucket)
			};

			let from_amount = from_bucket.get(from).ok_or(AccountError::DoesNotExist)?.1;
			let to_amount = to_bucket.get(to).ok_or(AccountError::DoesNotExist)?.1;
			assert!(from_bucket.update(from, from_amount - amount));
			assert!(to_bucket.update(to, to_amount + amount));
		}

		Ok(())
	}

	/// Calculate the total sum of all balances in each account
	pub fn sum_up_all_accounts(&self) -> f64 {
		let guards: Vec<_> = self.buckets.iter().map(|bucket| bucket.read().unwrap()).collect();

		guards
			.into_iter()
			.map(|bucket| -> f64 {
				bucket.map.iter().map(|(_, balance)| balance).sum()
			})
			.sum()
	}

	/// Add an account with a balance
	pub fn add_account(&self, id: AccountId, balance: f64) {
		let mut bucket = self.buckets[id % self.buckets.len()].write().unwrap();
		assert!(bucket.insert(id, balance), "account already exists");
	}
}

pub struct AccountsSeq {
	buckets: Box<[RefCell<Bucket<usize, f64>>]>,
}

impl AccountsSeq {
	/// Constructor with takes a number of buckets which corresponds to the number of stripes.
	pub fn new(buckets: usize) -> Self {
		let buckets: Vec<_> = (0..buckets).map(|_| RefCell::new(Bucket::new())).collect();
		
		Self {
			buckets: buckets.into_boxed_slice(),
		}
	}

	/// Move an amount of money from one account to another.
	/// There are not guarentees here that accounts will not go into negative balances.
	pub fn transfer(&self, from: AccountId, to: AccountId, amount: f64) -> Result<(), AccountError> {
		if to == from {
			return Ok(())
		}
		
		let from_bucket_idx = from % self.buckets.len();
		let to_bucket_idx = to % self.buckets.len();

		if from_bucket_idx == to_bucket_idx {
			// Same bucket.
			let mut bucket = self.buckets[from_bucket_idx].borrow_mut();

			let from_amount = bucket.get(from).ok_or(AccountError::DoesNotExist)?.1;
			let to_amount = bucket.get(to).ok_or(AccountError::DoesNotExist)?.1;
			assert!(bucket.update(from, from_amount - amount));
			assert!(bucket.update(to, to_amount + amount));
		} else {
			// Different buckets.
			let mut from_bucket = self.buckets[from_bucket_idx].borrow_mut();
			let mut to_bucket = self.buckets[to_bucket_idx].borrow_mut();

			let from_amount = from_bucket.get(from).ok_or(AccountError::DoesNotExist)?.1;
			let to_amount = to_bucket.get(to).ok_or(AccountError::DoesNotExist)?.1;
			assert!(from_bucket.update(from, from_amount - amount));
			assert!(to_bucket.update(to, to_amount + amount));
		}

		Ok(())
	}

	/// Calculate the total sum of all balances in each account
	pub fn sum_up_all_accounts(&self) -> f64 {
		self.buckets
			.into_iter()
			.map(|bucket| -> f64 {
				bucket.borrow().map.iter().map(|(_, balance)| balance).sum()
			})
			.sum()
	}

	/// Add an account with a balance
	pub fn add_account(&mut self, id: AccountId, balance: f64) {
		assert!(self.buckets[id % self.buckets.len()].borrow_mut().insert(id, balance), "account already exists");
	}
}

pub struct Bucket<K, V> {
	map: Vec<(K, V)>
}

/// A Bucket in this case is represented by a sorted vector
/// Insert and Remove are O(nlgn) while update and get are O(lgn)
impl<K: Ord + Copy, V: Copy> Bucket<K, V> {
	pub fn new() -> Self {
		Self {
			map: Vec::new()
		}
	}

	pub fn insert(&mut self, key: K, value: V) -> bool {
		match self.map.binary_search_by_key(&key, |(k,_)| *k) {
			Ok(_) => false,
			Err(index) => {
				self.map.insert(index, (key, value));
				true
			},
		}
	}

	pub fn update(&mut self, key: K, value: V) -> bool {
		match self.map.binary_search_by_key(&key, |(k,_)| *k) {
			Ok(index) => {
				self.map[index].1 = value;
				true
			},
			Err(_) => {
				false
			},
		}
	}

	// pub fn remove(&mut self, key: K) -> bool {
	// 	match self.map.binary_search_by_key(&key, |(k,_)| *k) {
	// 		Ok(index) => {
	// 			self.map.remove(index);
	// 			true
	// 		},
	// 		Err(_) => {
	// 			false
	// 		},
	// 	}
	// }

	pub fn get(&self, key: K) -> Option<&(K, V)> {
		match self.map.binary_search_by_key(&key, |(k,_)| *k) {
			Ok(index) => {
				Some(&self.map[index])
			},
			Err(_) => {
				None
			},
		}
	}
}