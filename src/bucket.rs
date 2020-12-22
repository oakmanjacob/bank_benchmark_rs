pub struct Bucket<K: std::cmp::Ord + std::marker::Copy, V: std::marker::Copy> {
	map: Vec<(K, V)>
}

impl<K: std::cmp::Ord + std::marker::Copy, V: std::marker::Copy> Bucket<K, V> {
	pub fn new() -> Bucket<K, V> {
		Bucket::<K, V> {
			map: Vec::<(K, V)>::new()
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

	pub fn iter(&self) -> std::slice::Iter<'_, (K, V)> {
		self.map.iter()
	}
}