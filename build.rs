use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let build_dir = manifest_dir.join("target").join("bergamot-build");

    let cache_file = build_dir.join("CMakeCache.txt");
    if cache_file.exists() {
        if let Ok(cache_content) = std::fs::read_to_string(&cache_file) {
            let expected_source = format!("CMAKE_HOME_DIRECTORY:INTERNAL={}", manifest_dir.display());
            if !cache_content.contains(&expected_source) {
                let _ = std::fs::remove_dir_all(&build_dir);
            }
        }
    }

    std::fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    println!("cargo:rerun-if-changed=src/bergamot_wrapper.cpp");
    println!("cargo:rerun-if-changed=CMakeLists.txt");
    println!("cargo:rerun-if-changed=build.rs");

    let use_threads = env::var("CARGO_FEATURE_THREADS").is_ok();

    let mut cmake_config = Command::new("cmake");
    cmake_config
        .current_dir(&build_dir)
        .arg(&manifest_dir)
        .arg("-DCMAKE_BUILD_TYPE=Release")
        .arg(format!("-DUSE_THREADS={}", if use_threads { "ON" } else { "OFF" }));

    let status = cmake_config.status().expect("Failed to run cmake configure");
    if !status.success() {
        panic!("CMake configuration failed");
    }

    let num_jobs = env::var("NUM_JOBS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or_else(|| std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1));

    let mut cmake_build = Command::new("cmake");
    cmake_build
        .current_dir(&build_dir)
        .arg("--build")
        .arg(".")
        .arg("--config")
        .arg("Release")
        .arg("--parallel")
        .arg(num_jobs.to_string());

    let status = cmake_build.status().expect("Failed to run cmake build");
    if !status.success() {
        panic!("CMake build failed");
    }

    println!("cargo:rustc-link-search=native={}", build_dir.display());
    println!("cargo:rustc-link-search=native={}/bergamot-translator/src/translator", build_dir.display());
    println!("cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/intgemm", build_dir.display());
    println!("cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/sentencepiece/src", build_dir.display());
    println!("cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/ruy", build_dir.display());
    println!("cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/ruy/profiler", build_dir.display());
    println!("cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/third_party/cpuinfo", build_dir.display());
    println!("cargo:rustc-link-search=native={}/bergamot-translator/3rd_party/marian-dev/src/3rd_party/ruy/third_party/cpuinfo/deps/clog", build_dir.display());
    println!("cargo:rustc-link-search=native={}/lib", build_dir.display());

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
