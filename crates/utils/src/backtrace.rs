use std::backtrace::Backtrace;

/// Macro for logging errors with filtered backtraces
/// Example
/// ```
/// error_with_backtrace!(error = %err, "Something went wrong");
/// ```
#[macro_export]
macro_rules! error_with_backtrace {
    ($($arg:tt)*) => {
        {
            let filtered_trace = $crate::backtrace::app_focused_backtrace();
            tracing::error!(
                backtrace = %filtered_trace,
                $($arg)*
            );
        }
    };
}

/// Filter the backtrace to only include lines that are relevant to the application
pub fn app_focused_backtrace() -> String {
    let backtrace = Backtrace::capture();
    let backtrace_str = format!("{backtrace:?}");

    backtrace_str
        .lines()
        .filter(|line| {
            // Include lines that contain application-specific paths
            (line.contains("nittei_") || 
             line.contains("app/nittei/") ||
             line.contains("crates/") ||
             line.contains("bins/nittei/"))
                && !line.contains("registry/src")  // Exclude registry source code
                && !line.contains("std::")         // Optionally exclude standard library
                && !line.contains("core::") // Optionally exclude core library
        })
        .collect::<Vec<_>>()
        .join("\n")
}
