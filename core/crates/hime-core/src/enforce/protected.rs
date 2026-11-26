/// Extracts the normalized base executable name (without directory paths and without `.exe`).
/// Symmetrically handles Windows backslashes and Unix forward slashes.
pub fn extract_process_base_name(process_name: &str) -> String {
    let trimmed = process_name.trim();
    let normalized = trimmed.replace('/', "\\");
    let file_name = std::path::Path::new(&normalized)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(trimmed);
    let lower = file_name.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// Static, immutable protected processes list per §3.3:
/// - Windows system: System, smss.exe, csrss.exe, wininit.exe, services.exe, lsass.exe, winlogon.exe, dwm.exe, svchost.exe, fontdrvhost.exe
/// - Shell: explorer.exe, SearchHost.exe, StartMenuExperienceHost.exe, ShellExperienceHost.exe, Taskmgr.exe
/// - Security: MsMpEng.exe, SecurityHealthService.exe, NisSrv.exe
/// - Hime runtime: hime-island.exe, hime-core.exe, msedgewebview2.exe, test dummy binaries
/// - Default development tools: code.exe, devenv.exe, idea64.exe, windowsterminal.exe, cmd.exe, powershell.exe
pub fn is_protected_process(process_name: &str) -> bool {
    let name = extract_process_base_name(process_name);

    matches!(
        name.as_str(),
        "system"
            | "smss"
            | "csrss"
            | "wininit"
            | "services"
            | "lsass"
            | "winlogon"
            | "dwm"
            | "svchost"
            | "fontdrvhost"
            | "explorer"
            | "searchhost"
            | "startmenuexperiencehost"
            | "shellexperiencehost"
            | "taskmgr"
            | "msmpeng"
            | "securityhealthservice"
            | "nissrv"
            | "hime-island"
            | "hime-core"
            | "msedgewebview2"
            | "code"
            | "devenv"
            | "idea64"
            | "windowsterminal"
            | "cmd"
            | "powershell"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protected_processes() {
        assert!(is_protected_process("explorer.exe"));
        assert!(is_protected_process("Explorer"));
        assert!(is_protected_process("dwm.exe"));
        assert!(is_protected_process("Code.exe"));
        assert!(is_protected_process("powershell.exe"));
        assert!(is_protected_process("hime-core.exe"));

        // Distractions must not be protected
        assert!(!is_protected_process("chrome.exe"));
        assert!(!is_protected_process("steam.exe"));
        assert!(!is_protected_process("discord.exe"));
    }

    #[test]
    fn test_protected_processes_path_evasion() {
        // Assert relative and absolute path-qualified process names are resolved safely
        assert!(is_protected_process(r".\explorer.exe"));
        assert!(is_protected_process(r"..\cmd.exe"));
        assert!(is_protected_process(r"C:\Windows\System32\cmd.exe"));
        assert!(is_protected_process("./powershell.exe"));
        assert!(is_protected_process(r"System32\cmd.exe"));
        assert!(is_protected_process(r"C:\Windows\explorer.exe"));
    }
}
