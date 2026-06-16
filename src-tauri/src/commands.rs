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
    pub id: u32,
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
        id = [int]$hwnd
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
fn activate_window(hwnd: u32) {
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
pub fn launch_wt(config: Config, target_window: Option<u32>) -> Result<String, String> {
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

/// 递归扫描指定目录下的 jsonl 文件
fn scan_jsonl_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_jsonl_files(&path, files);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "jsonl" {
                        files.push(path);
                    }
                }
            }
        }
    }
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

    let sessions_dir = match ai {
        AiProgram::Codex => PathBuf::from(&user_profile).join(".codex").join("sessions"),
        AiProgram::Claude => PathBuf::from(&user_profile).join(".claude").join("sessions"),
        AiProgram::Opencode => PathBuf::from(&user_profile).join(".opencode").join("sessions"),
    };

    if !sessions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut jsonl_files = Vec::new();
    scan_jsonl_files(&sessions_dir, &mut jsonl_files);

    let target_dir_norm = normalize_path(&dir);
    let mut sessions = Vec::new();

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

                            // 从剩余行中提取用户第一个真实 Goal 提问作为预览
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

    let sessions_dir = match ai {
        AiProgram::Codex => PathBuf::from(&user_profile).join(".codex").join("sessions"),
        AiProgram::Claude => PathBuf::from(&user_profile).join(".claude").join("sessions"),
        AiProgram::Opencode => PathBuf::from(&user_profile).join(".opencode").join("sessions"),
    };

    if !sessions_dir.exists() {
        return Err(format!("未找到 {:?} 的 sessions 目录", ai));
    }

    let mut jsonl_files = Vec::new();
    scan_jsonl_files(&sessions_dir, &mut jsonl_files);

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
    let mut messages = Vec::new();
    
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

    Ok(messages)
}