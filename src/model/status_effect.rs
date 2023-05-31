use super::stats::Stats;

#[derive(Debug, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum StatusEffect {
    GainStat(Stats),
}
