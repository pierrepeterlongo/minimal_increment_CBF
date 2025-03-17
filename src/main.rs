//! Increment only minimum counting bloom filter
use std::hash;
use std::collections::HashMap;

/* std use */
use rand::Rng;

/* project use */
use iomcbf::IncOnlyMinCbf;

///////////////////////// MAIN /////////////////////////

fn generate_random_kmer(k: usize) -> String {
    let mut rng = rand::rng();
    let mut kmer = String::new();
    for _ in 0..k {
        let base = match rng.random_range(0..=3) {
            0 => 'A',
            1 => 'C',
            2 => 'G',
            _ => 'T',
        };
        kmer.push(base);
    }
    kmer
}

fn check_fp_rate(x: u8, num_hashes: usize, size: usize, n_elements: usize) {
    let mut iomcbf = IncOnlyMinCbf::new(size, num_hashes,x); // 1000 counters, 4 hashes, x-bit counters
    
    let mut rng = rand::rng();

    // generate an array of random kmers. 
    // each kmer appears between 1 and 10 times
    let k = 31; // kmer length
    let mut kmers = vec![];
    for _ in 0..n_elements {
        let kmer = generate_random_kmer(k);
        let count = rng.random_range(1..=10);
        for _ in 0..count {
            kmers.push(kmer.clone());
        }
    }
    // add all kmers to the iomcbf
    for kmer in kmers.iter() {
        iomcbf.add(kmer);
    }

    // Generate one million random kmers.
    // Check how many of them are false positives.
    let mut false_positives = 0;
    for _ in 0..1_000_000 {
        let kmer = generate_random_kmer(k);
        if iomcbf.contains(&kmer) && !kmers.contains(&kmer) {
            false_positives += 1;
        }
    }
    println!("False positive rate: {}", false_positives as f64 / 1_000_000.0);
}

fn print_stats(overestimations: Vec<i32>) {
    let sum: i32 = overestimations.iter().sum();
    let mean = sum as f64 / overestimations.len() as f64;
    let median = {
        let mut sorted = overestimations.clone();
        sorted.sort();
        let mid = sorted.len() / 2;
        if sorted.len() % 2 == 0 {
            (sorted[mid - 1] + sorted[mid]) as f64 / 2.0
        } else {
            sorted[mid] as f64
        }
    };
    let variance = {
        let mean = mean as f64;
        let sum = overestimations.iter().map(|x| (*x as f64 - mean).powi(2)).sum::<f64>();
        sum / overestimations.len() as f64
    };
    let std_dev = variance.sqrt();
    println!("Overestimation sum: {}", sum);
    println!("Overestimation mean: {}", mean);
    println!("Overestimation median: {}", median);
    println!("Overestimation standard deviation: {}", std_dev);
}


