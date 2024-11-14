use crate::config::{Config, ConfigError};

pub trait Configurable {
    fn configure(&mut self, config: &Config) -> Result<(), ConfigError>;
}
