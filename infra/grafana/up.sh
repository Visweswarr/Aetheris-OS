#!/bin/bash

set -e

echo "🚀 Starting Polymera OS Monitoring Stack..."

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

# Check if Docker Compose is available
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install it and try again."
    exit 1
fi

# Create necessary directories
echo "📁 Creating necessary directories..."
mkdir -p grafana/dashboards
mkdir -p grafana/provisioning/datasources
mkdir -p grafana/provisioning/dashboards
mkdir -p rules

# Copy dashboard files
echo "📊 Setting up Grafana dashboards..."
cp grafana/provisioning/dashboards/polymera-overview.json grafana/dashboards/

# Start the monitoring stack
echo "🐳 Starting Docker containers..."
docker-compose up -d

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 30

# Health checks
echo "🏥 Performing health checks..."

# Check OpenTelemetry Collector
if curl -f http://localhost:8888/ > /dev/null 2>&1; then
    echo "✅ OpenTelemetry Collector is healthy"
else
    echo "❌ OpenTelemetry Collector health check failed"
fi

# Check Prometheus
if curl -f http://localhost:9090/-/healthy > /dev/null 2>&1; then
    echo "✅ Prometheus is healthy"
else
    echo "❌ Prometheus health check failed"
fi

# Check Grafana
if curl -f http://localhost:3000/api/health > /dev/null 2>&1; then
    echo "✅ Grafana is healthy"
else
    echo "❌ Grafana health check failed"
fi

# Check Jaeger
if curl -f http://localhost:16686/ > /dev/null 2>&1; then
    echo "✅ Jaeger is healthy"
else
    echo "❌ Jaeger health check failed"
fi

# Check Node Exporter
if curl -f http://localhost:9100/metrics > /dev/null 2>&1; then
    echo "✅ Node Exporter is healthy"
else
    echo "❌ Node Exporter health check failed"
fi

echo ""
echo "🎉 Monitoring stack is running!"
echo ""
echo "📊 Access URLs:"
echo "   Grafana:        http://localhost:3000 (admin/polymera123)"
echo "   Prometheus:     http://localhost:9090"
echo "   Jaeger:         http://localhost:16686"
echo "   Node Exporter:  http://localhost:9100/metrics"
echo ""
echo "🔧 Configuration:"
echo "   OpenTelemetry Collector: http://localhost:4317 (gRPC), http://localhost:4318 (HTTP)"
echo "   Prometheus Metrics:      http://localhost:8889"
echo ""
echo "📈 Next steps:"
echo "   1. Open Grafana at http://localhost:3000"
echo "   2. Login with admin/polymera123"
echo "   3. The Polymera OS dashboards should be automatically provisioned"
echo "   4. Configure your services to send metrics to the OpenTelemetry Collector"
echo ""
echo "🛑 To stop the stack, run: docker-compose down"
echo "🔄 To restart, run: docker-compose restart"
