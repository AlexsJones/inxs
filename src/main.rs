use crate::database::Database;
use clap::{Parser, Subcommand};
use std::{fs::OpenOptions, io::Write};
mod database;
mod indexer;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[clap(subcommand)]
    subcommands: SubCommands,
}
#[derive(Parser, Debug)]
enum SubCommands {
    Index {
        #[clap(short, long, required = true)]
        path: String,
    },
    Check {
        #[clap(short, long, required = true)]
        path: String,
        #[clap(short, long, required = false)]
        output: String,
    },
    Clean {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let etcd = database::Etcd::new().await?;
    let mut indexer = indexer::Indexer::new(etcd);
    let args: Args = Args::parse();

    match args.subcommands {
        SubCommands::Index { path } => {
            indexer.index(&path, false).await;
        }
        SubCommands::Check { path, output } => {
            let differences = indexer.index(&path, true).await;
            if let Some(differences) = differences {
                // Open the output file to write if selected
                for f in differences.changed.clone() {
                    println!("{} has changed", f);
                }
                if !output.is_empty() {
                    let mut file = OpenOptions::new()
                        .read(true)
                        .write(true)
                        .create(true)
                        .open(output)
                        .unwrap();
                    for f in differences.changed.clone() {
                        let mut str = f.clone();
                        str.push('\n');
                        file.write_all(str.as_bytes())?
                    }
                }
            }
        }
        SubCommands::Clean {} => {
            indexer.clear().await;
        }
    }

    Ok(())
}
