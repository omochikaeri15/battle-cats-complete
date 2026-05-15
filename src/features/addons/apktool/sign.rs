use std::process::Command;
use std::path::Path;
use std::fs;
use crate::features::addons::apktool::download::{get_signer_path, get_java_path};
use crate::features::mods::logic::state::SignType;

fn execute_command(binary_path: &Path, arguments: &[String]) -> Result<(), String> {
    let output = Command::new(binary_path)
        .args(arguments)
        .output()
        .map_err(|error| format!("Failed to start process: {}", error))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let exit_code = output.status.code().unwrap_or(-1);

    Err(format!("Exit Code {}:\nOut: {}\nErr: {}", exit_code, stdout.trim(), stderr.trim()))
}

fn run_java_with_fallback(arguments: &[String], log_callback: &impl Fn(String)) -> Result<(), String> {
    let local_java_path = get_java_path();

    if let Some(java_binary) = local_java_path {
        if execute_command(&java_binary, arguments).is_ok() {
            return Ok(());
        }
        log_callback("Local JRE environment crashed or is incompatible. Falling back to native system JRE...".to_string());
    } else {
        log_callback("Local portable JRE not found. Using system JRE...".to_string());
    }

    let system_java = Path::new("java");
    if let Err(system_error) = execute_command(system_java, arguments) {
        return Err(format!("System Java execution also failed.\nError: {}", system_error));
    }

    Ok(())
}

pub fn sign_apk(input_apk: &Path, output_apk: &Path, sign_type: &SignType, log_callback: &impl Fn(String)) -> Result<(), String> {
    let signer_jar = get_signer_path().ok_or("uber-apk-signer.jar is not installed.")?;

    log_callback(format!("Zipaligning and applying {:?} signature via uber-apk-signer...", sign_type));

    let temp_out_dir = input_apk.parent().unwrap_or_else(|| Path::new(".")).join("sign_temp");
    let _ = fs::create_dir_all(&temp_out_dir);

    let mut arguments = vec![
        "-jar".to_string(),
        signer_jar.to_string_lossy().to_string(),
        "-a".to_string(),
        input_apk.to_string_lossy().to_string(),
        "--out".to_string(),
        temp_out_dir.to_string_lossy().to_string(),
        "--allowResign".to_string(),
    ];

    let native_zipalign = Path::new("/usr/bin/zipalign");
    if native_zipalign.exists() {
        arguments.push("--zipalignLocation".to_string());
        arguments.push(native_zipalign.to_string_lossy().to_string());
    }

    if let Err(execution_error) = run_java_with_fallback(&arguments, log_callback) {
        let _ = fs::remove_dir_all(&temp_out_dir);
        return Err(format!("uber-apk-signer failed: {}", execution_error));
    }

    let mut signed_file_path = None;
    if let Ok(entries) = fs::read_dir(&temp_out_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("apk") {
                signed_file_path = Some(entry.path());
                break;
            }
        }
    }

    if let Some(signed_apk) = signed_file_path {
        if fs::rename(&signed_apk, output_apk).is_err() {
            let _ = fs::copy(&signed_apk, output_apk).map_err(|error| format!("Failed to copy signed APK: {}", error))?;
        }
        let _ = fs::remove_dir_all(&temp_out_dir);
        log_callback("Universal APK successfully zipaligned and signed!".to_string());
        Ok(())
    } else {
        let _ = fs::remove_dir_all(&temp_out_dir);
        Err("uber-apk-signer completed successfully, but no output APK was found.".into())
    }
}