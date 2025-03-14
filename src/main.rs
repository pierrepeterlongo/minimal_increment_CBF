//! Increment only minimum counting bloom filter
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

    // generate an array of random kmers. 
    // each kmer appears between 1 and 10 times
    let k = 31; // kmer length
    let mut kmers = vec![];
    println!("Generating kmers...");
    for _ in 0..n_elements {
        let kmer = generate_random_kmer(k);
        let count = rng.random_range(1..=10);
        for _ in 0..count {
            kmers.push(kmer.clone());
        }
    }
    println!("Adding kmers to the IncOnlyMinCbf with optimization...");
    // add all kmers to the iomcbf, measuring the time
    let start = std::time::Instant::now();
    for kmer in kmers.iter() {
        iomcbf.add(kmer);
    }
    let duration = start.elapsed();
    println!("Insertion time, with the optimization: {:?}",
        duration);

    // for each element in the iomcbf, store its overestimation
    // provide the sum, mean, median, and standard deviation of the overestimations
    println!("Querying {} kmers from the IncOnlyMinCbf...", kmers.len());
    let start: std::time::Instant = std::time::Instant::now();
    let mut overestimations = vec![];
    for kmer in kmers.iter() {
        let count = iomcbf.count(kmer);
        let true_count = kmers.iter().filter(|&x| x == kmer).count();
        let overestimation = count as i32 - true_count as i32;
        overestimations.push(overestimation);
    }
    let duration: std::time::Duration = start.elapsed();
    println!("Query time: {:?}", duration);

    println!("Overestimation rate, optimized:");
    print_stats(overestimations);

    // Make the same test without the optimization of incrementing only the smallest counters
    let mut cbf = IncOnlyMinCbf::new(size, num_hashes,x); 

    println!("Adding kmers to the IncOnlyMinCbf without optimization...");
    // add all kmers to the iomcbf, measuring the time
    let start = std::time::Instant::now();
    for kmer in kmers.iter() {
        cbf.add_all(kmer);
    }
    let duration = start.elapsed();
    println!("Insertion time, without the optimization: {:?}",
        duration);

    // for each element in the iomcbf, store its overestimation
    // provide the sum, mean, median, and standard deviation of the overestimations
    println!("Querying {} kmers from the IncOnlyMinCbf...", kmers.len());
    let start = std::time::Instant::now();
    let mut overestimations_non_optimized = vec![];
    for kmer in kmers.iter() {
        let count = cbf.count(kmer);
        let true_count = kmers.iter().filter(|&x| x == kmer).count();
        let overestimation = count as i32 - true_count as i32;
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
}


fn main() {
    let x = 4; // Number of bits per counter (max value = 2^4 - 1 = 15)
    let num_hashes = 7; // Number of hash functions
    let size = 100_000; // Number of counters
    let n_elements = 50_000; // Number of elements to insert
    println!("Checking there are no FN and no underestimations...");
    check_no_fn_no_underestimation(x, num_hashes, size, n_elements);

    println!("Checking false positive rate...");
    check_fp_rate(x, num_hashes, size, n_elements);

    println!("Checking overestimation rate...");
    check_overestimation_rate(x, num_hashes, size, n_elements);

    println!("All tests passed!");
}
