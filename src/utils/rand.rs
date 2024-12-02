use rand::prelude::*;

pub fn rand_alphanumeric(length: usize) -> String {
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    generate_random_string(charset, length)
}

fn generate_random_string(charset: &[u8], length: usize) -> String {
    let mut rng = rand::thread_rng();

    let random_string: String = (0..length)
        .map(|_| {
            let index = rng.gen_range(0..charset.len());
            charset[index] as char
        })
        .collect();

    random_string
}