use std::cell::Cell;

pub struct Rng {
	// Internal xorshift seed
	seed: Cell<u64>,
}

impl Rng {
	pub fn new() -> Self {
		let ret = Rng {
			seed: Cell::new(unsafe { core::arch::x86_64::_rdtsc() }),
		};

		for _ in 0..1000 {
			let _ = ret.rand();
		}

		ret
	}

	// Created a RNG with a fixed `seed` value
	// pub fn seeded(seed: u64) -> Self {
	// 	Rng {
	// 		seed: Cell::new(seed),
	// 	}
	// }

	pub fn rand(&self) -> usize {
		let mut seed = self.seed.get();
		seed ^= seed << 13;
		seed ^= seed >> 17;
		seed ^= seed << 43;
		self.seed.set(seed);

		seed as usize
	}


}

