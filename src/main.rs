#![feature(array_try_map)]

use anyhow::Result;
use clap::Parser;
use data::{SimConfig, SimResults};
use pool::{PoolPerShop, SinglePool, SinglePoolWithSingleAccount};
use rand::{rngs::SmallRng, RngCore, SeedableRng};
use sim::{AnnualData, GlobalData, GlobalStats, PoolStats};
use std::fs::File;

use crate::{
    data::DAYS_IN_YEAR,
    sim::{simulate_day, DailyData},
};

mod data;
mod pool;
mod sim;
mod util;

struct Args {
    config: SimConfig,
    seed: Option<u64>,
}

fn read_args() -> Result<Args> {
    #[derive(Parser, Debug)]
    #[command(author, version, about, long_about = None)]
    struct CmdArgs {
        #[arg(long, short)]
        config: String,

        #[arg(long, short)]
        seed: Option<u64>,
    }

    let args = CmdArgs::parse();
    let file = File::open(args.config)?;
    let config: SimConfig = serde_json::from_reader(file)?;

    Ok(Args {
        config,
        seed: args.seed,
    })
}

fn write_results(results: SimResults) -> Result<()> {
    println!(
        "Total number of transactions: {}",
        results.total_number_of_transactions
    );
    println!(
        "Peak parallel transactions number: {}",
        results.peak_parallel_transactions_number
    );

    for pool_results in results.pool_results {
        println!("");
        println!("Results for {}:", pool_results.pool_name);
        println!(
            "Total number of accounts: {}",
            pool_results.total_number_of_accounts
        );
        println!(
            "Total number of transactions during withdrawals: {}",
            pool_results.total_number_of_transactions_during_withdrawals
        );
    }

    Ok(())
}

fn main() -> Result<()> {
    let Args { config, seed } = read_args()?;

    let seed = seed.unwrap_or_else(|| rand::thread_rng().next_u64());
    let mut rng = SmallRng::seed_from_u64(seed);

    println!("Seed: {}", seed);

    let global_data = GlobalData::gen(&mut rng, &config);
    let mut global_stats = GlobalStats::default();

    let mut pool_per_shop = PoolPerShop::new();
    let mut single_pool = SinglePool::new();
    let mut single_pool_with_single_account =
        SinglePoolWithSingleAccount::new();

    let mut pool_per_shop_stats = PoolStats::default();
    let mut single_pool_stats = PoolStats::default();
    let mut single_pool_with_single_account_stats = PoolStats::default();

    for _year in 0..config.simulated_years_number {
        let annual_data = AnnualData::gen(&mut rng, &config, &global_data);
        for day in 0..DAYS_IN_YEAR {
            let daily_data =
                DailyData::gen(&mut rng, &config, &annual_data, day);
            global_stats.update(&daily_data);

            simulate_day(
                &daily_data,
                &mut pool_per_shop,
                &mut pool_per_shop_stats,
            );
            simulate_day(&daily_data, &mut single_pool, &mut single_pool_stats);
            simulate_day(
                &daily_data,
                &mut single_pool_with_single_account,
                &mut single_pool_with_single_account_stats,
            );
        }
    }

    let results = global_stats.results(vec![
        pool_per_shop_stats.results(&pool_per_shop),
        single_pool_stats.results(&single_pool),
        single_pool_with_single_account_stats
            .results(&single_pool_with_single_account),
    ]);
    write_results(results)?;

    Ok(())
}
