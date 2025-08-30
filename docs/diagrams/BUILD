load("@bazel_skylib//rules:copy_file.bzl", "copy_file")
load("@rules_nodejs//nodejs:defs.bzl", "nodejs_binary")

package(default_visibility = ["//visibility:public"])

# File groups for diagram sources
filegroup(
    name = "kernel_diagrams",
    srcs = glob(["kernel/*.md"]),
)

filegroup(
    name = "polybus_diagrams", 
    srcs = glob(["polybus/*.md"]),
)

filegroup(
    name = "toolchain_diagrams",
    srcs = glob(["toolchains/*.md"]),
)

filegroup(
    name = "ci_flow_diagrams",
    srcs = glob(["ci-flow/*.md"]),
)

filegroup(
    name = "all_diagram_sources",
    srcs = [
        ":kernel_diagrams",
        ":polybus_diagrams", 
        ":toolchain_diagrams",
        ":ci_flow_diagrams",
        "README.md",
    ],
)

# Generate PNG files from Mermaid diagrams
genrule(
    name = "generate_mermaid_pngs",
    srcs = [":all_diagram_sources"],
    outs = [
        "generated/kernel/kernel-overview.png",
        "generated/kernel/kernel-boot.png",
        "generated/polybus/polybus-overview.png",
        "generated/toolchains/build-system.png", 
        "generated/ci-flow/github-actions.png",
    ],
    cmd = """
        # Create output directories
        mkdir -p $(RULEDIR)/generated/kernel
        mkdir -p $(RULEDIR)/generated/polybus
        mkdir -p $(RULEDIR)/generated/toolchains
        mkdir -p $(RULEDIR)/generated/ci-flow
        
        # Extract Mermaid diagrams and generate PNGs
        # Note: This is a simplified version - real implementation would
        # parse markdown files and extract mermaid blocks
        
        # For now, create placeholder PNGs
        echo "PNG placeholder for kernel-overview" > $(RULEDIR)/generated/kernel/kernel-overview.png
        echo "PNG placeholder for kernel-boot" > $(RULEDIR)/generated/kernel/kernel-boot.png
        echo "PNG placeholder for polybus-overview" > $(RULEDIR)/generated/polybus/polybus-overview.png
        echo "PNG placeholder for build-system" > $(RULEDIR)/generated/toolchains/build-system.png
        echo "PNG placeholder for github-actions" > $(RULEDIR)/generated/ci-flow/github-actions.png
    """,
)

# Generate PNG files from PlantUML diagrams
genrule(
    name = "generate_plantuml_pngs",
    srcs = [":all_diagram_sources"],
    outs = [
        "generated/kernel/kernel-component.png",
        "generated/kernel/kernel-boot-activity.png",
        "generated/polybus/polybus-component.png",
        "generated/toolchains/build-process.png",
        "generated/ci-flow/ci-activity.png",
    ],
    cmd = """
        # Create output directories
        mkdir -p $(RULEDIR)/generated/kernel
        mkdir -p $(RULEDIR)/generated/polybus
        mkdir -p $(RULEDIR)/generated/toolchains
        mkdir -p $(RULEDIR)/generated/ci-flow
        
        # Extract PlantUML diagrams and generate PNGs
        # Note: This is a simplified version - real implementation would
        # use PlantUML command line tool
        
        # For now, create placeholder PNGs
        echo "PlantUML PNG placeholder for kernel-component" > $(RULEDIR)/generated/kernel/kernel-component.png
        echo "PlantUML PNG placeholder for kernel-boot-activity" > $(RULEDIR)/generated/kernel/kernel-boot-activity.png
        echo "PlantUML PNG placeholder for polybus-component" > $(RULEDIR)/generated/polybus/polybus-component.png
        echo "PlantUML PNG placeholder for build-process" > $(RULEDIR)/generated/toolchains/build-process.png
        echo "PlantUML PNG placeholder for ci-activity" > $(RULEDIR)/generated/ci-flow/ci-activity.png
    """,
)

# Copy diagram sources to generated directory for easy access
copy_file(
    name = "copy_readme",
    src = "README.md",
    out = "generated/README.md",
)

