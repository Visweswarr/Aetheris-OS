use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use tonic::{transport::Server, Request, Response, Status};
use tonic_health::{
    health_server::{Health, HealthServer},
    HealthCheckResponse, HealthCheckResponse_ServingStatus,
};
use tonic_reflection::server::Builder as ReflectionBuilder;

use hello::hello_service_server::{HelloService, HelloServiceServer};
use hello::{
    EchoRequest, EchoResponse, GetMetricsResponse, HealthCheckResponse as HelloHealthCheckResponse,
    MetricsResponse, SayHelloRequest, SayHelloResponse, SayHelloStreamRequest,
    SayHelloStreamResponse, ServiceInfo, Status as HelloStatus,
};

use opentelemetry::{
    global,
    metrics::{Counter, Histogram, Meter, MeterProvider, ObservableGauge},
    trace::{Span, Tracer},
    KeyValue,
};
use opentelemetry_sdk::{
    metrics::{MeterProvider as SdkMeterProvider, PeriodicReader},
    trace::{TracerProvider, TracerProviderInner},
};
use opentelemetry_sdk::{
    export::metrics::prometheus::PrometheusExporter,
    export::trace::stdout,
    metrics::controllers,
    propagation::TraceContextPropagator,
};
use opentelemetry_prometheus::exporter;
use prometheus::{Encoder, Registry, TextEncoder};

mod hello {
    tonic::include_proto!("hello.v1");
}

// Service implementation
#[derive(Default)]
pub struct HelloServiceImpl {
    metrics: Arc<HelloMetrics>,
    start_time: Instant,
}

// Metrics structure
#[derive(Clone)]
struct HelloMetrics {
    request_counter: Counter<u64>,
    request_duration: Histogram<f64>,
    active_connections: ObservableGauge<u64>,
    error_counter: Counter<u64>,
}

impl HelloMetrics {
    fn new(meter: Meter) -> Self {
        Self {
            request_counter: meter
                .u64_counter("hello_requests_total")
                .with_description("Total number of hello requests")
                .init(),
            request_duration: meter
                .f64_histogram("hello_request_duration_seconds")
                .with_description("Hello request duration in seconds")
                .init(),
            active_connections: meter
                .u64_observable_gauge("hello_active_connections")
                .with_description("Number of active connections")
                .init(),
            error_counter: meter
                .u64_counter("hello_errors_total")
                .with_description("Total number of hello errors")
                .init(),
        }
    }
}

#[tonic::async_trait]
impl HelloService for HelloServiceImpl {
    async fn say_hello(
        &self,
        request: Request<SayHelloRequest>,
    ) -> Result<Response<SayHelloResponse>, Status> {
        let start = Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        
        // Extract request data
        let req = request.into_inner();
        let name = req.name.as_deref().unwrap_or("World");
        let language = req.language.as_deref().unwrap_or("en");
        
        // Generate greeting message
        let message = match language {
            "es" => format!("¡Hola, {}!", name),
            "fr" => format!("Bonjour, {}!", name),
            "de" => format!("Hallo, {}!", name),
            "ja" => format!("こんにちは、{}さん！", name),
            "zh" => format!("你好，{}！", name),
            "ru" => format!("Привет, {}!", name),
            _ => format!("Hello, {}!", name),
        };
        
        // Record metrics
        self.metrics.request_counter.add(1, &[
            KeyValue::new("method", "say_hello"),
            KeyValue::new("language", language),
        ]);
        
        let duration = start.elapsed().as_secs_f64();
        self.metrics.request_duration.record(duration, &[
            KeyValue::new("method", "say_hello"),
            KeyValue::new("language", language),
        ]);
        
        // Create response
        let response = SayHelloResponse {
            message,
            language: language.to_string(),
            timestamp: Some(prost_types::Timestamp::from(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            )),
            metadata: Some(hello::ResponseMetadata {
                request_id,
                trace_id: "".to_string(), // TODO: Extract from context
                span_id: "".to_string(),  // TODO: Extract from context
                headers: HashMap::new(),
                timestamp: Some(prost_types::Timestamp::from(
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                )),
            }),
        };
        
        Ok(Response::new(response))
    }

    type SayHelloStreamStream = tonic::codec::Streaming<SayHelloResponse>;

