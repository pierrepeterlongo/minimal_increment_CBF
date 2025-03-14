use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct IncOnlyMinCbf{
    bit_array: Vec<u8>,  // Stores all counters in a packed format
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
        let bits_total = size * bits_per_counter as usize;
        let byte_size = (bits_total + 7) / 8; // Round up to nearest byte

        IncOnlyMinCbf {
            bit_array: vec![0; byte_size],
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
        let bit_position = index * self.bits_per_counter as usize;
        let byte_index = bit_position / 8;
        let bit_offset = bit_position % 8;

        let mut value: u32 = 0;
        for i in 0..self.bits_per_counter {
            let bit = ((self.bit_array[byte_index + ((bit_offset as usize + i as usize) / 8)] >> ((bit_offset as usize + i as usize) % 8)) & 1) as u32;
            value |= bit << i;
        }
        value
    }

    /// check if a counter value is zero
    fn is_zero(&self, index: usize) -> bool {
        let bit_position = index * self.bits_per_counter as usize;
        let byte_index = bit_position / 8;
        let bit_offset = bit_position % 8;

        for i in 0..self.bits_per_counter {
            let bit = ((self.bit_array[byte_index + ((bit_offset as usize + i as usize) / 8)] >> ((bit_offset as usize + i as usize) % 8)) & 1) as u32;
            if bit != 0 {
                return false;
            }
        }
        true
    }

    /// increment a counter value in the bit array
    fn increment_counter(&mut self, index: usize) {
        let bit_position = index * self.bits_per_counter as usize;
        let byte_index = bit_position / 8;
        let bit_offset = bit_position % 8;

        for i in 0..self.bits_per_counter {
            let byte_pos = byte_index + (bit_offset as usize + i as usize) / 8;
            let bit_pos = ((bit_offset as usize) + i as usize) % 8;
            let bit_val = ((self.bit_array[byte_pos] >> bit_pos) & 1) as u8;
            if bit_val == 0 {
                self.bit_array[byte_pos] |= 1 << bit_pos; // Set bit
                break;
            }
        }
    }

    /// Add an item to the filter, incrementing only the smallest counters
    pub fn add<T: Hash>(&mut self, item: &T) {
        let mut min_value = self.max_value;
        let mut min_hashes = Vec::new();
        let indices = self.hash_indices(item);
        for &index in indices.iter() {
            let current_value = self.get_counter(index);
            if current_value < min_value {
                min_value = current_value;
                min_hashes = vec![index];
            } else if current_value == min_value {
                min_hashes.push(index);
            }
        }
        
        for &index in min_hashes.iter() {
            self.increment_counter(index);
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

