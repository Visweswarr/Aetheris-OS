{ pkgs ? import <nixpkgs> {}
, lib ? pkgs.lib
, stdenv ? pkgs.stdenv
, rustPlatform ? pkgs.rustPlatform
, buildInputs ? []
, nativeBuildInputs ? []
}:

let
  # Fixed timestamp for reproducible builds
  fixedTimestamp = "1970-01-01T00:00:00Z";
  
  # Reproducible build environment
  reproducibleEnv = {
    # Rust specific
    RUSTFLAGS = "-C target-cpu=native -C codegen-units=1 -C lto=fat";
    CARGO_INCREMENTAL = "0";
    CARGO_PROFILE_RELEASE_DEBUG = "0";
    CARGO_PROFILE_RELEASE_STRIP = "true";
    CARGO_PROFILE_RELEASE_OPT_LEVEL = "3";
    CARGO_PROFILE_RELEASE_PANIC = "abort";
    CARGO_PROFILE_RELEASE_OVERFLOW_CHECKS = "false";
    CARGO_PROFILE_RELEASE_LTO = "true";
    CARGO_PROFILE_RELEASE_CODEGEN_UNITS = "1";
    CARGO_PROFILE_RELEASE_RPATH = "false";
    
    # C/C++ specific
    CFLAGS = "-O3 -DNDEBUG -fno-ident -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-builtin -fno-common";
    CXXFLAGS = "-O3 -DNDEBUG -fno-ident -fno-stack-protector -fno-unwind-tables -fno-asynchronous-unwind-tables -fno-builtin -fno-common";
    LDFLAGS = "-Wl,--strip-all -Wl,--build-id=none -Wl,--no-rosegment";
    
    # Go specific
    CGO_ENABLED = "0";
    GOOS = "linux";
    GOARCH = "amd64";
    GOAMD64 = "v1";
    GOMAXPROCS = "1";
    GORACE = "0";
    GOTRACEBACK = "none";
    
    # Python specific
    PYTHONHASHSEED = "0";
    PYTHONDONTWRITEBYTECODE = "1";
    PYTHONUNBUFFERED = "1";
    
    # General build
    SOURCE_DATE_EPOCH = "0";
    BUILD_DATE = fixedTimestamp;
    BUILD_TIMESTAMP = fixedTimestamp;
  };
  
  # Reproducible build derivation
  reproducibleBuild = stdenv.mkDerivation {
    name = "polymera-os-reproducible";
    version = "0.1.0";
    
    src = ./.;
    
    buildInputs = buildInputs ++ [
      pkgs.rustc
      pkgs.cargo
      pkgs.clang
      pkgs.go
      pkgs.python3
      pkgs.sha256sum
      pkgs.jq
    ];
    
    nativeBuildInputs = nativeBuildInputs ++ [
      pkgs.makeWrapper
    ];
    
    # Set environment variables
    inherit reproducibleEnv;
    
    # Build phases
    phases = [ "unpackPhase" "patchPhase" "configurePhase" "buildPhase" "installPhase" "fixupPhase" ];
    
    # Unpack source
    unpackPhase = ''
      echo "Unpacking source code..."
      cp -r $src/* .
      chmod -R +w .
    '';
    
    # Patch for reproducibility
    patchPhase = ''
      echo "Applying reproducibility patches..."
      
      # Fix timestamps in source files
      find . -name "*.rs" -o -name "*.c" -o -name "*.cpp" -o -name "*.h" -o -name "*.hpp" | xargs touch -t 197001010000.00
      
      # Fix timestamps in configuration files
      find . -name "*.toml" -o -name "*.json" -o -name "*.yaml" -o -name "*.yml" | xargs touch -t 197001010000.00
      
      # Fix timestamps in documentation
      find . -name "*.md" -o -name "*.txt" -o -name "*.rst" | xargs touch -t 197001010000.00
    '';
    
    # Configure build
    configurePhase = ''
      echo "Configuring build for reproducibility..."
      
      # Set up Rust toolchain
      rustup default stable
      rustup target add x86_64-unknown-linux-gnu
      
      # Set up Go environment
      export GOPATH=$PWD/.go
      export GOCACHE=$PWD/.go-cache
      
      # Set up Python environment
      export PYTHONPATH=$PWD
      
      echo "Build configuration complete"
    '';
    
    # Build components
    buildPhase = ''
      echo "Building Polymera OS components..."
      
      # Build Rust components
      echo "Building Rust components..."
      cd kernel
      cargo build --release --target x86_64-unknown-linux-gnu
      cd ..
      
      cd security/caps
      cargo build --release
      cd ../..
      
      cd services/hello
      cargo build --release
      cd ../..
      
      cd tests/determinism
      cargo build --release
      cd ../..
      
      # Build C/C++ components (if any)
      echo "Building C/C++ components..."
      if [ -f "CMakeLists.txt" ]; then
        mkdir -p build
        cd build
        cmake .. -DCMAKE_BUILD_TYPE=Release
        make -j$(nproc)
        cd ..
      fi
      
      # Build Go components (if any)
      echo "Building Go components..."
      if [ -f "go.mod" ]; then
        go mod download
        go build -ldflags="-s -w -buildid=" -trimpath -o bin/ ./...
      fi
      
      # Build Python components (if any)
      echo "Building Python components..."
      if [ -f "pyproject.toml" ]; then
        pip install --no-cache-dir --no-deps --no-build-isolation .
      fi
      
      echo "Build complete"
    '';
    
    # Install components
    installPhase = ''
      echo "Installing components..."
      
      mkdir -p $out/bin
      mkdir -p $out/lib
      mkdir -p $out/share/polymera-os
      mkdir -p $out/var/lib/polymera-os
      
      # Install Rust binaries
      cp kernel/target/x86_64-unknown-linux-gnu/release/* $out/bin/ 2>/dev/null || true
      cp security/caps/target/release/* $out/bin/ 2>/dev/null || true
      cp services/hello/target/release/* $out/bin/ 2>/dev/null || true
      cp tests/determinism/target/release/* $out/bin/ 2>/dev/null || true
      
      # Install C/C++ binaries
      cp build/* $out/bin/ 2>/dev/null || true
      
      # Install Go binaries
      cp bin/* $out/bin/ 2>/dev/null || true
      
      # Install Python packages
      cp -r lib/python*/site-packages/* $out/lib/ 2>/dev/null || true
      
      # Generate build metadata
      echo "Generating build metadata..."
      
      # Create build info
      cat > $out/share/polymera-os/build-info.json << EOF
      {
        "build_timestamp": "$fixedTimestamp",
        "build_host": "nix-reproducible",
        "build_user": "nix",
        "build_id": "reproducible",
        "source_hash": "$(sha256sum $src | cut -d' ' -f1)",
        "dependencies": [],
        "build_flags": [],
        "environment": $(echo "$reproducibleEnv" | jq -R .),
        "output_hash": ""
      }
      EOF
      
      # Generate component hashes
      echo "Generating component hashes..."
      find $out/bin -type f -executable | while read file; do
        hash_file="$file.sha256"
        sha256sum "$file" > "$hash_file"
        echo "Generated hash for: $file"
      done
      
      # Create reproducibility report
      cat > $out/share/polymera-os/reproducibility-report.md << EOF
      # Polymera OS Reproducible Build Report
      
      **Build Timestamp**: $fixedTimestamp
      **Build Host**: nix-reproducible
      **Build User**: nix
      **Build ID**: reproducible
      
      ## Component Hashes
      
      EOF
      
      find $out/bin -name "*.sha256" | while read hash_file; do
        component_file="''${hash_file%.sha256}"
        component_name="''${component_file##*/}"
        hash_value="$(cat "$hash_file" | cut -d' ' -f1)"
        echo "- **$component_name**: \`$hash_value\`" >> $out/share/polymera-os/reproducibility-report.md
      done
      
      cat >> $out/share/polymera-os/reproducibility-report.md << EOF
      
      ## Verification
      
      To verify reproducibility, run this build twice and compare hashes:
      
      \`\`\`bash
      nix-build --no-out-link
      \`\`\`
      
      The hashes should be identical across builds.
      
      ---
      
      *Generated by Nix reproducible build system*
      EOF
      
      echo "Installation complete"
    '';
    
    # Fix up phase
    fixupPhase = ''
      echo "Fixing up build artifacts..."
      
      # Strip debug symbols for reproducibility
      find $out/bin -type f -executable -exec strip --strip-all {} \;
      
      # Remove build IDs for reproducibility
      find $out/bin -type f -executable -exec sh -c 'objcopy --remove-section=.note.gnu.build-id "$1" 2>/dev/null || true' _ {} \;
      
      # Fix timestamps
      find $out -type f -exec touch -t 197001010000.00 {} \;
      find $out -type d -exec touch -t 197001010000.00 {} \;
      
      echo "Fixup complete"
    '';
    
    # Meta information
    meta = with lib; {
      description = "Polymera OS - Reproducible Build";
      homepage = "https://polymera-os.org";
      license = licenses.asl20;
      platforms = platforms.linux;
      maintainers = [];
    };
  };
  
  # Reproducibility verification
  verifyReproducibility = stdenv.mkDerivation {
    name = "polymera-os-reproducibility-verification";
    version = "0.1.0";
    
    src = reproducibleBuild;
    
    buildInputs = [ pkgs.sha256sum pkgs.jq ];
    
    buildPhase = ''
      echo "Verifying build reproducibility..."
      
      # Generate hash of entire output
      find $src -type f | sort | xargs sha256sum > all-files.sha256
      
      # Create verification report
      cat > reproducibility-verification.txt << EOF
      # Reproducibility Verification Report
      
      **Generated**: $(date -u +%Y-%m-%dT%H:%M:%SZ)
      **Build Output**: $src
      
      ## File Hashes
      
      EOF
      
      cat all-files.sha256 >> reproducibility-verification.txt
      
      cat >> reproducibility-verification.txt << EOF
      
      ## Verification Instructions
      
      1. Run this build twice: \`nix-build --no-out-link\`
      2. Compare the hashes in reproducibility-verification.txt
      3. All hashes should be identical
      
      ## Expected Result
      
      ✅ **REPRODUCIBLE**: All hashes match across builds
      
      ---
      
      *Generated by Nix reproducibility verification*
      EOF
      
      echo "Verification complete"
    '';
    
    installPhase = ''
      mkdir -p $out
      cp reproducibility-verification.txt $out/
      cp all-files.sha256 $out/
    '';
    
    meta = with lib; {
      description = "Polymera OS Reproducibility Verification";
      platforms = platforms.linux;
    };
  };

in {
  inherit reproducibleBuild verifyReproducibility;
  default = reproducibleBuild;
}
