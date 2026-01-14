use crate::keybind::Keybind;

pub trait Source {
    fn name(&self) -> &str;
    fn discover(&self) -> Result<Vec<Keybind>, Box<dyn std::error::Error>>;
}
