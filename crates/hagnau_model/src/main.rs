use scraper::CongestionTrainingInput;
use smartcore::{
    ensemble::random_forest_regressor::{RandomForestRegressor, RandomForestRegressorParameters},
    linalg::basic::matrix::DenseMatrix,
};

pub fn main() {
    // get db path from first argument
    let db_path = std::env::args()
        .nth(1)
        .expect("Please provide the path to the SQLite DB file as the first command line argument");

    let scraper = hagnau_sources::make_hagnau_scraper(db_path);
    let view = scraper.make_view();

    let mut x = Vec::with_capacity(view.num_points() * CongestionTrainingInput::N_FEATURES);
    let mut y = Vec::with_capacity(view.num_points());

    for tp in view.training_points("adac", false) {
        x.extend(tp.input.into_features());
        y.push(tp.congestion);
    }

    let x_matrix =
        DenseMatrix::new(y.len(), CongestionTrainingInput::N_FEATURES, x, false).unwrap();

    let model =
        RandomForestRegressor::fit(&x_matrix, &y, RandomForestRegressorParameters::default())
            .unwrap();

    let now = chrono::Utc::now();
    let prediction_input: CongestionTrainingInput = now.into();
    let prediction_matrix = DenseMatrix::new(
        1,
        CongestionTrainingInput::N_FEATURES,
        prediction_input.into_features().to_vec(),
        false,
    )
    .unwrap();
    let prediction = model.predict(&prediction_matrix).unwrap();

    println!("Prediction at {}: {}", now, prediction[0]);
}
