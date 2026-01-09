use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_prometheus::PrometheusMetricLayer;

pub fn setup_metrics() -> (PrometheusMetricLayer<'static>, PrometheusHandle) {
    PrometheusMetricLayer::pair()
}
