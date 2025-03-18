# Minimal increase counting bloom filter
A classical counting Bloom filter limiting the overestimation rate by increasing only minimal values at insertion time.

Blog post: [https://pierrepeterlongo.github.io/2025/03/17/minimal-increase-counting-bloom-filter.html](https://pierrepeterlongo.github.io/2025/03/17/minimal-increase-counting-bloom-filter.html)

## License 
AGPL-3.0

## Install
```bash
git clone https://github.com/pierrepeterlongo/minimal_increment_CBF
cd minimal_increment_CBF 
cargo install --path .  
```

The executable name is `iomcbf`.

## Generate documentation for API usage:
```bash
cargo doc
```

## Usage
The `lib.rs` file contains a class including the creation, the insertion, and the query of words in a "minimal increament counting bloom filter.

Results presented in the blog post were obtained by running `iomcbf`.