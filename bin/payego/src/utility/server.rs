use crate::utility::shutdown::shutdown_signal;
use axum::Router;
use eyre::Report;
use std::net::SocketAddr;

pub async fn serve(router: Router) -> Result<(), Report> {
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port = std::env::var("PORT").unwrap_or_else(|_| "8080".into());

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .map_err(|e| eyre::eyre!("Invalid bind address: {}", e))?;

    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Swagger UI: http://{}/swagger-ui/", addr);

    axum::serve(
        tokio::net::TcpListener::bind(&addr).await?,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await?;

    Ok(())
}
