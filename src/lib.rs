use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bitvec::prelude::*;


pub struct IncOnlyMinCbf{
    bb: BitBox,  // Stores all counters in a packed format
    size: usize,         // Total number of counters
    num_hashes: usize,   // Number of hash functions
    bits_per_counter: u8, // Number of bits per counter
    max_value: u32,      // Maximum value each counter can hold (2^x - 1)
}

impl IncOnlyMinCbf {
    /// Create a new Increment-Only Minimum Counting Bloom Filter with `x`-bit counters
    pub fn new(size: usize, num_hashes: usize, bits_per_counter: u8) -> Self {
        assert!(bits_per_counter > 0 && bits_per_counter <= 8, "bits_per_counter must be between 1 and 8");

        let max_value = (1 << bits_per_counter) - 1; // 2^x - 1


        IncOnlyMinCbf {
            bb: bitbox![0; size * bits_per_counter as usize],
            size,
            num_hashes,
            bits_per_counter,
            max_value,
        }
    }

    /// Generate multiple hash values for an item
    fn hash_indices<T: Hash>(&self, item: &T) -> Vec<usize> {
        let mut indices = Vec::new();
        for i in 0..self.num_hashes {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher); // Differentiate hash functions using i
            item.hash(&mut hasher);
            indices.push((hasher.finish() as usize) % self.size);
        }
        indices
    }

    /// Read a counter value from the bit array
    fn get_counter(&self, index: usize) -> u32 {
        let first_bit = index * self.bits_per_counter as usize;
        let last_bit = first_bit + self.bits_per_counter as usize;
        let mut value: u32 = 0;
        let mut pos = 0;
        for i in first_bit..last_bit {
            if self.bb[i] == true {
                value += 1 << pos;
            }
            pos += 1;
        }
        value
    }

    /// check if a counter value is zero
    fn is_zero(&self, index: usize) -> bool {
        // returns true if all bits from index * self.bits_per_counter as usize to (index + 1) * self.bits_per_counter as usize are zero
        let first_bit = index * self.bits_per_counter as usize;
        let last_bit = first_bit + self.bits_per_counter as usize;
        for i in first_bit..last_bit {
            if self.bb[i] {
                return false;
            }
        }
        true
    }


    /// increment a counter value in the bit array
    fn increment_counter(&mut self, index: usize) {
        let first_bit = index * self.bits_per_counter as usize;
        let last_bit = first_bit + self.bits_per_counter as usize;

        for i in first_bit..last_bit {
            if self.bb[i] {
                self.bb.set(i, false);
            } else {
                self.bb.set(i, true);
                return;
            }
        }
        // if we are here, it means all bits were set to 1, and we modified them to zero, 
        // so we need to reset them to 1, max value
        for i in first_bit..last_bit {
            self.bb.set(i, true);
        }
    }


    /// Add an item to the filter, incrementing only the smallest counters
    pub fn add<T: Hash>(&mut self, item: &T) {
        let mut min_value = self.max_value;
        let indices = self.hash_indices(item);
        let mut values = Vec::new();
        for &index in indices.iter() {
            let current_value = self.get_counter(index);
            values.push(current_value);
            if current_value < min_value {
                min_value = current_value;
            } 
        }
        // find all counters with the minimum value, and increment them
        for (i, &value) in values.iter().enumerate() {
            if value == min_value {
                self.increment_counter(indices[i]);
            }
        }
    }

    /// Add an item to the filter, incrementing all counters, not just the smallest ones
    /// This is useful for testing purposes
    pub fn add_all<T: Hash>(&mut self, item: &T) {
        let indices = self.hash_indices(item);
        for &index in indices.iter() {
            self.increment_counter(index);
        }
    }

    /// Check if an item is possibly in the filter (all hashed counters must be > 0)
    pub fn contains<T: Hash>(&self, item: &T) -> bool {
        for &i in self.hash_indices(item).iter() {
            if self.is_zero(i) {
                return false;
            }
        }
        true
    }


    /// Get the counter value for an item
    pub fn count<T: Hash>(&self, item: &T) -> u32 {
        let mut min_value = self.max_value;
        for &i in self.hash_indices(item).iter() {
            let value = self.get_counter(i);
            if value == 0 {
                return 0;
            }
            if value < min_value {
                min_value = value;
            }
        }
        min_value
    }

}