    async fn say_hello_stream(
        &self,
        request: Request<SayHelloStreamRequest>,
    ) -> Result<Response<Self::SayHelloStreamStream>, Status> {
        let start = Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        
        // Extract request data
        let req = request.into_inner();
        let name = req.name.as_deref().unwrap_or("World");
        let language = req.language.as_deref().unwrap_or("en");
        let count = req.count.max(1).min(100); // Limit to 1-100
        let interval_ms = req.interval_ms.max(100).min(5000); // 100ms to 5s
        
        // Record metrics
        self.metrics.request_counter.add(1, &[
            KeyValue::new("method", "say_hello_stream"),
            KeyValue::new("language", language),
        ]);
        
        // Create streaming response
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let metrics = self.metrics.clone();
        
        tokio::spawn(async move {
            for i in 0..count {
                let message = match language {
                    "es" => format!("¡Hola, {}! Mensaje #{}", name, i + 1),
                    "fr" => format!("Bonjour, {}! Message #{}", name, i + 1),
                    "de" => format!("Hallo, {}! Nachricht #{}", name, i + 1),
                    "ja" => format!("こんにちは、{}さん！メッセージ #{}", name, i + 1),
                    "zh" => format!("你好，{}！消息 #{}", name, i + 1),
                    "ru" => format!("Привет, {}! Сообщение #{}", name, i + 1),
                    _ => format!("Hello, {}! Message #{}", name, i + 1),
                };
                
                let response = SayHelloResponse {
                    message,
                    language: language.to_string(),
                    timestamp: Some(prost_types::Timestamp::from(
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                    )),
                    metadata: Some(hello::ResponseMetadata {
                        request_id: request_id.clone(),
                        trace_id: "".to_string(),
                        span_id: "".to_string(),
                        headers: HashMap::new(),
                        timestamp: Some(prost_types::Timestamp::from(
                            SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                        )),
                    }),
                };
                
                if tx.send(response).await.is_err() {
                    break;
                }
                
                if i < count - 1 {
                    tokio::time::sleep(Duration::from_millis(interval_ms as u64)).await;
                }
            }
        });
        
        let duration = start.elapsed().as_secs_f64();
        self.metrics.request_duration.record(duration, &[
            KeyValue::new("method", "say_hello_stream"),
            KeyValue::new("language", language),
        ]);
        
        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(rx),
        )))
    }

    async fn health_check(
        &self,
        _request: Request<()>,
    ) -> Result<Response<HelloHealthCheckResponse>, Status> {
        let uptime = self.start_time.elapsed();
        let uptime_str = format!(
            "{}d {}h {}m {}s",
            uptime.as_secs() / 86400,
            (uptime.as_secs() % 86400) / 3600,
            (uptime.as_secs() % 3600) / 60,
            uptime.as_secs() % 60
        );
        
        let response = HelloHealthCheckResponse {
            status: HelloStatus::Serving as i32,
            message: "Service is healthy".to_string(),
            details: HashMap::new(),
            timestamp: Some(prost_types::Timestamp::from(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            )),
            service_info: Some(ServiceInfo {
                name: "hello-service".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                build_date: env!("VERGEN_BUILD_TIMESTAMP").to_string(),
                git_commit: env!("VERGEN_GIT_SHA").to_string(),
                uptime: uptime_str,
                environment: HashMap::new(),
            }),
        };
        
        Ok(Response::new(response))
    }

    async fn get_metrics(
        &self,
        _request: Request<()>,
    ) -> Result<Response<MetricsResponse>, Status> {
        let metrics = vec![
            hello::Metric {
                name: "hello_requests_total".to_string(),
                description: "Total number of hello requests".to_string(),
                unit: "requests".to_string(),
                r#type: hello::MetricType::Counter as i32,
                value: Some(hello::metric::Value::CounterValue(0.0)), // TODO: Get actual value
                labels: HashMap::new(),
                timestamp: Some(prost_types::Timestamp::from(
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                )),
            },
            hello::Metric {
                name: "hello_request_duration_seconds".to_string(),
                description: "Hello request duration in seconds".to_string(),
                unit: "seconds".to_string(),
                r#type: hello::MetricType::Histogram as i32,
                value: Some(hello::metric::Value::HistogramValue(
                    hello::HistogramData {
                        buckets: vec![0.1, 0.5, 1.0, 2.0, 5.0],
                        counts: vec![0, 0, 0, 0, 0], // TODO: Get actual counts
                        sum: 0.0,
                        count: 0,
                        min: 0.0,
                        max: 0.0,
                    },
                )),
                labels: HashMap::new(),
                timestamp: Some(prost_types::Timestamp::from(
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                )),
            },
        ];
        
        let response = MetricsResponse {
            metrics,
            timestamp: Some(prost_types::Timestamp::from(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            )),
            service_name: "hello-service".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        Ok(Response::new(response))
    }

    async fn echo(
        &self,
        request: Request<EchoRequest>,
    ) -> Result<Response<EchoResponse>, Status> {
        let start = Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();
        
        // Extract request data
        let req = request.into_inner();
        let mut message = req.message;
        let uppercase = req.uppercase;
        let repeat_count = req.repeat_count.max(1).min(10); // Limit to 1-10
        
        // Process message
        if uppercase {
            message = message.to_uppercase();
        }
        
        let final_message = message.repeat(repeat_count as usize);
        
        // Record metrics
        self.metrics.request_counter.add(1, &[
            KeyValue::new("method", "echo"),
            KeyValue::new("uppercase", uppercase.to_string()),
        ]);
        
        let duration = start.elapsed().as_secs_f64();
        self.metrics.request_duration.record(duration, &[
            KeyValue::new("method", "echo"),
            KeyValue::new("uppercase", uppercase.to_string()),
        ]);
        
        // Create response
        let response = EchoResponse {
            message: final_message.clone(),
            length: final_message.len() as i32,
            timestamp: Some(prost_types::Timestamp::from(
                SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            )),
            metadata: Some(hello::ResponseMetadata {
                request_id,
                trace_id: "".to_string(),
                span_id: "".to_string(),
                headers: req.headers,
                timestamp: Some(prost_types::Timestamp::from(
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
                )),
            }),
        };
        
        Ok(Response::new(response))
    }
}

