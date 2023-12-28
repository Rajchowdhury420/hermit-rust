use rand::Rng;

pub fn random_name(prefix: String) -> String {
    let mut rng = rand::thread_rng();

    format!("{}_{}", prefix, rng.gen::<u32>())
}