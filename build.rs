use cmake;
use std::env;

fn main() {
    println!("cargo:rerun-if-changed=bindings/bergamot_wrapper.cpp");
    println!("cargo:rerun-if-changed=bindings/CMakeLists.txt");
    println!("cargo:rerun-if-changed=build.rs");

    let use_threads = env::var("CARGO_FEATURE_THREADS").is_ok();

    let dst = cmake::Config::new("bindings")
        .define("CMAKE_BUILD_TYPE", "Release")
        .define("COMPILE_WASM", "OFF")
        .define("COMPILE_TESTS", "OFF")
        .define("COMPILE_UNIT_TESTS", "OFF")
        .define("COMPILE_LIBRARY_ONLY", "ON")
        .define("USE_STATIC_LIBS", "ON")
        .define("USE_SENTENCEPIECE", "ON")
        .define("USE_MKL", "OFF")
        .define("ENABLE_CACHE_STATS", "OFF")
        .define("SSPLIT_COMPILE_LIBRARY_ONLY", "ON")
        .define("SSPLIT_USE_INTERNAL_PCRE2", "ON")
        .define("USE_RUY", "ON")
        .define("USE_RUY_SGEMM", "ON")
        .define("INTGEMM_DONT_BUILD_TESTS", "ON")
        .define("USE_THREADS", if use_threads { "ON" } else { "OFF" })
        .build();

    // Add all necessary library search paths
    let build_dir = dst.join("build");
    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!(
        "cargo:rustc-link-search=native={}/bergamot-translator/src/translator",
        build_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/intgemm",
        build_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/sentencepiece/src",
        build_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/ruy",
        build_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/ruy/profiler",
        build_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/third_party/cpuinfo",
        build_dir.display()
    );
    println!(
        "cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/third_party/cpuinfo/deps/clog",
        build_dir.display()
    );
    println!("cargo:rustc-link-search=native={}/lib", build_dir.display());

    // Link all static libraries (cmake crate doesn't handle transitive dependencies)
    println!("cargo:rustc-link-lib=static=bergamot_wrapper");
    println!("cargo:rustc-link-lib=static=bergamot-translator");
    println!("cargo:rustc-link-lib=static=marian");
    println!("cargo:rustc-link-lib=static=sentencepiece");
    println!("cargo:rustc-link-lib=static=sentencepiece_train");
    println!("cargo:rustc-link-lib=static=intgemm");
    println!("cargo:rustc-link-lib=static=ssplit");
    println!("cargo:rustc-link-lib=static=pcre2-8");

    // Link all ruy libraries
    println!("cargo:rustc-link-lib=static=ruy_ctx");
    println!("cargo:rustc-link-lib=static=ruy_context");
    println!("cargo:rustc-link-lib=static=ruy_context_get_ctx");
    println!("cargo:rustc-link-lib=static=ruy_frontend");
    println!("cargo:rustc-link-lib=static=ruy_trmul");
    println!("cargo:rustc-link-lib=static=ruy_prepare_packed_matrices");
    println!("cargo:rustc-link-lib=static=ruy_system_aligned_alloc");
    println!("cargo:rustc-link-lib=static=ruy_allocator");
    println!("cargo:rustc-link-lib=static=ruy_block_map");
    println!("cargo:rustc-link-lib=static=ruy_blocking_counter");
    println!("cargo:rustc-link-lib=static=ruy_cpuinfo");
    println!("cargo:rustc-link-lib=static=ruy_denormal");
    println!("cargo:rustc-link-lib=static=ruy_thread_pool");
    println!("cargo:rustc-link-lib=static=ruy_tune");
    println!("cargo:rustc-link-lib=static=ruy_wait");
    println!("cargo:rustc-link-lib=static=ruy_prepacked_cache");
    println!("cargo:rustc-link-lib=static=ruy_apply_multiplier");
    println!("cargo:rustc-link-lib=static=ruy_profiler_instrumentation");
    println!("cargo:rustc-link-lib=static=ruy_have_built_path_for_avx");
    println!("cargo:rustc-link-lib=static=ruy_have_built_path_for_avx2_fma");
    println!("cargo:rustc-link-lib=static=ruy_have_built_path_for_avx512");
    println!("cargo:rustc-link-lib=static=ruy_kernel_avx");
    println!("cargo:rustc-link-lib=static=ruy_kernel_avx2_fma");
    println!("cargo:rustc-link-lib=static=ruy_kernel_avx512");
    println!("cargo:rustc-link-lib=static=ruy_pack_avx");
    println!("cargo:rustc-link-lib=static=ruy_pack_avx2_fma");
    println!("cargo:rustc-link-lib=static=ruy_pack_avx512");
    println!("cargo:rustc-link-lib=static=cpuinfo");
    println!("cargo:rustc-link-lib=static=clog");

    println!("cargo:rustc-link-lib=stdc++");

    if use_threads {
        println!("cargo:rustc-link-lib=pthread");
    }
}
