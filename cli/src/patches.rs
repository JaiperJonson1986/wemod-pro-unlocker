use crate::files;
use std::{collections::HashMap, fs, path::PathBuf};

pub fn patch_pro_mode(extracted_resource_dir: PathBuf, opts: &HashMap<String, String>) {
    for app_bundle in files::get_all_app_bundles(extracted_resource_dir) {
        let contents_result = fs::read_to_string(&app_bundle);

        if contents_result.is_err() {
            println!(
                "error while reading possible app bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        let contents = contents_result.unwrap();

        if contents.contains(r#""application/json"===e.headers.get("Content-Type")"#) {
            let app_bundle_patch = include_str!("fetchIntercept.js").to_string().replace(
                "/*{%account%}*/",
                if opts.contains_key("account") {
                    opts.get("account").unwrap()
                } else {
                    ""
                },
            );
            let app_bundle_original_code =
              "return\"application/json\"===e.headers.get(\"Content-Type\")?await e.json():await e.text()";

            if !contents.contains(app_bundle_original_code) {
                crate::err(
                    "failed to enable pro mode. WeMod may have updated their program".to_string(),
                );
            }

            let app_bundle_contents_patched =
                contents.replace(app_bundle_original_code, app_bundle_patch.as_str());

            match fs::write(&app_bundle, app_bundle_contents_patched) {
                Ok(_) => break,
                Err(err) => println!("failed to enable pro mode (write): {}", err),
            };
        }
    }
}

pub fn patch_creator_mode(extracted_resource_dir: PathBuf) {
    for app_bundle in files::get_all_app_bundles(extracted_resource_dir) {
        let contents_result = fs::read_to_string(&app_bundle);

        if contents_result.is_err() {
            println!(
                "error while reading possible app bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        let contents = contents_result.unwrap();

        if contents.contains("get isCreator(){") {
            println!("creator");
            match fs::write(
                &app_bundle,
                contents.replace("get isCreator(){", "get isCreator(){return true;"),
            ) {
                Ok(_) => {}
                Err(err) => println!("failed to patch creator mode: {}", err),
            };
        }
    }
}

pub fn patch_index_js(extracted_resource_dir: PathBuf) {
    let index_js = extracted_resource_dir.join("index.js");
    if !index_js.exists() || !index_js.is_file() {
        crate::err("index.js not found. your WeMod version may not be supported.".to_string())
    }

    let index_js_contents = fs::read_to_string(&index_js)
        .expect("failed to read index.js")
        .replace("if(d.devMode)", "if(process.argv.includes('-dev'))");

    fs::write(index_js, index_js_contents).expect("failed to write index.js");
}

pub fn patch_vendor_bundle(extracted_resource_dir: PathBuf) {
    for vendor_bundle in files::get_all_vendor_bundles(extracted_resource_dir) {
        let contents_result = fs::read_to_string(&vendor_bundle);

        if contents_result.is_err() {
            println!(
                "error while reading possible vendor bundle file: {}",
                contents_result.unwrap_err()
            );
            continue;
        }

        let mut contents = contents_result.unwrap();

        let vendor_bundle_patch = include_str!("vendorPatch.js")
            .to_string()
            .replace("/*{%version%}*/", crate::VERSION);

        contents.insert_str(0, &vendor_bundle_patch);

        let write_result = fs::write(vendor_bundle, contents);

        if write_result.is_err() {
            println!(
                "error while writing vendor bundle file: {}",
                write_result.unwrap_err()
            );
            continue;
        }

        break;
    }
}

pub fn patch_asar_integrity(wemod_version_folder: PathBuf) {
    println!("Patching asar integrity...");

    let wemod_exe = wemod_version_folder.join("WeMod.exe");
    let wemod_exe_old = wemod_version_folder.join("WeMod.exe.old");
    if wemod_exe_old.exists() {
        fs::copy(&wemod_exe_old, &wemod_exe).expect("failed to copy WeMod.exe.old to WeMod.exe");
    } else {
        fs::copy(&wemod_exe, &wemod_exe_old).expect("failed to copy WeMod.exe to WeMod.exe.old");
    }

    let wemod_exe_contents = fs::read(&wemod_exe).expect("failed to read WeMod.exe");

    let bypass_hex = vec![0x30, 0x30, 0x30, 0x30, 0x30, 0x31, 0x30, 0x31];
    let old_hex = vec![0x30, 0x30, 0x30, 0x30, 0x31, 0x31, 0x30, 0x31];

    let bypass_hex_pos = wemod_exe_contents.windows(8).position(|window| window == bypass_hex);
    let old_hex_pos = wemod_exe_contents.windows(8).position(|window| window == old_hex);

    if bypass_hex_pos.is_some() {
        println!("Asar integrity already patched.");
        return;
    }

    if old_hex_pos.is_none() {
        println!("Failed to patch asar integrity.");
        return;
    }

    let old_hex_pos = old_hex_pos.unwrap();

    let mut wemod_exe_contents = wemod_exe_contents.to_vec();
    for i in 0..8 {
        wemod_exe_contents[old_hex_pos + i] = bypass_hex[i];
    }

    fs::write(&wemod_exe, wemod_exe_contents).expect("failed to write WeMod.exe");

    println!("Done.");
}
