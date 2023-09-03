use rand::Rng;
use rand_distr::Distribution;

use crate::{
    data::{
        AnnualOrdersDistribution, PoolResults, SimConfig, SimResults,
        Transaction, DAYS_IN_YEAR, HOURS_IN_DAY,
    },
    pool::AccountsPool,
};

pub struct GlobalData {
    pub shop_sizes: Vec<f64>,
}

impl GlobalData {
    pub fn gen(rng: impl Rng, config: &SimConfig) -> Self {
        Self {
            shop_sizes: config
                .shop_size_distribution
                .sample_iter(rng)
                .filter(|&size| size > 0.0)
                .take(config.simulated_shops_number)
                .collect(),
        }
    }
}

pub struct AnnualData {
    pub shop_distributions: Vec<AnnualOrdersDistribution>,
}

impl AnnualData {
    pub fn gen(
        mut rng: impl Rng,
        config: &SimConfig,
        global_data: &GlobalData,
    ) -> Self {
        Self {
            shop_distributions: global_data
                .shop_sizes
                .iter()
                .map(|&shop_size| {
                    let mut daily_multipliers =
                        config.default_daily_multipliers;
                    for _ in 0..config.sales_per_year_for_each_shop {
                        let i = (rng.next_u32() as usize) % DAYS_IN_YEAR;
                        daily_multipliers[i] *= config.sale_multiplier;
                    }

                    let default_daily_distribution =
                        config.default_daily_distribution.map(|txs_per_hour| {
                            ((txs_per_hour as f64) * shop_size) as usize
                        });

                    AnnualOrdersDistribution {
                        daily_multipliers,
                        default_daily_distribution,
                    }
                })
                .collect(),
        }
    }
}

pub struct DailyData {
    pub transactions: [Vec<Transaction>; HOURS_IN_DAY],
    pub withdrawal: bool,
}

impl DailyData {
    pub fn gen(
        rng: impl Rng,
        config: &SimConfig,
        annual_data: &AnnualData,
        day: usize,
    ) -> Self {
        let mut prices = config
            .price_distribution
            .sample_iter(rng)
            .filter(|&price| price > 0.0);

        Self {
            transactions: std::array::from_fn(|hour| {
                let mut transactions = Vec::new();
                for (shop_id, distr) in
                    annual_data.shop_distributions.iter().enumerate()
                {
                    let txs_number = distr.daily_multipliers[day]
                        * distr.default_daily_distribution[hour];

                    for _ in 0..txs_number {
                        let amount = prices.next().unwrap();
                        transactions.push(Transaction { amount, shop_id });
                    }
                }
                transactions
            }),
            withdrawal: (day % config.withdrawal_period_in_days) == 0,
        }
    }
}

pub fn simulate_day(
    daily_data: &DailyData,
    pool: &mut impl AccountsPool,
    pool_stats: &mut PoolStats,
) {
    for hour in 0..HOURS_IN_DAY {
        let transactions = &daily_data.transactions[hour];
        pool.process_transactions(transactions);
    }

    if daily_data.withdrawal {
        pool_stats.total_number_of_transactions_during_withdrawals +=
            pool.withdraw_all();
    }
}

#[derive(Default)]
pub struct PoolStats {
    total_number_of_transactions_during_withdrawals: usize,
}

impl PoolStats {
    pub fn results(self, pool: &impl AccountsPool) -> PoolResults {
        PoolResults {
            total_number_of_transactions_during_withdrawals: self
                .total_number_of_transactions_during_withdrawals,
            total_number_of_accounts: pool.total_accounts(),
            pool_name: pool.name(),
        }
    }
}

#[derive(Default)]
pub struct GlobalStats {
    total_number_of_transactions: usize,
    peak_parallel_transactions_number: usize,
}

impl GlobalStats {
    pub fn results(&self, pool_results: Vec<PoolResults>) -> SimResults {
        SimResults {
            total_number_of_transactions: self.total_number_of_transactions,
            peak_parallel_transactions_number: self
                .peak_parallel_transactions_number,
            pool_results,
        }
    }

    pub fn update(&mut self, daily_data: &DailyData) {
        for txs in &daily_data.transactions {
            self.peak_parallel_transactions_number =
                self.peak_parallel_transactions_number.max(txs.len());
            self.total_number_of_transactions += txs.len();
        }
    }
}
