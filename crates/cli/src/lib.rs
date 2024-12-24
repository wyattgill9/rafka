use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "Rafka", about = "Rafka, an async distributed message queue written in rust ",version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the message broker
    Broker {
        #[arg(short, long, default_value = "50051")]
        port: u16,

        #[arg(long, default_value = "0")]
        partition: u32,

        #[arg(short, long, default_value = "1")]
        total_partition: u32,

        #[arg(short, long, default_value = "1")]
        retention_secs: u64,
    },

    /// Start a consumer for the message broker
    Consumer {
        #[arg(short, long, default_value = "50051")]
        port: u16,

        #[arg(long, default_value = "0")]
        partition: u32,
    },

    /// Produces messages for a list of brokers
    Producer {
        #[arg(short, long, default_value = "127.0.0.1:50051")]
        brokers: Vec<String>,

        #[arg(short, long, default_value = "default-key")]
        key: String,

        #[arg(short, long, default_value = "greetings")]
        topic: String,

        #[arg(short, long, default_value = "Hello, world!")]
        message: String,
    },
}

impl CLI {
    // This exists to main code doesnt need to import clap
    pub fn get_parse() -> Commands {
        CLI::parse().command
    }
}
