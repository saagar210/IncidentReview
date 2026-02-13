#!/bin/bash
# Profile metrics computation and dashboard rendering performance
# Runs stress tests and reports performance metrics

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

echo "======================================================================"
echo "IncidentReview Performance Profiling Suite"
echo "======================================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Ollama is required (for AI tests)
OLLAMA_REQUIRED=false
if [[ "$1" == "--with-ai" ]]; then
    OLLAMA_REQUIRED=true
fi

echo "Building qir_core..."
cargo build -p qir_core --tests --release 2>&1 | grep -E "^(Compiling|Finished)" || true
echo ""

if [ "$OLLAMA_REQUIRED" = true ]; then
    echo "Checking Ollama availability..."
    if curl -s http://127.0.0.1:11434/api/tags > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Ollama is running${NC}"
    else
        echo -e "${YELLOW}✗ Ollama not running on localhost:11434${NC}"
        echo "  Skipping AI tests. Start Ollama and run with --with-ai flag."
        OLLAMA_REQUIRED=false
    fi
fi
echo ""

echo "======================================================================"
echo "Phase 1: Metrics Computation (1K incidents)"
echo "======================================================================"
echo ""
echo "Running: cargo test --test stress_large_dataset stress_test_metrics_computation_1k --ignored --release -- --nocapture"
time cargo test -p qir_core --test stress_large_dataset stress_test_metrics_computation_1k_incidents --ignored --release -- --nocapture 2>&1 | grep -E "(metrics|duration|assert)" || true
echo ""

echo "======================================================================"
echo "Phase 2: Metrics Computation (10K incidents)"
echo "======================================================================"
echo ""
echo "Running: cargo test --test stress_large_dataset stress_test_metrics_computation_10k --ignored --release -- --nocapture"
time cargo test -p qir_core --test stress_large_dataset stress_test_metrics_computation_10k_incidents --ignored --release -- --nocapture 2>&1 | grep -E "(metrics|duration|assert)" || true
echo ""

echo "======================================================================"
echo "Phase 3: Dashboard Rendering (1K incidents)"
echo "======================================================================"
echo ""
echo "Running: cargo test --test stress_large_dataset stress_test_dashboard_rendering_1k --ignored --release -- --nocapture"
time cargo test -p qir_core --test stress_large_dataset stress_test_dashboard_rendering_1k_incidents --ignored --release -- --nocapture 2>&1 | grep -E "(dashboard|duration|assert)" || true
echo ""

echo "======================================================================"
echo "Phase 4: Dashboard Rendering (10K incidents)"
echo "======================================================================"
echo ""
echo "Running: cargo test --test stress_large_dataset stress_test_dashboard_rendering_10k --ignored --release -- --nocapture"
time cargo test -p qir_core --test stress_large_dataset stress_test_dashboard_rendering_10k_incidents --ignored --release -- --nocapture 2>&1 | grep -E "(dashboard|duration|assert)" || true
echo ""

echo "======================================================================"
echo "Phase 5: Report Generation (1K incidents)"
echo "======================================================================"
echo ""
echo "Running: cargo test --test stress_large_dataset stress_test_report_generation_1k --ignored --release -- --nocapture"
time cargo test -p qir_core --test stress_large_dataset stress_test_report_generation_1k_incidents --ignored --release -- --nocapture 2>&1 | grep -E "(report|duration|assert)" || true
echo ""

echo "======================================================================"
echo "Phase 6: Report Generation (10K incidents)"
echo "======================================================================"
echo ""
echo "Running: cargo test --test stress_large_dataset stress_test_report_generation_10k --ignored --release -- --nocapture"
time cargo test -p qir_core --test stress_large_dataset stress_test_report_generation_10k_incidents --ignored --release -- --nocapture 2>&1 | grep -E "(report|duration|assert)" || true
echo ""

echo "======================================================================"
echo "Phase 7: Memory Usage Estimation (10K incidents)"
echo "======================================================================"
echo ""
echo "Running: cargo test --test stress_large_dataset stress_test_memory_usage_10k --ignored --release -- --nocapture"
cargo test -p qir_core --test stress_large_dataset stress_test_memory_usage_10k_incidents --ignored --release -- --nocapture 2>&1 | grep -E "(estimated|memory|bytes)" || true
echo ""

if [ "$OLLAMA_REQUIRED" = true ]; then
    echo "======================================================================"
    echo "Phase 8: AI Embeddings (10K chunks)"
    echo "======================================================================"
    echo ""
    echo "Running: cargo test --test stress_large_embeddings stress_test_embedding_index_build_10k --ignored --release -- --nocapture"
    time cargo test -p qir_ai --test stress_large_embeddings stress_test_embedding_index_build_10k_chunks --ignored --release -- --nocapture 2>&1 | grep -E "(index|duration|assert)" || true
    echo ""
fi

echo "======================================================================"
echo "Performance Summary"
echo "======================================================================"
echo ""
echo -e "${GREEN}Expected Performance Targets:${NC}"
echo "  • Metrics (1K incidents): <2 seconds"
echo "  • Metrics (10K incidents): <5 seconds"
echo "  • Dashboard (1K incidents): <1 second"
echo "  • Dashboard (10K incidents): <3 seconds"
echo "  • Report (1K incidents): <5 seconds"
echo "  • Report (10K incidents): <10 seconds"
echo "  • Memory (10K incidents): <50 MB"
if [ "$OLLAMA_REQUIRED" = true ]; then
    echo "  • Embeddings (10K chunks): <30 seconds"
fi
echo ""
echo -e "${YELLOW}Note: Times vary by machine. Above are recommendations for modern MacBook.${NC}"
echo ""
echo "======================================================================"
echo "Profiling complete!"
echo "======================================================================"
