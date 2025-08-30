use criterion::{black_box, criterion_group, criterion_main, Criterion};
use polymera_crypto::{
    kyber::{KyberKem, KyberParameterSet},
    dilithium::{Dilithium, DilithiumParameterSet},
    utils,
};

fn benchmark_kyber_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Kyber KEM Operations");

    // Benchmark key generation
    for params in [
        KyberParameterSet::Kyber512,
        KyberParameterSet::Kyber768,
        KyberParameterSet::Kyber1024,
    ] {
        group.bench_function(
            &format!("{}_key_generation", params.algorithm_name()),
            |b| {
                b.iter(|| {
                    let _ = KyberKem::generate_keypair(black_box(params));
                });
            },
        );
    }

    // Benchmark encapsulation and decapsulation
    for params in [
        KyberParameterSet::Kyber512,
        KyberParameterSet::Kyber768,
        KyberParameterSet::Kyber1024,
    ] {
        let (public_key, secret_key) = KyberKem::generate_keypair(params).unwrap();

        group.bench_function(
            &format!("{}_encapsulation", params.algorithm_name()),
            |b| {
                b.iter(|| {
                    let _ = KyberKem::encapsulate(black_box(&public_key));
                });
            },
        );

        let (ciphertext, _) = KyberKem::encapsulate(&public_key).unwrap();

        group.bench_function(
            &format!("{}_decapsulation", params.algorithm_name()),
            |b| {
                b.iter(|| {
                    let _ = KyberKem::decapsulate(black_box(&secret_key), black_box(&ciphertext));
                });
            },
        );
    }

    group.finish();
}

fn benchmark_dilithium_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("Dilithium Signature Operations");

    // Benchmark key generation
    for params in [
        DilithiumParameterSet::Dilithium2,
        DilithiumParameterSet::Dilithium3,
        DilithiumParameterSet::Dilithium5,
    ] {
        group.bench_function(
            &format!("{}_key_generation", params.algorithm_name()),
            |b| {
                b.iter(|| {
                    let _ = Dilithium::generate_keypair(black_box(params));
                });
            },
        );
    }

    // Benchmark signing and verification
    for params in [
        DilithiumParameterSet::Dilithium2,
        DilithiumParameterSet::Dilithium3,
        DilithiumParameterSet::Dilithium5,
    ] {
        let (public_key, secret_key) = Dilithium::generate_keypair(params).unwrap();
        let message = b"This is a test message for benchmarking digital signatures";

        group.bench_function(
            &format!("{}_signing", params.algorithm_name()),
            |b| {
                b.iter(|| {
                    let _ = Dilithium::sign(black_box(&secret_key), black_box(message));
                });
            },
        );

        let signature = Dilithium::sign(&secret_key, message).unwrap();

        group.bench_function(
            &format!("{}_verification", params.algorithm_name()),
            |b| {
                b.iter(|| {
                    let _ = Dilithium::verify(black_box(&public_key), black_box(message), black_box(&signature));
                });
            },
        );
    }

    group.finish();
}

fn benchmark_utility_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("Utility Functions");

    // Benchmark random bytes generation
    group.bench_function("random_bytes_32", |b| {
        b.iter(|| {
            let _ = utils::random_bytes(black_box(32));
        });
    });

    group.bench_function("random_bytes_64", |b| {
        b.iter(|| {
            let _ = utils::random_bytes(black_box(64));
        });
    });

    group.bench_function("random_bytes_128", |b| {
        b.iter(|| {
            let _ = utils::random_bytes(black_box(128));
        });
    });

    // Benchmark constant time comparison
    let data1 = vec![0x42u8; 64];
    let data2 = vec![0x42u8; 64];
    let data3 = vec![0x43u8; 64];

    group.bench_function("constant_time_compare_equal", |b| {
        b.iter(|| {
            let _ = utils::constant_time_compare(black_box(&data1), black_box(&data2));
        });
    });

    group.bench_function("constant_time_compare_different", |b| {
        b.iter(|| {
            let _ = utils::constant_time_compare(black_box(&data1), black_box(&data3));
        });
    });

    // Benchmark zeroize
    group.bench_function("zeroize_64", |b| {
        b.iter(|| {
            let mut data = vec![0x42u8; 64];
            utils::zeroize(black_box(&mut data));
        });
    });

    group.bench_function("zeroize_128", |b| {
        b.iter(|| {
            let mut data = vec![0x42u8; 128];
            utils::zeroize(black_box(&mut data));
        });
    });

    group.finish();
}

fn benchmark_algorithm_info(c: &mut Criterion) {
    let mut group = c.benchmark_group("Algorithm Information");

    // Benchmark algorithm info retrieval
    let kyber = polymera_crypto::Algorithm::Kyber(KyberParameterSet::Kyber512);
    let dilithium = polymera_crypto::Algorithm::Dilithium(DilithiumParameterSet::Dilithium2);

    group.bench_function("get_kyber_performance_info", |b| {
        b.iter(|| {
            let _ = utils::get_performance_info(black_box(&kyber));
        });
    });

    group.bench_function("get_dilithium_performance_info", |b| {
        b.iter(|| {
            let _ = utils::get_performance_info(black_box(&dilithium));
        });
    });

    group.bench_function("check_forward_secrecy", |b| {
        b.iter(|| {
            let _ = utils::provides_forward_secrecy(black_box(&kyber));
            let _ = utils::provides_forward_secrecy(black_box(&dilithium));
        });
    });

    group.finish();
}

fn benchmark_crypto_context(c: &mut Criterion) {
    let mut group = c.benchmark_group("Crypto Context");

    // Benchmark context creation
    group.bench_function("create_default_context", |b| {
        b.iter(|| {
            let _ = polymera_crypto::CryptoContext::new();
        });
    });

    group.bench_function("create_level3_context", |b| {
        b.iter(|| {
            let _ = polymera_crypto::CryptoContext::with_security_level(polymera_crypto::SecurityLevel::Level3);
        });
    });

    // Benchmark algorithm filtering
    let context = polymera_crypto::CryptoContext::new();

    group.bench_function("filter_by_security_level_128", |b| {
        b.iter(|| {
            let _ = context.algorithms_by_security_level(polymera_crypto::SecurityLevel::Level1);
        });
    });

    group.bench_function("filter_by_category_kem", |b| {
        b.iter(|| {
            let _ = context.algorithms_by_category(polymera_crypto::CryptoCategory::Kem);
        });
    });

    group.bench_function("filter_by_category_signature", |b| {
        b.iter(|| {
            let _ = context.algorithms_by_category(polymera_crypto::CryptoCategory::Signature);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_kyber_operations,
    benchmark_dilithium_operations,
    benchmark_utility_functions,
    benchmark_algorithm_info,
    benchmark_crypto_context,
);
criterion_main!(benches);
