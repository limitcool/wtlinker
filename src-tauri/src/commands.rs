use crate::config::*;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::mpsc;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_dialog::FilePath;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;


/// WT 窗口信息
#[derive(Clone, Serialize, Deserialize)]
pub struct WindowInfo {
    pub id: i64,
    pub title: String,
    pub pid: u32,
    pub tabs: Vec<String>,
}

/// 加载配置
#[tauri::command]
pub fn load_config() -> Config {
    load_config_file()
}

/// 保存配置
#[tauri::command]
pub fn save_config(config: Config) {
    save_config_file(&config);
}

/// 弹出文件夹选择对话框
#[tauri::command]
pub async fn pick_folder(app: tauri::AppHandle) -> Option<String> {
    let (tx, rx) = mpsc::channel();
    
    app.dialog().file().pick_folder(move |folder: Option<FilePath>| {
        let path = folder.and_then(|p| {
            let s = p.to_string();
            // 移除 file:// 前缀
            let s = s.strip_prefix("file://").unwrap_or(&s);
            #[cfg(windows)]
            let s = s.strip_prefix("file:///").unwrap_or(s);
            // 移除末尾的引号（如果有）
            let s = s.trim_end_matches('"');
            if s.is_empty() {
                None
            } else {
                Some(s.to_string())
            }
        });
        let _ = tx.send(path);
    });
    
    rx.recv().unwrap_or(None)
}

