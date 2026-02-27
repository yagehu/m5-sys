use std::{collections::HashMap, env, path::PathBuf};

use flate2::read::GzDecoder;
use tar::Archive;

#[derive(Clone, Debug)]
struct CallType {
    name: &'static str,
    impl_file: Option<&'static str>,
    verifier: Option<&'static str>,
    enabled: bool,
    default: bool,
}

impl CallType {
    fn new(name: &'static str) -> Self {
        Self {
            name,
            impl_file: None,
            verifier: None,
            enabled: false,
            default: false,
        }
    }
}

fn main() {
    println!("cargo:rustc-link-lib=static=m5");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let gem5_dir = match env::var("GEM5_SRC") {
        Ok(path) => PathBuf::from(path),
        Err(_) => {
            let mut response =
                ureq::get("https://github.com/gem5/gem5/archive/refs/tags/v25.1.0.0.tar.gz")
                    .call()
                    .unwrap();

            assert!(response.status().is_success());

            let mut archive = Archive::new(GzDecoder::new(response.body_mut().as_reader()));

            archive.unpack(&out_path).unwrap();

            out_path.join("gem5-25.1.0.0")
        }
    };
    let m5_src_path = gem5_dir.join("util").join("m5").join("src");
    let gem5_include_dir = gem5_dir.join("include");
    let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let arch_map = [("riscv64", "riscv"), ("x86_64", "x86")]
        .into_iter()
        .collect::<HashMap<_, _>>();
    let mut call_types = [
        ("inst", CallType::new("inst")),
        ("addr", CallType::new("addr")),
        ("semi", CallType::new("semi")),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();
    let abi = arch_map
        .get(arch.as_str())
        .unwrap_or(&arch.as_str())
        .to_string();
    let abi_call_types_map = [
        (
            "riscv",
            Box::new(|call_types: &mut HashMap<&str, CallType>| {
                let call_type = call_types.get_mut("inst").unwrap();

                call_type.impl_file = Some("m5op.S");
                call_type.verifier = Some("verify_inst.cc");
                call_type.enabled = true;
                call_type.default = true;
            }) as Box<dyn Fn(_)>,
        ),
        (
            "x86",
            Box::new(|cts: &mut HashMap<&str, CallType>| {
                let ct = cts.get_mut("inst").unwrap();

                ct.impl_file = Some("m5op.S");
                ct.verifier = Some("verify_inst.cc");
                ct.enabled = true;

                let ct = cts.get_mut("addr").unwrap();

                ct.impl_file = Some("m5op_addr.S");
                ct.enabled = true;
                ct.default = true;
            }) as Box<dyn Fn(_)>,
        ),
    ]
    .into_iter()
    .collect::<HashMap<_, _>>();

    abi_call_types_map.get(abi.as_str()).unwrap()(&mut call_types);
    drop(abi_call_types_map);

    let call_types: Vec<_> = call_types
        .values()
        .filter(|call_type| call_type.enabled)
        .collect();
    let m5ops = call_types
        .iter()
        .map(|call_type| {
            (
                call_type.name,
                PathBuf::from("abi")
                    .join(&abi)
                    .join(call_type.impl_file.unwrap()),
            )
        })
        .collect::<HashMap<_, _>>();
    let all_m5ops = m5ops.values().collect::<Vec<_>>();
    let m5_mmap = "m5_mmap.c";
    let mut all_files = all_m5ops
        .into_iter()
        .map(|p| m5_src_path.join(p))
        .collect::<Vec<_>>();

    all_files.push(m5_src_path.join(m5_mmap));

    let mut cc_build = cc::Build::new();

    if let Ok(m5_cc) = env::var("M5_CC") {
        cc_build.compiler(m5_cc);
    }

    cc_build
        .include(&gem5_include_dir)
        .files(&all_files)
        .compile("m5");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include", gem5_dir.display()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("unable to generate bindings");

    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
