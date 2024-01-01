use rand::{
    distributions::{Alphanumeric, DistString},
    Rng
};

pub fn random_name(prefix: String) -> String {
    let mut rng = rand::thread_rng();

    format!("{}_{}", prefix, rng.gen::<u32>())
}

pub fn random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();

    Alphanumeric.sample_string(&mut rng, length)
}