/// 获取 Windows Terminal 窗口列表（使用 EnumWindows 获取所有窗口）
/// 获取 Windows Terminal 窗口列表（使用 EnumWindows 获取所有窗口并通过 UI Automation 提取标签页）
#[tauri::command]
pub fn get_wt_windows() -> Result<Vec<WindowInfo>, String> {
    // 运行带有 UI Automation 和 EnumWindows 的 PowerShell 脚本
    let script = r#"
Add-Type -AssemblyName UIAutomationClient -ErrorAction SilentlyContinue
Add-Type -AssemblyName UIAutomationTypes -ErrorAction SilentlyContinue

$code = @"
using System;
using System.Runtime.InteropServices;
using System.Text;
using System.Collections.Generic;

public class WindowEnumerator {
    [DllImport("user32.dll")]
    public static extern bool EnumWindows(EnumWindowsProc enumProc, IntPtr lParam);
    
    [DllImport("user32.dll")]
    public static extern int GetClassName(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
    
    [DllImport("user32.dll")]
    public static extern bool IsWindowVisible(IntPtr hWnd);
    
    [DllImport("user32.dll")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    
    [DllImport("user32.dll")]
    public static extern int GetWindowTextLength(IntPtr hWnd);
    
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint processId);
    
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    
    private static List<IntPtr> results = new List<IntPtr>();
    
    private static bool FilterWindow(IntPtr hWnd, IntPtr lParam) {
        if (IsWindowVisible(hWnd)) {
            StringBuilder className = new StringBuilder(256);
            GetClassName(hWnd, className, 256);
            string cls = className.ToString();
            if (cls == "CASCADIA_HOSTING_WINDOW_CLASS" || cls == "MetroWindow") {
                results.Add(hWnd);
            }
        }
        return true;
    }
    
    public static List<IntPtr> GetAllWindows() {
        results.Clear();
        EnumWindows(new EnumWindowsProc(FilterWindow), IntPtr.Zero);
        return results;
    }
}
"@

Add-Type -TypeDefinition $code -ErrorAction SilentlyContinue

$wtWindows = @()
try {
    $hwnds = [WindowEnumerator]::GetAllWindows()
} catch {
    $hwnds = @()
}

foreach ($hwnd in $hwnds) {
    $len = [WindowEnumerator]::GetWindowTextLength($hwnd)
    $sb = New-Object System.Text.StringBuilder($len + 1)
    [WindowEnumerator]::GetWindowText($hwnd, $sb, $sb.Capacity) | Out-Null
    $title = $sb.ToString()
    
    $procId = 0
    [WindowEnumerator]::GetWindowThreadProcessId($hwnd, [ref]$procId) | Out-Null
    
    $tabsList = @()
    try {
        $el = [System.Windows.Automation.AutomationElement]::FromHandle($hwnd)
        if ($el -ne $null) {
            $cond = New-Object System.Windows.Automation.PropertyCondition(
                [System.Windows.Automation.AutomationElement]::ControlTypeProperty,
                [System.Windows.Automation.ControlType]::TabItem
            )
            $tabs = $el.FindAll([System.Windows.Automation.TreeScope]::Descendants, $cond)
            foreach ($t in $tabs) {
                if ($t.Current.Name) {
                    $tabsList += $t.Current.Name
                }
            }
        }
    } catch {}
    
    $wtWindows += [PSCustomObject]@{
        id = [long]$hwnd
        title = $title
        pid = [int]$procId
        tabs = $tabsList
    }
}

if ($wtWindows.Count -eq 0) {
    Write-Output "[]"
} else {
    @($wtWindows) | ConvertTo-Json -Compress
}
"#;

    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .map_err(|e| format!("执行失败: {}", e))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.trim().is_empty() {
        eprintln!("PowerShell stderr: {}", stderr);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stdout_trimmed = stdout.trim();
    
    if stdout_trimmed.is_empty() || stdout_trimmed == "null" {
        return Ok(Vec::new());
    }

    // 解析 JSON 结果
    let windows: Vec<WindowInfo> = serde_json::from_str(stdout_trimmed)
        .map_err(|e| format!("解析失败: {} - 输出: {}", e, stdout_trimmed))?;

    Ok(windows)
}

/// 激活特定 HWND 窗口的辅助函数
fn activate_window(hwnd: i64) {
    let script = format!(
        r#"
$code = @"
using System;
using System.Runtime.InteropServices;

public class Win32Active {{
    [DllImport("user32.dll")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport("user32.dll")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}}
"@
Add-Type -TypeDefinition $code -ErrorAction SilentlyContinue
[Win32Active]::ShowWindow([IntPtr]{}, 9) | Out-Null
[Win32Active]::SetForegroundWindow([IntPtr]{}) | Out-Null
"#,
        hwnd, hwnd
    );
    
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
}

/// 构建 wt 参数列表
fn build_wt_args_vec(entries: &[Entry], flags: &str, append_mode: bool, shell: &str) -> Vec<String> {
    let mut args = Vec::new();

    for (i, e) in entries.iter().enumerate() {
        if i > 0 {
            // 用分号隔开多个命令，作为 wt.exe 的子命令分隔符
            args.push(";".to_string());
        }

        // 第一个命令要加 wt 或 new-tab
        if i == 0 {
            if append_mode {
                args.push("-w".to_string());
                args.push("0".to_string());
            }
            args.push("new-tab".to_string());
        } else {
            args.push("new-tab".to_string());
        }

        // 启动目录
        args.push("-d".to_string());
        args.push(e.dir.clone());

        // 标签页内执行的命令
        // 根据 shell 决定使用 cmd /k 还是 pwsh/powershell -NoExit -Command
        if shell == "cmd" {
            args.push("cmd".to_string());
            args.push("/k".to_string());
        } else if shell == "powershell" {
            args.push("powershell".to_string());
            args.push("-NoExit".to_string());
            args.push("-Command".to_string());
        } else {
            // 默认使用 pwsh
            args.push("pwsh".to_string());
            args.push("-NoExit".to_string());
            args.push("-Command".to_string());
        }

        let ai_cmd = e.ai.as_str();
        let mut inner_cmd = ai_cmd.to_string();
        if !e.session.is_empty() {
            inner_cmd.push_str(" resume ");
            inner_cmd.push_str(&e.session);
        }
        
        let mut ai_flags = flags.trim().to_string();
        if e.ai == AiProgram::Codex {
            if !ai_flags.is_empty() {
                inner_cmd.push(' ');
                inner_cmd.push_str(&ai_flags);
            }
        } else if e.ai == AiProgram::Claude {
            ai_flags = ai_flags.replace("--dangerously-bypass-approvals-and-sandbox", "--dangerously-skip-permissions");
            if !ai_flags.is_empty() {
                inner_cmd.push(' ');
                inner_cmd.push_str(&ai_flags);
            }
        } else if e.ai == AiProgram::Opencode {
            let clean_flags = ai_flags.replace("--dangerously-bypass-approvals-and-sandbox", "").trim().to_string();
            if !clean_flags.is_empty() {
                inner_cmd.push(' ');
                inner_cmd.push_str(&clean_flags);
            }
        }
        args.push(inner_cmd);
    }

    args
}

/// 启动 Windows Terminal
#[tauri::command]
pub fn launch_wt(config: Config, target_window: Option<i64>) -> Result<String, String> {
    // 收集启用的、有目录的项
    let to_launch: Vec<&Entry> = config
        .entries
        .iter()
        .filter(|e| e.enabled && !e.dir.trim().is_empty())
        .collect();

    if to_launch.is_empty() {
        return Err("没有可启动的项（需选择目录）".to_string());
    }

    // 如果是追加模式，且指定了目标窗口，则先将该窗口带到最前（激活）
    if config.append_mode {
        if let Some(hwnd) = target_window {
            activate_window(hwnd);
        }
    }

    let args = build_wt_args_vec(
        &to_launch.iter().map(|e| (*e).clone()).collect::<Vec<Entry>>(),
        &config.default_flags,
        config.append_mode,
        &config.shell,
    );

    // 直接执行 wt 命令而不通过 cmd.exe /c，这样系统会由 CreateProcess 确保参数的完美双引号转义
    let result = Command::new("wt")
        .args(&args)
        .spawn();

    match result {
        Ok(_) => Ok(format!("已启动 {} 个标签", to_launch.len())),
        Err(e) => Err(format!("启动失败: {}", e)),
    }
}

/// Codex 会话信息
#[derive(Serialize, Deserialize, Clone)]
pub struct CodexSession {
    pub id: String,
    pub timestamp: String,
    pub last_modified: u64,
    pub preview: String,
}

#[derive(Deserialize)]
struct SessionMetaPayload {
    id: String,
    cwd: String,
}

#[derive(Deserialize)]
struct SessionMeta {
    timestamp: String,
    #[serde(rename = "type")]
    msg_type: String,
    payload: SessionMetaPayload,
}

/// 辅助函数：格式化/归一化路径，在 Windows 上统一为反斜杠和小写
fn normalize_path(p: &str) -> String {
    p.replace('/', "\\").to_lowercase()
}

/// 递归扫描指定目录下特定后缀的文件
fn scan_files_by_ext(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_files_by_ext(&path, extension, files);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == extension {
                        files.push(path);
                    }
                }
            }
        }
    }
}

