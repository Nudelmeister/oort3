use anyhow::Result;
use clap::Parser as _;
use glob::glob;
use libflate::gzip::Encoder;
use rayon::prelude::*;
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use tar::Header;

thread_local! {
  static COMPILERS: std::cell::RefCell<oort_compiler::Compiler> = RefCell::new(oort_compiler::Compiler::new());
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("build_builtin_ais=info"),
    )
    .init();

    #[derive(clap::Parser, Debug)]
    struct Arguments {
        #[clap(short, long, default_value = "shared/ai")]
        input: String,
        #[clap(short, long, default_value = "shared/ai/builtin-ai.tar.gz")]
        output: String,
    }
    let args = Arguments::parse();

    let paths: Vec<_> = glob(&format!("{}/**/*.rs", args.input))?
        .map(|x| x.unwrap())
        .filter(|x| !["user.rs", "lib.rs"].contains(&x.file_name().unwrap().to_str().unwrap()))
        .collect();

    let results: Vec<_> = paths
        .par_iter()
        .map(
            |path| -> Result<(PathBuf, /*rust*/ String, /*wasm*/ Vec<u8>)> {
                let source_code = std::fs::read_to_string(path).unwrap();
                let wasm = COMPILERS
                    .with(|compiler_cell| compiler_cell.borrow_mut().compile(&source_code))?;
                let optimized_wasm = wasm_opt(&wasm)?;
                println!(
                    "{} source {}K wasm {}K optimized {}K",
                    path.display(),
                    source_code.len() / 1000,
                    wasm.len() / 1000,
                    optimized_wasm.len() / 1000
                );
                Ok((path.clone(), source_code, optimized_wasm))
            },
        )
        .collect();

    let writer = std::fs::File::create(args.output)?;
    let encoder = Encoder::new(writer).unwrap();
    let mut ar = tar::Builder::new(encoder);

    for r in results.iter() {
        let (path, source_code, _) = r.as_ref().unwrap();
        let path = path.strip_prefix(&args.input)?;
        let data = source_code.as_bytes();
        let mut header = Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_cksum();
        ar.append_data(&mut header, path, data).unwrap();
    }

    for r in results.iter() {
        let (path, _, wasm) = r.as_ref().unwrap();
        let mut path = path.strip_prefix(&args.input)?.to_path_buf();
        path.set_extension("wasm");
        let mut header = Header::new_gnu();
        header.set_size(wasm.len() as u64);
        header.set_cksum();
        ar.append_data(&mut header, path, &wasm[..]).unwrap();
    }

    let encoder = ar.into_inner()?;
    encoder.finish();

    Ok(())
}

fn wasm_opt(wasm: &[u8]) -> Result<Vec<u8>> {
    let mut child = std::process::Command::new("wasm-opt")
        .args(["-Oz", "-o", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    let mut child_stdin = child.stdin.take().unwrap();
    child_stdin.write_all(wasm)?;
    drop(child_stdin);
    let output = child.wait_with_output()?;
    Ok(output.stdout)
}
