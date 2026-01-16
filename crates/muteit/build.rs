// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::path::Path;
use std::{env, fs};

use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, StyleSheet};
use lightningcss::targets::{Browsers, Targets};
use oxc::allocator::Allocator;
use oxc::codegen::{Codegen, CodegenOptions, CommentOptions};
use oxc::mangler::MangleOptions;
use oxc::minifier::{CompressOptions, Minifier, MinifierOptions};
use oxc::parser::Parser;
use oxc::span::SourceType;
use toml::Table;
use winresource::WindowsResource;

fn minify_tailwind() {
    println!("cargo::rerun-if-changed=assets/tailwind.css");

    // `include_str!()` adds the file to the dependency list, even if the code does not run (i.e.
    // even if `bundle` feature is not selected), so we prefer to read the file at runtime to avoid
    // the build script running every time we make changes to our css.
    let source =
        fs::read_to_string("assets/tailwind.css").expect("`assets/tailwind.css` to be read");
    let source = &source;

    // Edge >= 140
    let targets = Targets {
        browsers: Some(Browsers {
            edge: Some(140 << 16),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut stylesheet =
        StyleSheet::parse(source, ParserOptions::default()).expect("tailwind.css to be parsed");
    stylesheet
        .minify(MinifyOptions {
            targets,
            ..Default::default()
        })
        .expect("tailwind.css to be minified");
    let code = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            targets,
            ..Default::default()
        })
        .expect("minified tailwind to be serialized to a String")
        .code;

    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("tailwind.css");
    fs::write(path, code).expect("tailwind.css to be written");
}

fn minify_js() {
    println!("cargo::rerun-if-changed=assets/hotkey_binding.js");

    let source = fs::read_to_string("assets/hotkey_binding.js")
        .expect("`assets/hotkey_binding.js` to be read");
    let source = &source;

    let allocator = Allocator::default();
    let parser_ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
    let mut program = parser_ret.program;
    let minifier_ret = Minifier::new(MinifierOptions {
        mangle: Some(MangleOptions {
            top_level: true,
            ..Default::default()
        }),
        compress: Some(CompressOptions::smallest()),
    })
    .minify(&allocator, &mut program);
    let code = Codegen::new()
        .with_options(CodegenOptions {
            comments: CommentOptions::disabled(),
            minify: true,
            ..Default::default()
        })
        .with_scoping(minifier_ret.scoping)
        .build(&program)
        .code;

    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("hotkey_binding.js");
    fs::write(path, code).expect("hotkey_binding.js to be written");
}

fn convert_image_to_rgba(path: &Path) {
    println!("cargo::rerun-if-changed={}", path.display());

    let rgba = image::open(path)
        .expect(format!("{} to open as an image", path.display()).as_str())
        .into_rgba8()
        .into_raw();

    let out_dir = env::var("OUT_DIR").unwrap();
    let filename = path.with_extension("bin");
    let filename = filename.file_name().expect("image to have a filename");
    let path = Path::new(&out_dir).join(filename);
    fs::write(&path, rgba).expect(format!("{} to be written", path.display()).as_str());
}

fn convert_images() {
    let files = ["mic-none.png", "mic-off.png", "mic.png"];
    for file in files {
        let path = Path::new("assets").join(file);
        convert_image_to_rgba(path.as_path());
    }
}

fn set_resource() {
    println!("cargo::rerun-if-changed=assets/muteit_icon.ico");

    let mut res = WindowsResource::new();
    res.set_icon("assets/muteit_icon.ico");
    res.compile().expect("resource file to compile");
}

fn set_envs() {
    println!("cargo::rerun-if-changed=env.toml");

    let env_file = include_str!("env.toml");

    let envs = env_file
        .parse::<Table>()
        .expect("env.toml to be a valid toml file");

    for (key, value) in envs {
        let value = value.as_str().expect("envs should be strings");
        println!("cargo::rustc-env={}={}", key, value);
    }
}

fn main() {
    // NOTE: hotreloading doesn't work when the build script needs to run, so a full app relaunch is
    // required. Avoid build script run at all cost.

    set_envs();
    set_resource();

    convert_images();

    if cfg!(feature = "bundle") {
        minify_js();
        minify_tailwind();
    }
}