/// 辅助函数：将 Unix 时间戳（毫秒）格式化为 RFC3339 风格的字符串（纯 Rust 算法，零第三方库依赖）
fn format_timestamp_ms(ms: u64) -> String {
    let secs = ms / 1000;
    let days = secs / 86400;
    let r_secs = secs % 86400;
    let hour = r_secs / 3600;
    let min = (r_secs % 3600) / 60;
    let sec = r_secs % 60;
    
    let mut year = 1970;
    let mut day_of_year = days;
    loop {
        let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if is_leap { 366 } else { 365 };
        if day_of_year >= days_in_year {
            day_of_year -= days_in_year;
            year += 1;
        } else {
            break;
        }
    }
    
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut month = 1;
    let mut rem_days = day_of_year;
    for &days_in_month in &month_days {
        if rem_days >= days_in_month {
            rem_days -= days_in_month;
            month += 1;
        } else {
            break;
        }
    }
    let day = rem_days + 1;
    
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, hour, min, sec)
}

/// 辅助函数：将字符串过滤清理换行并截断，用于做摘要预览
fn truncate_string(s: &str, max_len: usize) -> String {
    let clean: String = s.chars().filter(|&c| c != '\n' && c != '\r').collect();
    if clean.chars().count() > max_len {
        let truncated: String = clean.chars().take(max_len).collect();
        format!("{}...", truncated)
    } else {
        clean
    }
}

/// 辅助函数：读取 jsonl 后续行，提取首个用户的非 instructions 提问内容作为会话预览
fn extract_user_preview(reader: &mut BufReader<File>) -> String {
    use serde_json::Value;
    let mut line = String::new();
    // 最多扫描 30 行，以防有些会话开头有长 instructions 和系统环境数据注入
    for _ in 0..30 {
        line.clear();
        if reader.read_line(&mut line).is_err() || line.is_empty() {
            break;
        }
        if let Ok(val) = serde_json::from_str::<Value>(&line) {
            if let Some(role) = val.pointer("/payload/role").and_then(|r| r.as_str()) {
                if role == "user" {
                    let mut txt_opt = val.pointer("/payload/content/0/text").and_then(|t| t.as_str());
                    if txt_opt.is_none() {
                        txt_opt = val.pointer("/payload/content/0/input_text").and_then(|t| t.as_str());
                    }
                    if let Some(txt) = txt_opt {
                        // 跳过 instructions 和 aborted 节点
                        if txt.contains("<INSTRUCTIONS>") || txt.contains("AGENTS.md") || txt.contains("<turn_aborted>") {
                            continue;
                        }
                        return truncate_string(txt, 75);
                    }
                }
            }
        }
    }
    "新会话 (暂无内容预览)".to_string()
}

