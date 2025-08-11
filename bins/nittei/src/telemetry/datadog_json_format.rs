use nittei_api::telemetry::correlation_layer::CorrelationId;
use opentelemetry::trace::TraceContextExt;
use tracing::{Event, Subscriber};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{
    fmt::format::{FormatEvent, FormatFields},
    registry::LookupSpan,
};

pub struct DatadogJsonFmt;

impl<S, N> FormatEvent<S, N> for DatadogJsonFmt
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        ctx: &tracing_subscriber::fmt::FmtContext<'_, S, N>,
        mut writer: tracing_subscriber::fmt::format::Writer<'_>,
        event: &Event<'_>,
    ) -> std::fmt::Result {
        use std::borrow::Cow;

        // Collect the event fields into a JSON map
        let mut fields_map = serde_json::Map::new();

        // Custom visitor to collect fields
        struct FieldVisitor<'a>(&'a mut serde_json::Map<String, serde_json::Value>);

        impl<'a> tracing::field::Visit for FieldVisitor<'a> {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                self.0.insert(
                    field.name().to_string(),
                    serde_json::Value::String(format!("{:?}", value)),
                );
            }

            fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
                self.0.insert(
                    field.name().to_string(),
                    serde_json::Value::String(value.to_string()),
                );
            }

            fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
                self.0.insert(
                    field.name().to_string(),
                    serde_json::Value::Number(serde_json::Number::from(value)),
                );
            }

            fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
                self.0.insert(
                    field.name().to_string(),
                    serde_json::Value::Number(serde_json::Number::from(value)),
                );
            }

            fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
                self.0
                    .insert(field.name().to_string(), serde_json::Value::Bool(value));
            }
        }

        let mut visitor = FieldVisitor(&mut fields_map);
        event.record(&mut visitor);
        let fields_value = serde_json::Value::Object(fields_map);

        // Get current span's otel context -> Datadog ids
        let (dd_trace_id, dd_span_id) = {
            let span = tracing::Span::current();
            let ctx_otel = span.context();
            let otel_span = ctx_otel.span();
            let sc = otel_span.span_context();
            if sc.is_valid() {
                // Datadog expects 64-bit decimal ids:
                let trace_id_bytes = sc.trace_id().to_bytes();
                let tid64 = u64::from_be_bytes([
                    trace_id_bytes[8],
                    trace_id_bytes[9],
                    trace_id_bytes[10],
                    trace_id_bytes[11],
                    trace_id_bytes[12],
                    trace_id_bytes[13],
                    trace_id_bytes[14],
                    trace_id_bytes[15],
                ]);
                let sid64 = u64::from_be_bytes(sc.span_id().to_bytes());
                (Some(tid64), Some(sid64))
            } else {
                (None, None)
            }
        };

        // Include correlation_id field from the current span if present
        let correlation_id = {
            if let Some(scope) = ctx.lookup_current() {
                scope
                    .extensions()
                    .get::<CorrelationId>()
                    .map(|c| Cow::from(c.0.clone()))
            } else {
                None
            }
        };

        // Build final JSON
        let mut obj = serde_json::Map::new();

        // timestamp
        obj.insert(
            "@timestamp".into(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );

        // level & target
        let meta = event.metadata();
        obj.insert("level".into(), meta.level().to_string().into());
        obj.insert("target".into(), meta.target().into());

        // event fields
        if let serde_json::Value::Object(map) = fields_value {
            for (k, v) in map {
                obj.insert(k, v);
            }
        }

        // correlation id (if any)
        if let Some(cid) = correlation_id {
            obj.insert(
                "correlation_id".into(),
                serde_json::Value::String(cid.into_owned()),
            );
        }

        // datadog correlation fields
        if let (Some(t), Some(s)) = (dd_trace_id, dd_span_id) {
            obj.insert(
                "dd.trace_id".into(),
                serde_json::Value::String(t.to_string()),
            );
            obj.insert(
                "dd.span_id".into(),
                serde_json::Value::String(s.to_string()),
            );
        }

        // write out one line
        let json_str =
            serde_json::to_string(&serde_json::Value::Object(obj)).map_err(|_| std::fmt::Error)?;
        writeln!(writer, "{}", json_str)
    }
}
