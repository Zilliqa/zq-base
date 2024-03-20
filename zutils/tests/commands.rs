use zutils::commands::CommandBuilder;

#[tokio::test]
async fn test_no_color() {
    let _ = CommandBuilder::new()
        .throw_on_failure()
        .cmd("git", &["status"])
        .log_output()
        .run()
        .await
        .expect("Error executing command");
}

#[tokio::test]
async fn test_color() {
    let _ = CommandBuilder::new()
        .throw_on_failure()
        .cmd("git", &["status"])
        .log_output()
        .color(colored::Color::BrightGreen)
        .run()
        .await
        .expect("Error executing command");
}
