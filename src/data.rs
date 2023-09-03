use rand_distr::Normal;
use serde::{Deserialize, Serialize};

use crate::util::{
    deserialize_daily_multipliers, deserialize_daily_orders_distribution,
};

pub const HOURS_IN_DAY: usize = 24;
pub const DAYS_IN_YEAR: usize = 365;

pub type ShopId = usize;

#[derive(Clone, Copy, Debug)]
pub struct Transaction {
    pub amount: f64,
    pub shop_id: ShopId,
}

pub type DailyOrdersDistribution = [usize; HOURS_IN_DAY];
pub type DailyMultipliers = [usize; DAYS_IN_YEAR];

#[derive(Clone, Copy, Debug)]
pub struct AnnualOrdersDistribution {
    pub daily_multipliers: DailyMultipliers,
    pub default_daily_distribution: DailyOrdersDistribution,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SimConfig {
    /** Number of shops in the simulation. */
    pub simulated_shops_number: usize,

    /** Number of years for which simulation is run. */
    pub simulated_years_number: usize,

    /**
     * Probalistic distribution of a shop size.
     * The default daily distribution for this shop
     * is multiplied by this value.
     */
    pub shop_size_distribution: Normal<f64>,

    /**
     * Number of sell-outs each shop conducts per year.
     */
    pub sales_per_year_for_each_shop: usize,

    /**
     * If a sale occurs at that day, we multiply
     * the daily multiplier by this value.
     */
    pub sale_multiplier: usize,

    /**
     * For each day of a year, describes a number by which
     * we multiply the daily distribution. These multipliers
     * are varied for each shop by the size distribution.
     *
     * It is defined by a function that maps
     * a day number to the multiplier at that day.
     */
    #[serde(deserialize_with = "deserialize_daily_multipliers")]
    pub default_daily_multipliers: DailyMultipliers,

    /**
     * For each hour of day, describes a number of orders
     * at that hour.
     *
     * It is defined by a function that maps
     * an hour to the number of orders.
     */
    #[serde(deserialize_with = "deserialize_daily_orders_distribution")]
    pub default_daily_distribution: DailyOrdersDistribution,

    /**
     * Probalistic distribution of a price.
     * When a transaction is issued, its amount is
     * randomly sampled from this distribution.
     */
    pub price_distribution: Normal<f64>,

    /**
     * Let this value be k. Then money will be withdrawed every k days.
     */
    pub withdrawal_period_in_days: usize,
}

#[derive(Serialize)]
pub struct PoolResults {
    pub pool_name: &'static str,
    pub total_number_of_transactions_during_withdrawals: usize,
    pub total_number_of_accounts: usize,
}

#[derive(Serialize)]
pub struct SimResults {
    pub total_number_of_transactions: usize,
    pub peak_parallel_transactions_number: usize,
    pub pool_results: Vec<PoolResults>,
}
