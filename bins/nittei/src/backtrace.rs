use nittei_utils::backtrace::app_focused_backtrace;

/// Install a custom panic hook to filter the backtrace to only include lines that are relevant to the application
pub fn install_custom_panic_hook() {
    let default_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Log with filtered backtrace
        let filtered_trace = app_focused_backtrace();

        tracing::error!(
            panic = %panic_info,
            backtrace = %filtered_trace,
            "Application panic occurred"
        );

        // Call the default hook too
        default_hook(panic_info);
    }));
}