fn check_overestimation_rate(x: u8, num_hashes: usize, size: usize, n_elements: usize) {
    println!("Creating IncOnlyMinCbf with size = {}, num_hashes = {}, x = {}", size, num_hashes, x);
    let mut iomcbf = IncOnlyMinCbf::new(size, num_hashes,x); // 1000 counters, 4 hashes, x-bit counters
    
    let mut rng = rand::rng();

    // generate a dictionary of random kmers with counts
    // each kmer appears between 1 and 10 times
    let k = 31;
    println!("Generating kmers...");
    let mut kmers = HashMap::new();
    for _ in 0..n_elements {
        let kmer = generate_random_kmer(k);
        let count = rng.random_range(1..=10);
        kmers.insert(kmer, count);
    }
    println!("Adding kmers to the IncOnlyMinCbf with optimization...");
    // add all kmers to the iomcbf, measuring the time
    let start = std::time::Instant::now();
    for (kmer, count) in kmers.iter() {
        for _ in 0..*count {
            iomcbf.add(kmer);
        }
    }
    let duration = start.elapsed();
    println!("Insertion time, with the optimization: {:?}",
        duration);
    

    // for each element in the iomcbf, store its overestimation
    // provide the sum, mean, median, and standard deviation of the overestimations
    println!("Querying {} kmers from the IncOnlyMinCbf...", kmers.len());
    let start: std::time::Instant = std::time::Instant::now();
    let mut overestimations = vec![];
    for (kmer, true_count) in kmers.iter() {
        let count = iomcbf.count(kmer);
        let overestimation = count as i32 - *true_count as i32;
        overestimations.push(overestimation);
    }
    let duration: std::time::Duration = start.elapsed();
    println!("Query time: {:?}", duration);

    println!("Overestimation rate, optimized:");
    print_stats(overestimations);

    // Make the same test without the optimization of incrementing only the smallest counters
    let mut cbf = IncOnlyMinCbf::new(size, num_hashes,x); 

    println!("Adding kmers to the IncOnlyMinCbf without optimization...");
    // add all kmers to the cbf, measuring the time
    let start = std::time::Instant::now();
    for (kmer, count) in kmers.iter() {
        for _ in 0..*count {
            cbf.add_all(kmer);
        }
    }
    let duration = start.elapsed();
    println!("Insertion time, without the optimization: {:?}",
        duration);

    // for each element in the iomcbf, store its overestimation
    // provide the sum, mean, median, and standard deviation of the overestimations
    println!("Querying {} kmers from the IncOnlyMinCbf...", kmers.len());
    let start = std::time::Instant::now();
    let mut overestimations_non_optimized = vec![];
    for (kmer, true_count) in kmers.iter() {
        let count = cbf.count(kmer);
        let overestimation = count as i32 - *true_count as i32;
        overestimations_non_optimized.push(overestimation);
    }
    let duration = start.elapsed();
    println!("Query time: {:?}", duration);

    println!("Overestimation rate, non optimized:");
    print_stats(overestimations_non_optimized);

}

fn check_no_fn_no_underestimation(x: u8, num_hashes: usize, size: usize, n_elements: usize) {
    let mut iomcbf = IncOnlyMinCbf::new(size, num_hashes,x); 
    
    let mut rng = rand::rng();

    println!("Checking with size = {}, num_hashes = {}, x = {}, and {} elements", size, num_hashes, x, n_elements);
    // generate an array of random kmers. 
    // each kmer appears between 1 and 10 times
    let k = 31; // kmer length
    let mut kmers = vec![];
    for _ in 0..n_elements {
        let kmer = generate_random_kmer(k);
        let count = rng.random_range(1..=10);
        for _ in 0..count {
            kmers.push(kmer.clone());
        }
    }

    // add all kmers to the iomcbf, measuring the time
    let start = std::time::Instant::now();
    for kmer in kmers.iter() {
        iomcbf.add(kmer);
    }
    let duration = start.elapsed();
    println!("Insertion time: {:?}", duration);

    // check that all kmers are present
    let start = std::time::Instant::now();
    for kmer in kmers.iter() {
        assert!(iomcbf.contains(kmer));
    }
    let duration = start.elapsed();
    println!("Query time: {:?}", duration);

    // check that the counts are not underestimated
    let start = std::time::Instant::now();
    for kmer in kmers.iter() {
        let count = iomcbf.count(kmer);
        let true_count = kmers.iter().filter(|&x| x == kmer).count();
        assert!(count >= true_count as u32);
    }
    let duration = start.elapsed();
    println!("Count time: {:?}", duration);
    println!("No false negatives and no underestimations found");
}


fn main() {

    let x = 4; // Number of bits per counter (max value = 2^4 - 1 = 15)
    let num_hashes = 7; // Number of hash functions
    let size = 50_000_000; // Number of counters
    let n_elements = 50; // Number of elements to insert
    println!("Checking there are no FN and no underestimations...");
    check_no_fn_no_underestimation(x, num_hashes, size, n_elements);

    // println!("Checking false positive rate...");
    // check_fp_rate(x, num_hashes, size, n_elements);

    // println!("Checking overestimation rate...");
    // check_overestimation_rate(x, num_hashes, size, n_elements);

    // test the overestimation rate with different values of n_elements
    // test values from 1 to 50_000_001 in steps of 1_000_000
    for n_elements in (0..=100).map(|x| x * 1_000_000) {
        println!("\n Checking overestimation rate with n_elements = {}...", n_elements);
        check_overestimation_rate(x, num_hashes, size, n_elements +1);
    }
    println!("All tests passed!");
}
