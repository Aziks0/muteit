// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::path::Path;
use std::{env, fs};

use minijinja::{Environment, UndefinedBehavior};
use toml::Table;
use windows_bindgen::Bindgen;
use winresource::WindowsResource;

fn convert_image_to_rgba(path: &Path) {
    println!("cargo::rerun-if-changed={}", path.display());

    let rgba = image::open(path)
        .unwrap_or_else(|err| {
            panic!(
                "Expected {} to open as an image. Got error: {}",
                path.display(),
                err
            )
        })
        .into_rgba8()
        .into_raw();

    let out_dir = env::var("OUT_DIR").unwrap();
    let filename = path.with_extension("bin");
    let filename = filename.file_name().expect("image to have a filename");
    let path = Path::new(&out_dir).join(filename);
    fs::write(&path, rgba).unwrap_or_else(|err| {
        panic!(
            "Expected {} to be written. Got error: {}",
            path.display(),
            err
        )
    });
}

fn convert_images() {
    let files = ["mic-none.png", "mic-off.png", "mic.png"];
    for file in files {
        let path = Path::new("assets").join(file);
        convert_image_to_rgba(path.as_path());
    }
}

type Context = toml::map::Map<String, toml::Value>;

fn set_resource(mut ctx: Context) {
    println!("cargo::rerun-if-changed=assets/muteit_icon.ico");

    let mut res = WindowsResource::new();

    if !cfg!(feature = "dev") {
        println!("cargo::rerun-if-changed=assets/bundle/resource.rc.jinja");

        ctx.insert(
            "VERSION_MAJOR".into(),
            std::env::var("CARGO_PKG_VERSION_MAJOR").unwrap().into(),
        );
        ctx.insert(
            "VERSION_MINOR".into(),
            std::env::var("CARGO_PKG_VERSION_MINOR").unwrap().into(),
        );
        ctx.insert(
            "VERSION_PATCH".into(),
            std::env::var("CARGO_PKG_VERSION_PATCH").unwrap().into(),
        );

        let source = fs::read_to_string("assets/bundle/resource.rc.jinja")
            .expect("`assets/bundle/resource.rc.jinja` to be read");

        let mut env = Environment::empty();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        let resource = env
            .render_str(&source, ctx)
            .expect("resource template to be rendered");

        let out_dir = env::var("OUT_DIR").unwrap();
        let path = Path::new(&out_dir).join("resource.rc");
        std::fs::write(&path, resource).expect("resource.rc to be written");

        res.add_toolkit_include(true);
        res.set_resource_file(path.to_str().unwrap());
    } else {
        // Just set the minimum resources required to make the app build, when not bundling.
        res.set_icon("assets/muteit_icon.ico");
    }

    res.compile().expect("resource file to compile");
}

fn set_envs() -> Context {
    println!("cargo::rerun-if-changed=env.toml");

    let env_file = include_str!("env.toml");

    let envs = env_file
        .parse::<Table>()
        .expect("env.toml to be a valid toml file");

    let mut temp = String::new();
    for (key, value) in &envs {
        let value = value.as_str().unwrap_or_else(|| {
            temp = value.to_string();
            &temp
        });

        println!("cargo::rustc-env={}={}", key, value);
    }

    envs
}

fn generate_windows_binding() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let path = Path::new(&out_dir).join("windows.rs");

    Bindgen::new()
        .output(path)
        .filters(["GetSystemMetrics", "SM_CXICON", "SM_CXSMICON"])
        .write();
}

fn main() {
    // NOTE: hotreloading doesn't work when the build script needs to run, so a full app relaunch is
    // required. Avoid build script run at all cost.

    generate_windows_binding();

    let ctx = set_envs();
    set_resource(ctx);

    convert_images();
}
