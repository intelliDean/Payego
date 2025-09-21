// use tracing_subscriber::EnvFilter;
//
// pub fn setup_logging() {
//     tracing_subscriber::fmt()
//         .with_env_filter(
//             EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")), // Default to debug
//         )
//         .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout())) // Auto ANSI detection
//         .with_target(true) // Show target/module names
//         .with_thread_ids(true) // Show thread IDs
//         .init();
//
//     tracing::info!(
//         "Logging initialized with level: {:?}",
//         std::env::var("RUST_LOG")
//     );
// }

use tracing_subscriber::EnvFilter;
pub fn setup_logging() {
    let is_terminal = std::io::IsTerminal::is_terminal(&std::io::stdout());
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    if is_terminal {
        tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_ansi(true)
            .with_target(true)
            .with_thread_ids(true)
            .init();
    } else {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .init();
    }
    tracing::info!(
        "Logging initialized with level: {:?}",
        std::env::var("RUST_LOG")
    );
}