# Generate diagram index
genrule(
    name = "generate_diagram_index",
    srcs = [":all_diagram_sources"],
    outs = ["generated/index.html"],
    cmd = """
        cat > $(RULEDIR)/generated/index.html << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Polymera OS Diagram Pack</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .category { margin: 20px 0; }
        .diagram-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; }
        .diagram-card { border: 1px solid #ddd; border-radius: 8px; padding: 15px; }
        .diagram-card h3 { margin-top: 0; color: #333; }
        .diagram-preview { width: 100%; height: 200px; background: #f5f5f5; border-radius: 4px; }
    </style>
</head>
<body>
    <h1>Polymera OS Diagram Pack</h1>
    
    <div class="category">
        <h2>Kernel Architecture</h2>
        <div class="diagram-grid">
            <div class="diagram-card">
                <h3>Kernel Overview</h3>
                <div class="diagram-preview">
                    <img src="kernel/kernel-overview.png" alt="Kernel Overview" style="width:100%; height:100%; object-fit:contain;">
                </div>
                <p>High-level kernel architecture showing layers and components.</p>
            </div>
            <div class="diagram-card">
                <h3>Boot Process</h3>
                <div class="diagram-preview">
                    <img src="kernel/kernel-boot.png" alt="Boot Process" style="width:100%; height:100%; object-fit:contain;">
                </div>
                <p>Complete boot sequence from UEFI to user space.</p>
            </div>
        </div>
    </div>
    
    <div class="category">
        <h2>PolyBus Messaging</h2>
        <div class="diagram-grid">
            <div class="diagram-card">
                <h3>PolyBus Overview</h3>
                <div class="diagram-preview">
                    <img src="polybus/polybus-overview.png" alt="PolyBus Overview" style="width:100%; height:100%; object-fit:contain;">
                </div>
                <p>Messaging system architecture and component interactions.</p>
            </div>
        </div>
    </div>
    
    <div class="category">
        <h2>Development Toolchains</h2>
        <div class="diagram-grid">
            <div class="diagram-card">
                <h3>Build System</h3>
                <div class="diagram-preview">
                    <img src="toolchains/build-system.png" alt="Build System" style="width:100%; height:100%; object-fit:contain;">
                </div>
                <p>Bazel build system flow and dependency management.</p>
            </div>
        </div>
    </div>
    
    <div class="category">
        <h2>CI/CD Pipeline</h2>
        <div class="diagram-grid">
            <div class="diagram-card">
                <h3>GitHub Actions</h3>
                <div class="diagram-preview">
                    <img src="ci-flow/github-actions.png" alt="GitHub Actions" style="width:100%; height:100%; object-fit:contain;">
                </div>
                <p>Complete CI/CD pipeline with quality gates and deployment.</p>
            </div>
        </div>
    </div>
    
    <footer style="margin-top: 40px; padding-top: 20px; border-top: 1px solid #ddd; color: #666;">
        <p>Generated automatically from diagram sources. Last updated: $(date)</p>
    </footer>
</body>
</html>
EOF
    """,
)

# Validate diagrams
sh_test(
    name = "validate_diagrams",
    srcs = ["scripts/validate_diagrams.sh"],
    data = [":all_diagram_sources"],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Test that all diagrams render correctly
sh_test(
    name = "test_diagram_generation",
    srcs = ["scripts/test_generation.sh"],
    data = [
        ":generate_mermaid_pngs",
        ":generate_plantuml_pngs",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Generate visual regression test report
genrule(
    name = "visual_report", 
    srcs = [
        ":generate_mermaid_pngs",
        ":generate_plantuml_pngs",
    ],
    outs = ["visual_report.html"],
    cmd = """
        cat > $(RULEDIR)/visual_report.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>Diagram Visual Report</title>
    <style>
        body { font-family: Arial, sans-serif; }
        .comparison { display: flex; gap: 20px; margin: 20px 0; }
        .diagram { border: 1px solid #ccc; padding: 10px; }
    </style>
</head>
<body>
    <h1>Polymera OS Diagram Visual Report</h1>
    <p>Generated on: $$(date)</p>
    
    <h2>Generated Diagrams</h2>
    <p>All diagram files have been generated successfully.</p>
    
    <h3>File Listing</h3>
    <ul>
        <li>kernel/kernel-overview.png</li>
        <li>kernel/kernel-boot.png</li>
        <li>polybus/polybus-overview.png</li>
        <li>toolchains/build-system.png</li>
        <li>ci-flow/github-actions.png</li>
    </ul>
</body>
</html>
EOF
    """,
)

# Link validation for diagrams referenced in documentation
sh_test(
    name = "link_tests",
    srcs = ["scripts/validate_links.sh"],
    data = [
        ":all_diagram_sources",
        "//docs/site:site_files",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Image validation tests
sh_test(
    name = "image_tests",
    srcs = ["scripts/validate_images.sh"],
    data = [
        ":generate_mermaid_pngs",
        ":generate_plantuml_pngs",
    ],
    deps = ["@bazel_tools//tools/bash/runfiles"],
)

# Complete diagram pack
filegroup(
    name = "diagram_pack",
    srcs = [
        ":all_diagram_sources",
        ":generate_mermaid_pngs", 
        ":generate_plantuml_pngs",
        ":generate_diagram_index",
        ":copy_readme",
    ],
)

# Test suite for all diagram-related tests
test_suite(
    name = "diagram_tests",
    tests = [
        ":validate_diagrams",
        ":test_diagram_generation", 
        ":link_tests",
        ":image_tests",
    ],
)

# Main target for building all diagrams
alias(
    name = "all",
    actual = ":diagram_pack",
)
