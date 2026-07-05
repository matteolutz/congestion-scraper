use crate::CongestionTrainingPoint;

pub mod random_forest;

pub struct CongestionModelFitInput<I: Iterator<Item = CongestionTrainingPoint>> {
    pub(crate) n_points: usize,
    pub(crate) points: I,
}

pub trait CongestionModel: Sized {
    type Error;

    fn fit<T>(input: CongestionModelFitInput<T>) -> Result<Self, Self::Error>
    where
        T: Iterator<Item = CongestionTrainingPoint>;

    fn predict(
        &self,
        xs: impl IntoIterator<Item = chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<f64>, Self::Error>;
}

pub trait CongestionModelExt<M: CongestionModel> {
    fn predict_single(&self, x: chrono::DateTime<chrono::Utc>) -> Result<f64, M::Error>;
}

impl<M: CongestionModel> CongestionModelExt<M> for M {
    fn predict_single(
        &self,
        x: chrono::DateTime<chrono::Utc>,
    ) -> Result<f64, <M as CongestionModel>::Error> {
        let vals = self.predict(std::iter::once(x))?;
        let val = vals
            .first()
            .expect("predict should return at least one value");
        Ok(*val)
    }
}
