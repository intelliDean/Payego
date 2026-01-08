mod common;

use axum_test::TestServer;
use serde_json::json;
use common::{create_test_app, create_test_app_state};
use http::StatusCode;

#[tokio::test]
async fn test_rate_limiting() {
    let state = create_test_app_state();
    let app = create_test_app(state);
    let server = TestServer::new(app).unwrap();

    // The rate limit is set to 2 requests per second with a burst of 10.
    // We'll send 15 requests quickly to trigger the limit.
    
    let mut successes = 0;
    let mut limited = 0;

    for _ in 0..15 {
        let response = server
            .get("/api/banks") // A public route
            .await;

        if response.status_code() == StatusCode::TOO_MANY_REQUESTS {
            limited += 1;
        } else if response.status_code() == StatusCode::OK {
            successes += 1;
        }
    }

    println!("Successes: {}, Limited: {}", successes, limited);
    
    // We expect at least some to be limited if the burst is 10 and we send 15.
    assert!(limited > 0, "Expected some requests to be rate limited");
    assert!(successes >= 10, "Expected at least 10 requests (burst) to succeed");
}
