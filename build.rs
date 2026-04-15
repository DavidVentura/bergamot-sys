use cmake;
use std::fs;
use std::env;
use std::path::PathBuf;

fn latest_android_ndk_from_sdk() -> Option<PathBuf> {
    let sdk_root = env::var_os("ANDROID_SDK_ROOT")
        .or_else(|| env::var_os("ANDROID_HOME"))
        .map(PathBuf::from)?;
    let ndk_root = sdk_root.join("ndk");
    let mut entries = fs::read_dir(ndk_root)
        .ok()?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort();
    entries.pop()
}

fn android_ndk_root() -> Option<PathBuf> {
    env::var_os("ANDROID_NDK_ROOT")
        .or_else(|| env::var_os("ANDROID_NDK_HOME"))
        .or_else(|| env::var_os("ANDROID_NDK"))
        .map(PathBuf::from)
        .or_else(latest_android_ndk_from_sdk)
}

fn main() {
    println!("cargo:rerun-if-changed=bindings/bergamot_wrapper.cpp");
    println!("cargo:rerun-if-changed=bindings/CMakeLists.txt");
    println!("cargo:rerun-if-changed=build.rs");

    let use_threads = env::var("CARGO_FEATURE_THREADS").is_ok();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target = env::var("TARGET").unwrap();
    let host = env::var("HOST").unwrap();
    let is_android = target_os == "android";

    let build_arch = match target_arch.as_str() {
        "aarch64" => "armv8-a",
        "arm" => "armv7-a",
        _ => "native",
    };

    let mut config = cmake::Config::new("bindings");
    config
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
        .define("BUILD_ARCH", build_arch);

    if is_android {
        let android_abi = match target_arch.as_str() {
            "aarch64" => "arm64-v8a",
            "arm" => "armeabi-v7a",
            "x86_64" => "x86_64",
            "x86" => "x86",
            _ => panic!("Unsupported Android target arch: {target_arch}"),
        };
        let android_platform = env::var("CARGO_NDK_PLATFORM")
            .ok()
            .filter(|value| value.chars().all(|ch| ch.is_ascii_digit()))
            .or_else(|| {
                env::var("ANDROID_PLATFORM")
                    .ok()
                    .and_then(|value| value.strip_prefix("android-").map(str::to_string))
                    .filter(|value| value.chars().all(|ch| ch.is_ascii_digit()))
            })
            .unwrap_or_else(|| "21".to_string());
        let ndk_root =
            android_ndk_root().expect("Android target requires ANDROID_NDK_ROOT or ANDROID_SDK_ROOT");

        config
            .generator("Ninja")
            .define(
                "CMAKE_TOOLCHAIN_FILE",
                ndk_root.join("build/cmake/android.toolchain.cmake"),
            )
            .define("ANDROID_ABI", android_abi)
            .define("ANDROID_PLATFORM", format!("android-{android_platform}"));
    }

    if target != host && !is_android {
        let cmake_system_processor = match target_arch.as_str() {
            "x86_64" => "x86_64",
            "x86" => "i686",
            "aarch64" => "aarch64",
            "arm" => "armv7",
            _ => &target_arch,
        };

        let cmake_c_compiler = match target_arch.as_str() {
            "aarch64" => "aarch64-linux-gnu-gcc",
            "arm" => "arm-linux-gnueabihf-gcc",
            _ => "gcc",
        };

        let cmake_cxx_compiler = match target_arch.as_str() {
            "aarch64" => "aarch64-linux-gnu-g++",
            "arm" => "arm-linux-gnueabihf-g++",
            _ => "g++",
        };

        config.define("CMAKE_SYSTEM_NAME", "Linux");
        config.define("CMAKE_SYSTEM_PROCESSOR", cmake_system_processor);
        config.define("CMAKE_C_COMPILER", cmake_c_compiler);
        config.define("CMAKE_CXX_COMPILER", cmake_cxx_compiler);
    }

    let dst = config.build();

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

    // intgemm is x86-only, ARM uses RUY instead
    if target_arch == "x86_64" || target_arch == "x86" {
        println!("cargo:rustc-link-lib=static=intgemm");
    }

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

    if target_arch == "x86_64" || target_arch == "x86" {
        println!("cargo:rustc-link-lib=static=ruy_have_built_path_for_avx");
        println!("cargo:rustc-link-lib=static=ruy_have_built_path_for_avx2_fma");
        println!("cargo:rustc-link-lib=static=ruy_have_built_path_for_avx512");
        println!("cargo:rustc-link-lib=static=ruy_kernel_avx");
        println!("cargo:rustc-link-lib=static=ruy_kernel_avx2_fma");
        println!("cargo:rustc-link-lib=static=ruy_kernel_avx512");
        println!("cargo:rustc-link-lib=static=ruy_pack_avx");
        println!("cargo:rustc-link-lib=static=ruy_pack_avx2_fma");
        println!("cargo:rustc-link-lib=static=ruy_pack_avx512");
    } else if target_arch == "aarch64" || target_arch == "arm" {
        println!("cargo:rustc-link-lib=static=ruy_kernel_arm");
        println!("cargo:rustc-link-lib=static=ruy_pack_arm");
    }

    println!("cargo:rustc-link-lib=static=cpuinfo");
    println!("cargo:rustc-link-lib=static=clog");

    if is_android {
        println!("cargo:rustc-link-lib=c++_shared");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }
    if use_threads && !is_android {
        println!("cargo:rustc-link-lib=pthread");
    }
}
