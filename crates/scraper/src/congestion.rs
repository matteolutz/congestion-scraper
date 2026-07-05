#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CongestionUnit {
    Kilometers(f64),
    Minutes(f64),
    None,
}

impl CongestionUnit {
    pub fn as_minutes(&self) -> f64 {
        match self {
            CongestionUnit::Kilometers(_) => todo!("km to minutes"),
            CongestionUnit::Minutes(m) => *m,
            CongestionUnit::None => 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum CongestionDirection {
    Inbound,
    Outbound,
    Both,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct CongestionAmount {
    pub inbound: CongestionUnit,
    pub outbound: CongestionUnit,
}

impl CongestionAmount {
    pub fn new(inbound: CongestionUnit, outbound: CongestionUnit) -> Self {
        Self { inbound, outbound }
    }

    pub fn new_minutes(inbound: f64, outbound: f64) -> Self {
        Self {
            inbound: CongestionUnit::Minutes(inbound),
            outbound: CongestionUnit::Minutes(outbound),
        }
    }

    pub fn none() -> Self {
        Self {
            inbound: CongestionUnit::None,
            outbound: CongestionUnit::None,
        }
    }

    pub fn inbound(unit: CongestionUnit) -> Self {
        Self {
            inbound: unit,
            outbound: CongestionUnit::None,
        }
    }

    pub fn outbound(unit: CongestionUnit) -> Self {
        Self {
            inbound: CongestionUnit::None,
            outbound: unit,
        }
    }

    pub fn both(unit: CongestionUnit) -> Self {
        Self {
            inbound: unit,
            outbound: unit,
        }
    }

    pub fn get(&self, direction: CongestionDirection) -> CongestionUnit {
        match direction {
            CongestionDirection::Inbound => self.inbound,
            CongestionDirection::Outbound => self.outbound,
            CongestionDirection::Both => unreachable!(),
        }
    }
}

impl From<CongestionUnit> for CongestionAmount {
    fn from(unit: CongestionUnit) -> Self {
        Self::both(unit)
    }
}

#[derive(Debug, Clone)]
pub struct Congestion {
    timestamp: chrono::DateTime<chrono::Utc>,
    amount: CongestionAmount,
}

pub trait CongestionClassifier {
    type Output;

    fn classify(&self, congestion_unit: &CongestionUnit) -> Self::Output;
}
