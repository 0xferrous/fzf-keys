use std::fmt::Display;

pub trait Source {
    type Item: Display;

    fn name(&self) -> &str;
    fn discover(&self) -> Result<Vec<Self::Item>, Box<dyn std::error::Error>>;
}
