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

fn minify_tailwind() {
    println!("cargo:rerun-if-changed=assets/tailwind.css");

    let source = include_str!("assets/tailwind.css");

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

fn main() {
    println!("cargo:rerun-if-changed=assets/hotkey_binding.js");

    let source = include_str!("assets/hotkey_binding.js");

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

    if !cfg!(debug_assertions) {
        minify_tailwind();
    }
}