// Health check implementation
#[derive(Default)]
pub struct HealthCheckService;

#[tonic::async_trait]
impl Health for HealthCheckService {
    async fn check(
        &self,
        _request: tonic::Request<tonic_health::HealthCheckRequest>,
    ) -> Result<tonic::Response<tonic_health::HealthCheckResponse>, tonic::Status> {
        Ok(tonic::Response::new(HealthCheckResponse {
            status: HealthCheckResponse_ServingStatus::Serving as i32,
        }))
    }

    async fn watch(
        &self,
        _request: tonic::Request<tonic_health::HealthCheckRequest>,
    ) -> Result<
        tonic::Response<tonic::streaming::Streaming<tonic_health::HealthCheckResponse>>,
        tonic::Status,
    > {
        Err(tonic::Status::unimplemented("Watch not implemented"))
    }
}

// Initialize OpenTelemetry
fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let tracer_provider = TracerProvider::builder()
        .with_simple_exporter(stdout::Exporter::default())
        .build();
    
    global::set_tracer_provider(tracer_provider);
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    // Initialize metrics
    let controller = controllers::basic(
        controllers::config()
            .with_collector_period(Duration::from_secs(1))
            .with_collector_timeout(Duration::from_millis(500)),
    )
    .build();
    
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(PeriodicReader::builder(controller, exporter()).build())
        .build();
    
    global::set_meter_provider(meter_provider);
    
    Ok(())
}

// Prometheus metrics endpoint
async fn metrics_handler(registry: Registry) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    
    Ok(warp::reply::with_header(
        String::from_utf8(buffer).unwrap(),
        "content-type",
        "text/plain; version=0.0.4; charset=utf-8",
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Initialize OpenTelemetry
    init_telemetry()?;
    
    let addr: SocketAddr = "[::1]:50051".parse()?;
    let service = HelloServiceImpl::default();
    let health_service = HealthCheckService::default();
    
    // Create reflection service
    let reflection_service = ReflectionBuilder::configure()
        .register_grpc_reflection_service()
        .build()?;
    
    // Create metrics registry
    let registry = Registry::new();
    
    // Start metrics server
    let metrics_addr: SocketAddr = "[::1]:9090".parse()?;
    let metrics_route = warp::path!("metrics").and(warp::get()).map(move || {
        let registry = registry.clone();
        async move { metrics_handler(registry).await }
    });
    
    tokio::spawn(async move {
        warp::serve(metrics_route).run(metrics_addr).await;
    });
    
    tracing::info!("🚀 Hello Service starting on {}", addr);
    tracing::info!("📊 Metrics available at http://{}:9090/metrics", addr.ip());
    tracing::info!("🔍 gRPC reflection enabled");
    tracing::info!("💚 Health checks enabled");
    
    // Start gRPC server
    Server::builder()
        .add_service(HelloServiceServer::new(service))
        .add_service(HealthServer::new(health_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;
    
    Ok(())
}
