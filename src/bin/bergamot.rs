use bergamot_sys::{BlockingService, TranslationModel};

fn create_config(data_path: &str, model: &str, src_vocab: &str, tgt_vocab: &str) -> String {
    format!(
        r#"models:
  - {data_path}/{model}
vocabs:
  - {data_path}/{src_vocab}
  - {data_path}/{tgt_vocab}
beam-size: 1
normalize: 1.0
word-penalty: 0
max-length-break: 128
mini-batch-words: 1024
max-length-factor: 2.0
skip-cost: true
cpu-threads: 1
quiet: true
quiet-translation: true
gemm-precision: int8shiftAlphaAll
alignment: soft"#
    )
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        eprintln!(
            "Usage: {} <data_path> <model> <src_vocab> <tgt_vocab>",
            args[0]
        );
        std::process::exit(1);
    }

    let data_path = &args[1];
    let model = &args[2];
    let src_vocab = &args[3];
    let tgt_vocab = &args[4];

    println!("Creating service...");
    let service = BlockingService::new(256);

    let config = create_config(data_path, model, src_vocab, tgt_vocab);
    println!("Loading model into cache with key 'enes'...");
    let model = TranslationModel::from_config(&config).expect("Failed to load model");

    println!("Translating 'hello'...");
    let inputs = vec!["hello"];
    let results = service.translate(&model, inputs.as_slice());

    println!("Translation result:");
    for result in results {
        println!("  {}", result);
    }
}
