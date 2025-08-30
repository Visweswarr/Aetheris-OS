{ lib
, stdenv
, fetchurl
, p7zip
, makeWrapper
, qemu
}:

let
  # OVMF firmware versions and URLs
  ovmfVersions = {
    x86_64 = {
      version = "202311";
      url = "https://github.com/edk2/edk2/releases/download/edk2-stable${version}/OVMF-X64.zip";
      sha256 = "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA="; # Placeholder
      arch = "x86_64";
    };
    aarch64 = {
      version = "202311";
      url = "https://github.com/edk2/edk2/releases/download/edk2-stable${version}/OVMF-AA64.zip";
      sha256 = "sha256-BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB="; # Placeholder
      arch = "aarch64";
    };
  };

  # Build OVMF derivation for a specific architecture
  buildOVMF = arch: let
    version = ovmfVersions.${arch};
  in stdenv.mkDerivation rec {
    pname = "ovmf-${arch}";
    version = version.version;
    
    src = fetchurl {
      url = version.url;
      sha256 = version.sha256;
    };
    
    nativeBuildInputs = [ p7zip makeWrapper ];
    
    buildInputs = [ qemu ];
    
    # Extract the ZIP file
    unpackPhase = ''
      runHook preUnpack
      7z x $src
      runHook postUnpack
    '';
    
    # Install OVMF files
    installPhase = ''
      runHook preInstall
      
      # Create output directory
      mkdir -p $out/share/ovmf/${arch}
      
      # Copy OVMF files
      cp -r * $out/share/ovmf/${arch}/
      
      # Create symlinks for common names
      ln -sf $out/share/ovmf/${arch}/OVMF_CODE.fd $out/share/ovmf/${arch}/OVMF.fd
      ln -sf $out/share/ovmf/${arch}/OVMF_VARS.fd $out/share/ovmf/${arch}/OVMF_VARS.fd
      
      # Create convenience scripts
      mkdir -p $out/bin
      
      # QEMU runner script for this architecture
      cat > $out/bin/qemu-run-${arch} << 'EOF'
      #!/bin/bash
      
      # QEMU runner for ${arch} with OVMF
      OVMF_CODE="$out/share/ovmf/${arch}/OVMF_CODE.fd"
      OVMF_VARS="$out/share/ovmf/${arch}/OVMF_VARS.fd"
      
      if [ ! -f "$OVMF_CODE" ]; then
        echo "Error: OVMF firmware not found at $OVMF_CODE"
        exit 1
      fi
      
      if [ ! -f "$OVMF_VARS" ]; then
        echo "Error: OVMF variables not found at $OVMF_VARS"
        exit 1
      fi
      
      echo "Using OVMF firmware: $OVMF_CODE"
      echo "Using OVMF variables: $OVMF_VARS"
      
      # Run QEMU with OVMF
      exec qemu-system-${arch} \
        -firmware "$OVMF_CODE" \
        -drive file="$OVMF_VARS",if=pflash,format=raw,readonly=off \
        "$@"
      EOF
      
      chmod +x $out/bin/qemu-run-${arch}
      
      # Create wrapper for qemu-system-${arch}
      makeWrapper ${qemu}/bin/qemu-system-${arch} $out/bin/qemu-system-${arch}-ovmf \
        --add-flags "-firmware $out/share/ovmf/${arch}/OVMF_CODE.fd" \
        --add-flags "-drive file=$out/share/ovmf/${arch}/OVMF_VARS.fd,if=pflash,format=raw,readonly=off"
      
      runHook postInstall
    '';
    
    # Create a working directory for OVMF variables
    postFixup = ''
      mkdir -p $out/working
      cp $out/share/ovmf/${arch}/OVMF_VARS.fd $out/working/OVMF_VARS.fd
      chmod 644 $out/working/OVMF_VARS.fd
    '';
    
    meta = with lib; {
      description = "OVMF UEFI firmware for ${arch} virtualization";
      homepage = "https://github.com/edk2/edk2";
      license = licenses.bsd2;
      platforms = platforms.linux;
      maintainers = with maintainers; [ ];
    };
  };

in {
  # Export OVMF derivations for both architectures
  x86_64 = buildOVMF "x86_64";
  aarch64 = buildOVMF "aarch64";
  
  # Combined package with both architectures
  all = stdenv.mkDerivation {
    pname = "ovmf-all";
    version = "202311";
    
    buildInputs = [ makeWrapper ];
    
    installPhase = ''
      runHook preInstall
      
      # Create output directory
      mkdir -p $out
      
      # Copy x86_64 OVMF
      cp -r ${buildOVMF "x86_64"}/* $out/
      
      # Copy aarch64 OVMF
      mkdir -p $out/share/ovmf/aarch64
      cp -r ${buildOVMF "aarch64"}/share/ovmf/aarch64/* $out/share/ovmf/aarch64/
      
      # Create convenience scripts
      mkdir -p $out/bin
      
      # Universal QEMU runner
      cat > $out/bin/qemu-run << 'EOF'
      #!/bin/bash
      
      # Detect architecture
      ARCH=$(uname -m)
      
      case $ARCH in
        x86_64)
          exec $out/bin/qemu-run-x86_64 "$@"
          ;;
        aarch64|arm64)
          exec $out/bin/qemu-run-aarch64 "$@"
          ;;
        *)
          echo "Unsupported architecture: $ARCH"
          exit 1
          ;;
      esac
      EOF
      
      chmod +x $out/bin/qemu-run
      
      # Create symlinks
      ln -sf $out/bin/qemu-run-x86_64 $out/bin/qemu-run-x64
      ln -sf $out/bin/qemu-run-aarch64 $out/bin/qemu-run-arm64
      
      runHook postInstall
    '';
    
    meta = with lib; {
      description = "OVMF UEFI firmware for x86_64 and aarch64 virtualization";
      homepage = "https://github.com/edk2/edk2";
      license = licenses.bsd2;
      platforms = platforms.linux;
      maintainers = with maintainers; [ ];
    };
  };
}
