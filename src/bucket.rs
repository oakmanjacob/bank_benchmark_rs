use super::AccountId;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountError {
	DoesNotExist,
}

pub struct Accounts {
	buckets: Box<[RwLock<Bucket<usize, f64>>]>,
}

impl Accounts {
	pub fn new(buckets: usize) -> Self {
		let buckets: Vec<_> = (0..buckets).map(|_| RwLock::new(Bucket::new())).collect();
		
		Self {
			buckets: buckets.into_boxed_slice(),
		}
	}

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

	pub fn sum_up_all_accounts(&self) -> f64 {
		let guards: Vec<_> = self.buckets.iter().map(|bucket| bucket.read().unwrap()).collect();

		guards
			.into_iter()
			.map(|bucket| -> f64 {
				bucket.map.iter().map(|(_, balance)| balance).sum()
			})
			.sum()

	}

	pub fn add_account(&self, id: AccountId, balance: f64) {
		let mut bucket = self.buckets[id % self.buckets.len()].write().unwrap();
		assert!(bucket.insert(id, balance), "account already exists");
	}
}

pub struct Bucket<K, V> {
	map: Vec<(K, V)>
}

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