/// 获取某个项目目录匹配的历史会话 ID 列表（支持 codex, claude, opencode）
#[tauri::command]
pub fn get_sessions(dir: String, ai: AiProgram) -> Result<Vec<CodexSession>, String> {
    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| "无法获取 USERPROFILE 环境变量".to_string())?;

    let target_dir_norm = normalize_path(&dir);
    let mut sessions = Vec::new();

    match ai {
        AiProgram::Codex => {
            let sessions_dir = PathBuf::from(&user_profile).join(".codex").join("sessions");
            if !sessions_dir.exists() {
                return Ok(Vec::new());
            }

            let mut jsonl_files = Vec::new();
            scan_files_by_ext(&sessions_dir, "jsonl", &mut jsonl_files);

            for path in jsonl_files {
                if let Ok(file) = File::open(&path) {
                    let mut reader = BufReader::new(file);
                    let mut first_line = String::new();
                    if reader.read_line(&mut first_line).is_ok() {
                        if let Ok(meta) = serde_json::from_str::<SessionMeta>(&first_line) {
                            if meta.msg_type == "session_meta" {
                                let session_cwd_norm = normalize_path(&meta.payload.cwd);
                                if session_cwd_norm == target_dir_norm {
                                    let last_modified = fs::metadata(&path)
                                        .and_then(|m| m.modified())
                                        .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                                        .unwrap_or(0);

                                    let preview = extract_user_preview(&mut reader);

                                    sessions.push(CodexSession {
                                        id: meta.payload.id,
                                        timestamp: meta.timestamp,
                                        last_modified,
                                        preview,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        AiProgram::Claude => {
            let sessions_dir = PathBuf::from(&user_profile).join(".claude").join("projects");
            if !sessions_dir.exists() {
                return Ok(Vec::new());
            }

            let mut jsonl_files = Vec::new();
            scan_files_by_ext(&sessions_dir, "jsonl", &mut jsonl_files);

            for path in jsonl_files {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    let mut session_id = String::new();
                    let mut timestamp = String::new();
                    let mut is_matched = false;
                    let mut preview = String::new();

                    use serde_json::Value;
                    for line_res in reader.lines() {
                        if let Ok(line) = line_res {
                            if let Ok(val) = serde_json::from_str::<Value>(&line) {
                                if let Some(cwd) = val.get("cwd").and_then(|c| c.as_str()) {
                                    if normalize_path(cwd) == target_dir_norm {
                                        is_matched = true;
                                    }
                                }
                                if session_id.is_empty() {
                                    if let Some(sid) = val.get("sessionId").and_then(|s| s.as_str()) {
                                        session_id = sid.to_string();
                                    }
                                }
                                if timestamp.is_empty() {
                                    if let Some(t) = val.get("timestamp").and_then(|s| s.as_str()) {
                                        timestamp = t.to_string();
                                    }
                                }
                                if preview.is_empty() {
                                    if let Some(role) = val.pointer("/message/role").and_then(|r| r.as_str()) {
                                        if role == "user" {
                                            if let Some(content) = val.pointer("/message/content").and_then(|c| c.as_str()) {
                                                if !content.contains("<INSTRUCTIONS>") {
                                                    preview = truncate_string(content, 75);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if is_matched && !session_id.is_empty() {
                        let last_modified = fs::metadata(&path)
                            .and_then(|m| m.modified())
                            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                            .unwrap_or(0);
                        if preview.is_empty() {
                            preview = "新会话 (暂无内容预览)".to_string();
                        }
                        sessions.push(CodexSession {
                            id: session_id,
                            timestamp,
                            last_modified,
                            preview,
                        });
                    }
                }
            }
        }
        AiProgram::Opencode => {
            let mut sessions_dir = PathBuf::from(&user_profile).join(".opencode").join("sessions");
            if !sessions_dir.exists() {
                sessions_dir = PathBuf::from(&user_profile).join(".zcode").join("v2").join("sessions");
            }
            if !sessions_dir.exists() {
                return Ok(Vec::new());
            }

            let mut json_files = Vec::new();
            scan_files_by_ext(&sessions_dir, "json", &mut json_files);

            #[derive(Deserialize)]
            #[allow(non_snake_case)]
            struct ZCodeMeta {
                taskId: String,
                workspacePath: String,
                createdAt: u64,
            }
            #[derive(Deserialize)]
            struct ZCodeMessage {
                role: String,
                content: String,
            }
            #[derive(Deserialize)]
            struct ZCodeSessionFile {
                meta: ZCodeMeta,
                messages: Option<Vec<ZCodeMessage>>,
            }

            for path in json_files {
                if let Ok(file_content) = fs::read_to_string(&path) {
                    if let Ok(sess_data) = serde_json::from_str::<ZCodeSessionFile>(&file_content) {
                        if normalize_path(&sess_data.meta.workspacePath) == target_dir_norm {
                            let last_modified = fs::metadata(&path)
                                .and_then(|m| m.modified())
                                .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
                                .unwrap_or(0);

                            let mut preview = "新会话 (暂无内容预览)".to_string();
                            if let Some(msgs) = &sess_data.messages {
                                for m in msgs {
                                    if m.role == "user" {
                                        preview = truncate_string(&m.content, 75);
                                        break;
                                    }
                                }
                            }

                            let ts_str = format_timestamp_ms(sess_data.meta.createdAt);

                            sessions.push(CodexSession {
                                id: sess_data.meta.taskId,
                                timestamp: ts_str,
                                last_modified,
                                preview,
                            });
                        }
                    }
                }
            }
        }
    }

    // 按最后修改时间降序排序，最新的排在前面
    sessions.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    Ok(sessions)
}

/// 详细对话单条消息
#[derive(Serialize, Deserialize, Clone)]
pub struct CodexMessage {
    pub role: String,
    pub content: String,
}

/// 辅助函数：提取 <USER_REQUEST> 包裹的真实内容，过滤 instructions 系统注入
fn extract_user_request(text: &str) -> String {
    if let Some(start_idx) = text.find("<USER_REQUEST>") {
        if let Some(end_idx) = text[start_idx..].find("</USER_REQUEST>") {
            let actual_start = start_idx + "<USER_REQUEST>".len();
            let actual_end = start_idx + end_idx;
            if actual_end > actual_start {
                return text[actual_start..actual_end].trim().to_string();
            }
        }
    }
    // 如果不含标签但包含 instructions，则视为系统任务初始化
    if text.contains("<INSTRUCTIONS>") || text.contains("AGENTS.md") {
        return "系统环境初始化 (已自动过滤长系统注入数据)".to_string();
    }
    text.to_string()
}

/// 获取某个特定会话的详细对话历史（支持 codex, claude, opencode）
#[tauri::command]
pub fn get_session_details(session_id: String, ai: AiProgram) -> Result<Vec<CodexMessage>, String> {
    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| "无法获取 USERPROFILE 环境变量".to_string())?;

    let mut messages = Vec::new();

    match ai {
        AiProgram::Codex => {
            let sessions_dir = PathBuf::from(&user_profile).join(".codex").join("sessions");
            if !sessions_dir.exists() {
                return Err("未找到 .codex/sessions 目录".to_string());
            }

            let mut jsonl_files = Vec::new();
            scan_files_by_ext(&sessions_dir, "jsonl", &mut jsonl_files);

            // 寻找匹配 session_id 的文件
            let mut matched_path = None;
            for path in jsonl_files {
                if let Ok(file) = File::open(&path) {
                    let mut reader = BufReader::new(file);
                    let mut first_line = String::new();
                    if reader.read_line(&mut first_line).is_ok() {
                        if let Ok(meta) = serde_json::from_str::<SessionMeta>(&first_line) {
                            if meta.msg_type == "session_meta" && meta.payload.id == session_id {
                                matched_path = Some(path);
                                break;
                            }
                        }
                    }
                }
            }

            let path = match matched_path {
                Some(p) => p,
                None => return Err(format!("未找到会话 ID 为 {} 的日志文件", session_id)),
            };

            let file = File::open(path).map_err(|e| format!("无法打开日志文件: {}", e))?;
            let reader = BufReader::new(file);

            use serde_json::Value;
            for line_res in reader.lines() {
                let line = match line_res {
                    Ok(l) => l,
                    Err(_) => continue,
                };
                if let Ok(val) = serde_json::from_str::<Value>(&line) {
                    if let Some(role) = val.pointer("/payload/role").and_then(|r| r.as_str()) {
                        if role == "user" || role == "assistant" || role == "model" {
                            let mut txt_opt = val.pointer("/payload/content/0/text").and_then(|t| t.as_str());
                            if txt_opt.is_none() {
                                txt_opt = val.pointer("/payload/content/0/input_text").and_then(|t| t.as_str());
                            }
                            if txt_opt.is_none() {
                                txt_opt = val.pointer("/payload/content/0/output_text").and_then(|t| t.as_str());
                            }
                            if let Some(txt) = txt_opt {
                                let cleaned = if role == "user" {
                                    extract_user_request(txt)
                                } else {
                                    txt.replace("<turn_aborted>", "").trim().to_string()
                                };

                                if !cleaned.is_empty() {
                                    messages.push(CodexMessage {
                                        role: if role == "user" { "user".to_string() } else { "assistant".to_string() },
                                        content: cleaned,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        AiProgram::Claude => {
            let sessions_dir = PathBuf::from(&user_profile).join(".claude").join("projects");
            if !sessions_dir.exists() {
                return Err("未找到 .claude/projects 目录".to_string());
            }

            let mut jsonl_files = Vec::new();
            scan_files_by_ext(&sessions_dir, "jsonl", &mut jsonl_files);

            let mut matched_path = None;
            for path in jsonl_files {
                if let Ok(file) = File::open(&path) {
                    let reader = BufReader::new(file);
                    use serde_json::Value;
                    for line_res in reader.lines() {
                        if let Ok(line) = line_res {
                            if let Ok(val) = serde_json::from_str::<Value>(&line) {
                                if let Some(sid) = val.get("sessionId").and_then(|s| s.as_str()) {
                                    if sid == session_id {
                                        matched_path = Some(path.clone());
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    if matched_path.is_some() {
                        break;
                    }
                }
            }

            let path = match matched_path {
                Some(p) => p,
                None => return Err(format!("未找到会话 ID 为 {} 的日志文件", session_id)),
            };

            let file = File::open(path).map_err(|e| format!("无法打开日志文件: {}", e))?;
            let reader = BufReader::new(file);

            use serde_json::Value;
            for line_res in reader.lines() {
                let line = match line_res {
                    Ok(l) => l,
                    Err(_) => continue,
                };
                if let Ok(val) = serde_json::from_str::<Value>(&line) {
                    let role_opt = val.get("type").and_then(|t| t.as_str());
                    if let Some(role) = role_opt {
                        if role == "user" {
                            if let Some(content) = val.pointer("/message/content").and_then(|c| c.as_str()) {
                                let cleaned = extract_user_request(content);
                                if !cleaned.is_empty() {
                                    messages.push(CodexMessage {
                                        role: "user".to_string(),
                                        content: cleaned,
                                    });
                                }
                            }
                        } else if role == "assistant" {
                            let mut text_content = String::new();
                            if let Some(content_arr) = val.pointer("/message/content").and_then(|c| c.as_array()) {
                                for item in content_arr {
                                    if let Some(t) = item.get("type").and_then(|tp| tp.as_str()) {
                                        if t == "text" {
                                            if let Some(txt) = item.get("text").and_then(|tx| tx.as_str()) {
                                                text_content.push_str(txt);
                                            }
                                        }
                                    }
                                }
                            }
                            let cleaned = text_content.replace("<turn_aborted>", "").trim().to_string();
                            if !cleaned.is_empty() {
                                messages.push(CodexMessage {
                                    role: "assistant".to_string(),
                                    content: cleaned,
                                });
                            }
                        }
                    }
                }
            }
        }
        AiProgram::Opencode => {
            let mut sessions_dir = PathBuf::from(&user_profile).join(".opencode").join("sessions");
            if !sessions_dir.exists() {
                sessions_dir = PathBuf::from(&user_profile).join(".zcode").join("v2").join("sessions");
            }
            if !sessions_dir.exists() {
                return Err("未找到 OpenCode 会话日志目录".to_string());
            }

            let mut json_files = Vec::new();
            scan_files_by_ext(&sessions_dir, "json", &mut json_files);

            #[derive(Deserialize)]
            #[allow(non_snake_case)]
            struct ZCodeMeta {
                taskId: String,
            }
            #[derive(Deserialize)]
            struct ZCodeMessage {
                role: String,
                content: String,
            }
            #[derive(Deserialize)]
            struct ZCodeSessionFile {
                meta: ZCodeMeta,
                messages: Option<Vec<ZCodeMessage>>,
            }

            let mut matched_file_content = None;
            for path in json_files {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(sess_data) = serde_json::from_str::<ZCodeSessionFile>(&content) {
                        if sess_data.meta.taskId == session_id {
                            matched_file_content = Some(sess_data);
                            break;
                        }
                    }
                }
            }

            let sess_data = match matched_file_content {
                Some(data) => data,
                None => return Err(format!("未找到会话 ID 为 {} 的日志文件", session_id)),
            };

            if let Some(msgs) = sess_data.messages {
                for m in msgs {
                    let cleaned = if m.role == "user" {
                        extract_user_request(&m.content)
                    } else {
                        m.content.replace("<turn_aborted>", "").trim().to_string()
                    };
                    if !cleaned.is_empty() {
                        messages.push(CodexMessage {
                            role: if m.role == "user" { "user".to_string() } else { "assistant".to_string() },
                            content: cleaned,
                        });
                    }
                }
            }
        }
    }

    Ok(messages)
}

fn escape_project_path(cwd: &str) -> String {
    cwd.replace(':', "-").replace('\\', "-").replace('/', "-")
}

fn generate_dummy_uuid() -> String {
    use std::time::SystemTime;
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let mut seed = start as u64;
    
    let mut next_random = move || {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        seed
    };

    let r1 = next_random();
    let r2 = next_random();
    let r3 = next_random();
    let r4 = next_random();

    format!(
        "{:08x}-{:04x}-4{:03x}-a{:03x}-{:012x}",
        (r1 & 0xFFFFFFFF) as u32,
        ((r1 >> 32) & 0xFFFF) as u16,
        ((r2) & 0xFFF) as u16,
        ((r2 >> 12) & 0xFFF) as u16,
        (r3 as u64) << 16 | ((r4 & 0xFFFF) as u64)
    )
}

#[tauri::command]
pub fn convert_claude_to_codex(session_id: String, cwd: String) -> Result<String, String> {
    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| "无法获取 USERPROFILE 环境变量".to_string())?;

    let sessions_dir = PathBuf::from(&user_profile).join(".claude").join("projects");
    if !sessions_dir.exists() {
        return Err("未找到 .claude/projects 目录".to_string());
    }

    let mut jsonl_files = Vec::new();
    scan_files_by_ext(&sessions_dir, "jsonl", &mut jsonl_files);

    let mut matched_path = None;
    for path in jsonl_files {
        if let Ok(file) = File::open(&path) {
            let reader = BufReader::new(file);
            use serde_json::Value;
            for line_res in reader.lines() {
                if let Ok(line) = line_res {
                    if let Ok(val) = serde_json::from_str::<Value>(&line) {
                        if let Some(sid) = val.get("sessionId").and_then(|s| s.as_str()) {
                            if sid == session_id {
                                matched_path = Some(path.clone());
                                break;
                            }
                        }
                    }
                }
            }
            if matched_path.is_some() {
                break;
            }
        }
    }

    let path = match matched_path {
        Some(p) => p,
        None => return Err(format!("未找到会话 ID 为 {} 的 Claude 日志文件", session_id)),
    };

    let file = File::open(path).map_err(|e| format!("无法打开 Claude 日志文件: {}", e))?;
    let reader = BufReader::new(file);

    let codex_sessions_dir = PathBuf::from(&user_profile).join(".codex").join("sessions");
    fs::create_dir_all(&codex_sessions_dir).map_err(|e| format!("创建 .codex/sessions 目录失败: {}", e))?;
    let target_codex_path = codex_sessions_dir.join(format!("{}.jsonl", session_id));

    let mut writer = File::create(&target_codex_path).map_err(|e| format!("无法创建目标 Codex 日志文件: {}", e))?;
    use std::io::Write;
    use serde_json::{json, Value};
    use std::time::SystemTime;
    
    let mut session_timestamp = format_timestamp_ms(
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64
    );

    let mut codex_messages = Vec::new();

    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        if let Ok(val) = serde_json::from_str::<Value>(&line) {
            if let Some(ts) = val.get("timestamp").and_then(|t| t.as_str()) {
                if !ts.is_empty() {
                    session_timestamp = ts.to_string();
                }
            }

            let role_opt = val.get("type").and_then(|t| t.as_str());
            if let Some(role) = role_opt {
                if role == "user" {
                    if let Some(content) = val.pointer("/message/content").and_then(|c| c.as_str()) {
                        codex_messages.push(json!({
                            "timestamp": session_timestamp.clone(),
                            "type": "node_message",
                            "payload": {
                                "role": "user",
                                "content": [
                                    {
                                        "type": "text",
                                        "text": content
                                    }
                                ]
                            }
                        }));
                    }
                } else if role == "assistant" {
                    let mut text_content = String::new();
                    if let Some(content_arr) = val.pointer("/message/content").and_then(|c| c.as_array()) {
                        for item in content_arr {
                            if let Some(t) = item.get("type").and_then(|tp| tp.as_str()) {
                                if t == "text" {
                                    if let Some(txt) = item.get("text").and_then(|tx| tx.as_str()) {
                                        text_content.push_str(txt);
                                    }
                                }
                            }
                        }
                    }
                    if !text_content.is_empty() {
                        codex_messages.push(json!({
                            "timestamp": session_timestamp.clone(),
                            "type": "node_message",
                            "payload": {
                                "role": "assistant",
                                "content": [
                                    {
                                        "type": "text",
                                        "text": text_content
                                    }
                                ]
                            }
                        }));
                    }
                }
            }
        }
    }

    let meta_line = json!({
        "timestamp": session_timestamp,
        "type": "session_meta",
        "payload": {
            "id": session_id,
            "cwd": cwd
        }
    });
    writeln!(writer, "{}", meta_line.to_string()).map_err(|e| format!("写入首行失败: {}", e))?;

    for msg in codex_messages {
        writeln!(writer, "{}", msg.to_string()).map_err(|e| format!("写入消息行失败: {}", e))?;
    }

    Ok(format!("成功转换 Claude 会话 {} 到 Codex", session_id))
}

#[tauri::command]
pub fn convert_codex_to_claude(session_id: String, cwd: String) -> Result<String, String> {
    let user_profile = std::env::var("USERPROFILE")
        .map_err(|_| "无法获取 USERPROFILE 环境变量".to_string())?;

    let sessions_dir = PathBuf::from(&user_profile).join(".codex").join("sessions");
    if !sessions_dir.exists() {
        return Err("未找到 .codex/sessions 目录".to_string());
    }

    let mut jsonl_files = Vec::new();
    scan_files_by_ext(&sessions_dir, "jsonl", &mut jsonl_files);

    let mut matched_path = None;
    for path in jsonl_files {
        if let Ok(file) = File::open(&path) {
            let mut reader = BufReader::new(file);
            let mut first_line = String::new();
            if reader.read_line(&mut first_line).is_ok() {
                if let Ok(meta) = serde_json::from_str::<SessionMeta>(&first_line) {
                    if meta.msg_type == "session_meta" && meta.payload.id == session_id {
                        matched_path = Some(path.clone());
                        break;
                    }
                }
            }
        }
    }

    let path = match matched_path {
        Some(p) => p,
        None => return Err(format!("未找到会话 ID 为 {} 的 Codex 日志文件", session_id)),
    };

    let file = File::open(path).map_err(|e| format!("无法打开 Codex 日志文件: {}", e))?;
    let reader = BufReader::new(file);

    let escaped_cwd = escape_project_path(&cwd);
    let claude_project_dir = PathBuf::from(&user_profile)
        .join(".claude")
        .join("projects")
        .join(&escaped_cwd);
    fs::create_dir_all(&claude_project_dir).map_err(|e| format!("创建 Claude 项目目录失败: {}", e))?;
    let target_claude_path = claude_project_dir.join(format!("{}.jsonl", session_id));

    let mut writer = File::create(&target_claude_path).map_err(|e| format!("无法创建目标 Claude 日志文件: {}", e))?;
    use std::io::Write;
    use serde_json::{json, Value};

    let first_line = json!({
        "type": "mode",
        "mode": "normal",
        "sessionId": session_id
    });
    let second_line = json!({
        "type": "permission-mode",
        "permissionMode": "bypassPermissions",
        "sessionId": session_id
    });

    writeln!(writer, "{}", first_line.to_string()).map_err(|e| format!("写入首行引导失败: {}", e))?;
    writeln!(writer, "{}", second_line.to_string()).map_err(|e| format!("写入第二行引导失败: {}", e))?;

    let mut parent_uuid: Option<String> = None;

    let mut is_first = true;
    for line_res in reader.lines() {
        let line = match line_res {
            Ok(l) => l,
            Err(_) => continue,
        };
        if is_first {
            is_first = false;
            continue;
        }

        if let Ok(val) = serde_json::from_str::<Value>(&line) {
            let msg_type = val.get("type").and_then(|t| t.as_str());
            if msg_type == Some("node_message") {
                let role = val.pointer("/payload/role").and_then(|r| r.as_str()).unwrap_or("user");
                let mut text_content = String::new();
                
                if let Some(content_arr) = val.pointer("/payload/content").and_then(|c| c.as_array()) {
                    for item in content_arr {
                        if let Some(t) = item.get("type").and_then(|tp| tp.as_str()) {
                            if t == "text" {
                                if let Some(txt) = item.get("text").and_then(|tx| tx.as_str()) {
                                    text_content.push_str(txt);
                                }
                            }
                        }
                    }
                }

                if text_content.is_empty() {
                    if let Some(txt) = val.pointer("/payload/content/0/input_text").and_then(|t| t.as_str()) {
                        text_content = txt.to_string();
                    } else if let Some(txt) = val.pointer("/payload/content/0/output_text").and_then(|t| t.as_str()) {
                        text_content = txt.to_string();
                    }
                }

                let current_uuid = generate_dummy_uuid();
                let timestamp = val.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");

                let claude_line = if role == "user" {
                    json!({
                        "parentUuid": parent_uuid,
                        "isSidechain": false,
                        "type": "user",
                        "message": {
                            "role": "user",
                            "content": text_content
                        },
                        "uuid": current_uuid.clone(),
                        "timestamp": timestamp,
                        "permissionMode": "bypassPermissions",
                        "cwd": cwd,
                        "sessionId": session_id
                    })
                } else {
                    json!({
                        "parentUuid": parent_uuid,
                        "isSidechain": false,
                        "type": "assistant",
                        "message": {
                            "role": "assistant",
                            "content": [
                                {
                                    "type": "text",
                                    "text": text_content
                                }
                            ]
                        },
                        "uuid": current_uuid.clone(),
                        "timestamp": timestamp,
                        "cwd": cwd,
                        "sessionId": session_id
                    })
                };

                writeln!(writer, "{}", claude_line.to_string()).map_err(|e| format!("写入转换行失败: {}", e))?;
                parent_uuid = Some(current_uuid);
            }
        }
    }

    Ok(format!("成功转换 Codex 会话 {} 到 Claude", session_id))
}