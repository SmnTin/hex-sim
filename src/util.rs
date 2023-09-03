use evalexpr::*;
use serde::{de, Deserialize, Deserializer};
use std::{
    cmp::Ordering,
    hash::{Hash, Hasher},
    ops::Deref,
};

use crate::data::{
    DailyMultipliers, DailyOrdersDistribution, DAYS_IN_YEAR, HOURS_IN_DAY,
};

#[repr(transparent)]
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct F64AsKey(f64);

impl F64AsKey {
    pub fn new(value: f64) -> Self {
        assert!(
            !value.is_nan(),
            "Can't implement Ord and Eq when NaNs are present."
        );
        Self(value)
    }

    pub fn inner(&self) -> f64 {
        self.0
    }
}

impl From<f64> for F64AsKey {
    fn from(value: f64) -> Self {
        Self::new(value)
    }
}

impl From<F64AsKey> for f64 {
    fn from(value: F64AsKey) -> f64 {
        value.0
    }
}

impl Deref for F64AsKey {
    type Target = f64;
    fn deref(&self) -> &f64 {
        &self.0
    }
}

impl Hash for F64AsKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Eq for F64AsKey {}
impl Ord for F64AsKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

pub fn deserialize_daily_orders_distribution<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<DailyOrdersDistribution, D::Error> {
    let expr = String::deserialize(de)?;
    let mut distribution = [0; HOURS_IN_DAY];
    for hour in 0..HOURS_IN_DAY {
        distribution[hour] =
            eval_expr(&expr, "h", hour).map_err(de::Error::custom)?;
    }
    Ok(distribution)
}

pub fn deserialize_daily_multipliers<'de, D: Deserializer<'de>>(
    de: D,
) -> Result<DailyMultipliers, D::Error> {
    let expr = String::deserialize(de)?;
    let mut distribution = [0; DAYS_IN_YEAR];
    for day in 0..DAYS_IN_YEAR {
        distribution[day] =
            eval_expr(&expr, "d", day).map_err(de::Error::custom)?;
    }
    Ok(distribution)
}

fn eval_expr(
    expr: &str,
    var_name: &str,
    var_value: usize,
) -> Result<usize, EvalexprError> {
    let mut context = HashMapContext::new();
    context.set_value(var_name.into(), (var_value as i64).into())?;

    let value = eval_number_with_context(expr.into(), &context)?;

    Ok(value.round() as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_eval_expr() {
        let value =
            eval_expr("math::exp(x * x) * math::sin(x)", "x", 5).unwrap();
        let x: f64 = 5.0;
        let expected_value = x.powi(2).exp() * x.sin();
        assert_eq!(value, expected_value.round() as usize);
    }
}
