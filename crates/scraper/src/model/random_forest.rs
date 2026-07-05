use smartcore::{
    ensemble::random_forest_regressor::{RandomForestRegressor, RandomForestRegressorParameters},
    error::Failed,
    linalg::basic::matrix::DenseMatrix,
};

use crate::{CongestionModel, CongestionTrainingInput};

pub struct CongestionRandomForestModel(RandomForestRegressor<f64, f64, DenseMatrix<f64>, Vec<f64>>);

impl CongestionModel for CongestionRandomForestModel {
    type Error = Failed;

    fn fit<T>(input: super::CongestionModelFitInput<T>) -> Result<Self, Self::Error>
    where
        T: Iterator<Item = crate::CongestionTrainingPoint>,
    {
        let mut x = Vec::with_capacity(input.n_points * CongestionTrainingInput::N_FEATURES);
        let mut y = Vec::with_capacity(input.n_points);

        for p in input.points {
            x.extend(p.input.into_features());
            y.push(p.congestion);
        }

        let x_matrix = DenseMatrix::new(y.len(), CongestionTrainingInput::N_FEATURES, x, false)?;
        let model =
            RandomForestRegressor::fit(&x_matrix, &y, RandomForestRegressorParameters::default())?;

        Ok(Self(model))
    }

    fn predict(
        &self,
        xs: impl IntoIterator<Item = chrono::DateTime<chrono::Utc>>,
    ) -> Result<Vec<f64>, Self::Error> {
        let predict_features = xs
            .into_iter()
            .flat_map(|p| {
                let p: CongestionTrainingInput = p.into();
                p.into_features()
            })
            .collect::<Vec<_>>();
        let n_points = predict_features.len() / CongestionTrainingInput::N_FEATURES;

        let x_matrix = DenseMatrix::new(
            n_points,
            CongestionTrainingInput::N_FEATURES,
            predict_features,
            false,
        )?;

        let prediction = self.0.predict(&x_matrix)?;
        Ok(prediction)
    }
}
