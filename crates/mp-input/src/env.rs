//! Environment detection: OS, version, session type (X11/Wayland), desktop.

use mp_core::Environment;

pub fn detect(app_version: &str) -> Environment {
    let mut env = Environment {
        os_family: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        app_version: app_version.to_string(),
        ..Default::default()
    };
    #[cfg(target_os = "linux")]
    linux(&mut env);
    #[cfg(windows)]
    windows(&mut env);
    env
}

#[cfg(target_os = "linux")]
fn linux(env: &mut Environment) {
    use std::fs;
    if let Ok(text) = fs::read_to_string("/etc/os-release") {
        let get = |key: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix(&format!("{key}=")))
                .map(|v| v.trim_matches('"').to_string())
        };
        env.os_name = get("PRETTY_NAME").or_else(|| get("NAME"));
        env.os_version = get("VERSION_ID");
    }
    env.kernel_version = fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|s| s.trim().to_string());
    let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
    env.session_type = var("XDG_SESSION_TYPE").or_else(|| {
        if var("WAYLAND_DISPLAY").is_some() {
            Some("wayland".into())
        } else if var("DISPLAY").is_some() {
            Some("x11".into())
        } else {
            None
        }
    });
    env.desktop = var("XDG_CURRENT_DESKTOP").or_else(|| var("DESKTOP_SESSION"));
    env.notes.push(format!(
        "WAYLAND_DISPLAY={} DISPLAY={}",
        var("WAYLAND_DISPLAY").unwrap_or_else(|| "-".into()),
        var("DISPLAY").unwrap_or_else(|| "-".into())
    ));
    // Can we read input devices?
    let mut total = 0;
    let mut readable = 0;
    if let Ok(dir) = fs::read_dir("/dev/input") {
        for e in dir.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with("event") {
                total += 1;
                if fs::File::open(e.path()).is_ok() {
                    readable += 1;
                }
            }
        }
    }
    env.notes.push(format!(
        "/dev/input: {readable} de {total} dispositivos legíveis"
    ));
    if total > 0 && readable == 0 {
        env.notes.push(
            "SEM PERMISSÃO para ler /dev/input. Solução temporária: sudo usermod -aG input $USER e reiniciar a sessão. (A tela 'Preparar meu sistema' fará isso com um clique.)"
                .into(),
        );
    }
}

#[cfg(windows)]
fn windows(env: &mut Environment) {
    use windows::Win32::System::Registry::{
        HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegGetValueW,
    };
    use windows::core::{HSTRING, PCWSTR, w};

    let key = w!("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");
    let read_sz = |name: &str| -> Option<String> {
        let name = HSTRING::from(name);
        let mut buf = vec![0u16; 256];
        let mut size = (buf.len() * 2) as u32;
        let r = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                key,
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(buf.as_mut_ptr() as *mut _),
                Some(&mut size),
            )
        };
        if r.is_ok() {
            let len = (size as usize / 2).saturating_sub(1).min(buf.len());
            Some(
                String::from_utf16_lossy(&buf[..len])
                    .trim_end_matches('\0')
                    .to_string(),
            )
        } else {
            None
        }
    };
    let read_dword = |name: &str| -> Option<u32> {
        let name = HSTRING::from(name);
        let mut v = 0u32;
        let mut size = 4u32;
        let r = unsafe {
            RegGetValueW(
                HKEY_LOCAL_MACHINE,
                key,
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_DWORD,
                None,
                Some(&mut v as *mut u32 as *mut _),
                Some(&mut size),
            )
        };
        r.is_ok().then_some(v)
    };
    let build: Option<u32> = read_sz("CurrentBuildNumber").and_then(|b| b.parse().ok());
    let mut product = read_sz("ProductName");
    // Windows 11 still reports "Windows 10" in ProductName; build >= 22000 is Windows 11.
    if let (Some(p), Some(b)) = (product.as_mut(), build)
        && b >= 22000
        && p.contains("Windows 10")
    {
        *p = p.replace("Windows 10", "Windows 11");
    }
    env.os_name = product;
    let display = read_sz("DisplayVersion").or_else(|| read_sz("CSDVersion"));
    let ubr = read_dword("UBR");
    env.os_version = Some(format!(
        "{} (build {}{})",
        display.unwrap_or_else(|| "-".into()),
        build.map(|b| b.to_string()).unwrap_or_else(|| "?".into()),
        ubr.map(|u| format!(".{u}")).unwrap_or_default()
    ));
    env.kernel_version = build.map(|b| format!("NT build {b}"));
}
