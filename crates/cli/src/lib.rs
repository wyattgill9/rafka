use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
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
    Consumer {
        #[arg(short, long, default_value = "50051")]
        port: u16,

        #[arg(long, default_value = "0")]
        partition: u32,
    },
    Producer {
        #[arg(short, long, default_value = "127.0.0.1:50051")]
        brokers: Vec<String>,

        #[arg(short, long, default_value = "Hello, world!")]
        message: String,

        #[arg(short, long, default_value = "default-key")]
        key: String,
    },
    Metrics {
        #[arg(short, long, default_value = "50051")]
        port: u16,
    },
}

impl CLI {
    // This exists to main code doesnt need to import clap
    pub fn get_parse() -> Commands {
        CLI::parse().command
    }
}
