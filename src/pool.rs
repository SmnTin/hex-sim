use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashMap},
};

use crate::{
    data::{ShopId, Transaction},
    util::F64AsKey,
};

pub trait AccountsPool {
    /**
     * Process all transaction as though they happen in parallel.
     */
    fn process_transactions(&mut self, transactions: &[Transaction]);

    /**
     * Withdraw all money from all accounts from the pool
     * and distribute between shops.
     *
     * Returns the total number of transactions.
     */
    fn withdraw_all(&mut self) -> usize;

    /**
     * Returns the total number of accounts in all pools.
     */
    fn total_accounts(&self) -> usize;

    /**
     * Returns the name of the pool.
     * Receives &self to be object-safe.
     */
    fn name(&self) -> &'static str;
}

#[derive(Debug, Default)]
pub struct PoolPerShop {
    pools: HashMap<ShopId, Vec<f64>>,
}

impl AccountsPool for PoolPerShop {
    fn process_transactions(&mut self, transactions: &[Transaction]) {
        let mut txs_per_shop = HashMap::<ShopId, Vec<f64>>::new();
        for &Transaction { shop_id, amount } in transactions {
            let txs = txs_per_shop.entry(shop_id).or_default();
            txs.push(amount);
        }

        for (shop_id, txs) in txs_per_shop {
            let pool = self.pools.entry(shop_id).or_default();
            if pool.len() < txs.len() {
                pool.resize(txs.len(), 0.0);
            }

            for (account, amount) in pool.iter_mut().zip(txs) {
                *account += amount;
            }
        }
    }

    fn withdraw_all(&mut self) -> usize {
        for (_, pool) in &mut self.pools {
            pool.fill(0.0);
        }
        self.total_accounts()
    }

    fn total_accounts(&self) -> usize {
        self.pools.iter().map(|(_, pool)| pool.len()).sum()
    }

    fn name(&self) -> &'static str {
        "Pool per Shop"
    }
}

impl PoolPerShop {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default)]
pub struct SinglePool {
    pool: BinaryHeap<Reverse<F64AsKey>>,
    shop_balances: HashMap<ShopId, f64>,
}

impl AccountsPool for SinglePool {
    fn process_transactions(&mut self, transactions: &[Transaction]) {
        let mut updated_accounts = vec![];
        for &Transaction { shop_id, amount } in transactions {
            let balance = self.shop_balances.entry(shop_id).or_default();
            *balance += amount;

            let account = self.pool.pop().unwrap_or_default();
            let updated_account = Reverse((amount + *account.0).into());
            updated_accounts.push(updated_account);
        }
        self.pool.extend(updated_accounts)
    }

    fn withdraw_all(&mut self) -> usize {
        let mut current = 0;
        let mut accounts = self.accounts();
        let mut total_transactions = 0;

        'outer: for (_, balance) in &mut self.shop_balances {
            while *balance > 0.0 {
                while accounts[current] == 0.0 {
                    current += 1;
                    if current == accounts.len() {
                        break 'outer;
                    }
                }
                let amount = balance.min(accounts[current]);
                accounts[current] -= amount;
                *balance -= amount;
                total_transactions += 1;
            }
        }
        self.reset();
        total_transactions
    }

    fn total_accounts(&self) -> usize {
        self.pool.len()
    }

    fn name(&self) -> &'static str {
        "Single Pool"
    }
}

impl SinglePool {
    pub fn new() -> Self {
        Self::default()
    }

    fn accounts(&self) -> Vec<f64> {
        self.pool.iter().map(|account| *account.0).collect()
    }

    fn reset(&mut self) {
        self.pool = (0..self.pool.len())
            .map(|_| Reverse(F64AsKey::new(0.0)))
            .collect();
        self.shop_balances.clear();
    }

    fn shop_balances(&self) -> &HashMap<ShopId, f64> {
        &self.shop_balances
    }
}

#[derive(Debug, Default)]
pub struct SinglePoolWithSingleAccount {
    inner: SinglePool,
}

impl SinglePoolWithSingleAccount {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AccountsPool for SinglePoolWithSingleAccount {
    fn process_transactions(&mut self, transactions: &[Transaction]) {
        self.inner.process_transactions(transactions);
    }

    fn withdraw_all(&mut self) -> usize {
        let total_transactions =
            self.inner.total_accounts() + self.inner.shop_balances().len();
        self.inner.reset();
        total_transactions
    }

    fn total_accounts(&self) -> usize {
        self.inner.total_accounts()
    }

    fn name(&self) -> &'static str {
        "Single Pool with Single Account"
    }
}
