use std::time::SystemTime;
use chrono::prelude::{DateTime, Utc};
use rand::distributions::Alphanumeric;
use rand::distributions::DistString;

pub fn rfc3339(st: &SystemTime) -> String {
    let dt: DateTime<Utc> = st.clone().into();
    dt.to_rfc3339() 
}

pub fn generate_string(length: usize) -> String {
    let mut rng = rand::thread_rng();

    // Generate alphanumeric characters
    let alphanumeric_string: String = Alphanumeric.sample_string(&mut rng, length);

    // Replace some characters with special characters
    let final_string: Vec<char> = alphanumeric_string.chars().collect();

    final_string.into_iter().collect::<String>().to_uppercase()
}
