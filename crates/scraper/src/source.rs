use crate::CongestionAmount;

pub trait CongestionSource {
    fn source_id(&self) -> &'static str;
    fn poll(&self) -> Option<CongestionAmount>;
}
