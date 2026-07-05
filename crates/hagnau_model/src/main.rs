use plotters::{
    backend::BitMapBackend,
    chart::{ChartBuilder, LabelAreaPosition},
    drawing::IntoDrawingArea,
    element::PathElement,
    series::{DashedLineSeries, LineSeries},
    style::{BLUE, GREEN, WHITE, full_palette::LIGHTBLUE},
};
use scraper::CongestionTrainingInput;
use smartcore::{
    ensemble::random_forest_regressor::{RandomForestRegressor, RandomForestRegressorParameters},
    linalg::basic::matrix::DenseMatrix,
};

use crate::plotters_ext::IntoChronoHourly;

mod plotters_ext;

pub fn main() {
    // get db path from first argument
    let db_path = std::env::args()
        .nth(1)
        .expect("Please provide the path to the SQLite DB file as the first command line argument");

    let scraper = hagnau_sources::make_hagnau_scraper(db_path);
    let view = scraper.make_view();

    // the chrono today date at 0:00
    let start_of_today: chrono::DateTime<chrono::Utc> = chrono::Utc::now()
        .with_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
        .unwrap();
    let end_of_today: chrono::DateTime<chrono::Utc> = start_of_today + chrono::Duration::days(1);

    let root = BitMapBackend::new("plot.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();

    let mut chart = ChartBuilder::on(&root)
        .margin(10)
        .caption(
            format!("Congestion @ Hagnau ({})", start_of_today.date_naive()),
            ("sans-serif", 40),
        )
        .set_label_area_size(LabelAreaPosition::Left, 60)
        .set_label_area_size(LabelAreaPosition::Right, 60)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d((start_of_today..end_of_today).hourly(), 0.0..60.0)
        .unwrap();

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .x_labels(30)
        .max_light_lines(4)
        .y_desc("Congestion in minutes")
        .draw()
        .unwrap();

    let inbound_congestion = view
        .timestamped_values_for("adac")
        .skip_while(|(timestamp, _)| timestamp < &start_of_today)
        .take_while(|(timestamp, _)| timestamp < &end_of_today)
        .map(|(timestamp, congestion)| {
            (
                timestamp,
                congestion.map(|c| c.inbound.as_minutes()).unwrap_or(0.0),
            )
        });

    chart
        .draw_series(LineSeries::new(inbound_congestion, &BLUE))
        .unwrap()
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], BLUE))
        .label("Inbound (to Stockach)");

    let outbound_congestion = view
        .timestamped_values_for("adac")
        .skip_while(|(timestamp, _)| timestamp < &start_of_today)
        .take_while(|(timestamp, _)| timestamp < &end_of_today)
        .map(|(timestamp, congestion)| {
            (
                timestamp,
                congestion.map(|c| c.outbound.as_minutes()).unwrap_or(0.0),
            )
        });

    chart
        .draw_series(LineSeries::new(outbound_congestion, &GREEN))
        .unwrap()
        .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], GREEN))
        .label("Outbound (to FN)");

    let mut x = Vec::with_capacity(view.num_points() * CongestionTrainingInput::N_FEATURES);
    let mut y = Vec::with_capacity(view.num_points());

    for tp in view.training_points("adac", true) {
        x.extend(tp.input.into_features());
        y.push(tp.congestion);
    }

    let x_matrix =
        DenseMatrix::new(y.len(), CongestionTrainingInput::N_FEATURES, x, false).unwrap();

    let model =
        RandomForestRegressor::fit(&x_matrix, &y, RandomForestRegressorParameters::default())
            .unwrap();

    // let model_bytes: Vec<u8> = postcard::to_allocvec(&model).unwrap();
    // std::fs::write("model.dat", &model_bytes).unwrap();

    let predict_points = 100;

    let mut prediction_chart = Vec::with_capacity(predict_points);

    let mut prediction_features =
        Vec::with_capacity(predict_points * CongestionTrainingInput::N_FEATURES);

    for i in 0..predict_points {
        let now = chrono::Utc::now() + chrono::Duration::minutes(i as i64 * 5);
        prediction_chart.push(now);

        let prediction_input: CongestionTrainingInput = now.into();

        prediction_features.extend(prediction_input.into_features());
    }

    let prediction_matrix = DenseMatrix::new(
        predict_points,
        CongestionTrainingInput::N_FEATURES,
        prediction_features,
        false,
    )
    .unwrap();
    let prediction = model.predict(&prediction_matrix).unwrap();

    let prediction_chart = prediction_chart.into_iter().zip(prediction.iter().copied());

    chart
        .draw_series(DashedLineSeries::new(
            prediction_chart,
            1,
            5,
            LIGHTBLUE.into(),
        ))
        .unwrap();

    chart.configure_series_labels().draw().unwrap();

    root.present().expect("Unable to write result to file");

    println!("Prediction for the next..");
    for i in 0..predict_points {
        println!("\t{} minutes: {}", i * 5, prediction[i])
    }
}
