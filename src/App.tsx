import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'

interface Entry {
  name: string
  dir: string
  session: string
  enabled: boolean
  ai: 'claude' | 'codex' | 'opencode'
}

interface Config {
  default_flags: string
  append_mode: boolean
  default_ai: 'claude' | 'codex' | 'opencode'
  entries: Entry[]
  shell: 'pwsh' | 'powershell' | 'cmd'
}

interface WindowInfo {
  id: number
  title: string
  pid: number
  tabs: string[]
}

interface CodexSession {
  id: string
  timestamp: string
  last_modified: number
  preview: string
}

interface CodexMessage {
  role: 'user' | 'assistant'
  content: string
}


const CodexIcon = ({ className = "w-3.5 h-3.5 mr-1.5 shrink-0" }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor" fillRule="evenodd" style={{ flex: 'none', lineHeight: 1 }}>
    <path d="M9.205 8.658v-2.26c0-.19.072-.333.238-.428l4.543-2.616c.619-.357 1.356-.523 2.117-.523 2.854 0 4.662 2.212 4.662 4.566 0 .167 0 .357-.024.547l-4.71-2.759a.797.797 0 00-.856 0l-5.97 3.473zm10.609 8.8V12.06c0-.333-.143-.57-.429-.737l-5.97-3.473 1.95-1.118a.433.433 0 01.476 0l4.543 2.617c1.309.76 2.189 2.378 2.189 3.948 0 1.808-1.07 3.473-2.76 4.163zM7.802 12.703l-1.95-1.142c-.167-.095-.239-.238-.239-.428V5.899c0-2.545 1.95-4.472 4.591-4.472 1 0 1.927.333 2.712.928L8.23 5.067c-.285.166-.428.404-.428.737v6.898zM12 15.128l-2.795-1.57v-3.33L12 8.658l2.795 1.57v3.33L12 15.128zm1.796 7.23c-1 0-1.927-.332-2.712-.927l4.686-2.712c.285-.166.428-.404.428-.737v-6.898l1.974 1.142c.167.095.238.238.238.428v5.233c0 2.545-1.974 4.472-4.614 4.472zm-5.637-5.303l-4.544-2.617c-1.308-.761-2.188-2.378-2.188-3.948A4.482 4.482 0 014.21 6.327v5.423c0 .333.143.571.428.738l5.947 3.449-1.95 1.118a.432.432 0 01-.476 0zm-.262 3.9c-2.688 0-4.662-2.021-4.662-4.519 0-.19.024-.38.047-.57l4.686 2.71c.286.167.571.167.856 0l5.97-3.448v2.26c0 .19-.07.333-.237.428l-4.543 2.616c-.619.357-1.356.523-2.117.523zm5.899 2.83a5.947 5.947 0 005.827-4.756C22.287 18.339 24 15.84 24 13.296c0-1.665-.713-3.282-1.998-4.448.119-.5.19-.999.19-1.498 0-3.401-2.759-5.947-5.946-5.947-.642 0-1.26.095-1.88.31A5.962 5.962 0 0010.205 0a5.947 5.947 0 00-5.827 4.757C1.713 5.447 0 7.945 0 10.49c0 1.666.713 3.283 1.998 4.448-.119.5-.19 1-.19 1.499 0 3.401 2.759 5.946 5.946 5.946.642 0 1.26-.095 1.88-.309a5.96 5.96 0 004.162 1.713z" />
  </svg>
)

const ClaudeIcon = ({ className = "w-3.5 h-3.5 mr-1.5 shrink-0" }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor" fillRule="evenodd" style={{ flex: 'none', lineHeight: 1 }}>
    <path d="M4.709 15.955l4.72-2.647.08-.23-.08-.128H9.2l-.79-.048-2.698-.073-2.339-.097-2.266-.122-.571-.121L0 11.784l.055-.352.48-.321.686.06 1.52.103 2.278.158 1.652.097 2.449.255h.389l.055-.157-.134-.098-.103-.097-2.358-1.596-2.552-1.688-1.336-.972-.724-.491-.364-.462-.158-1.008.656-.722.881.06.225.061.893.686 1.908 1.476 2.491 1.833.365.304.145-.103.019-.073-.164-.274-1.355-2.446-1.446-2.49-.644-1.032-.17-.619a2.97 2.97 0 01-.104-.729L6.283.134 6.696 0l.996.134.42.364.62 1.414 1.002 2.229 1.555 3.03.456.898.243.832.091.255h.158V9.01l.128-1.706.237-2.095.23-2.695.08-.76.376-.91.747-.492.584.28.48.685-.067.444-.286 1.851-.559 2.903-.364 1.942h.212l.243-.242.985-1.306 1.652-2.064.73-.82.85-.904.547-.431h1.033l.76 1.129-.34 1.166-1.064 1.347-.881 1.142-1.264 1.7-.79 1.36.073.11.188-.02 2.856-.606 1.543-.28 1.841-.315.833.388.091.395-.328.807-1.969.486-2.309.462-3.439.813-.042.03.049.061 1.549.146.662.036h1.622l3.02.225.79.522.474.638-.079.485-1.215.62-1.64-.389-3.829-.91-1.312-.329h-.182v.11l1.093 1.068 2.006 1.81 2.509 2.33.127.578-.322.455-.34-.049-2.205-1.657-.851-.747-1.926-1.62h-.128v.17l.444.649 2.345 3.521.122 1.08-.17.353-.608.213-.668-.122-1.374-1.925-1.415-2.167-1.143-1.943-.14.08-.674 7.254-.316.37-.729.28-.607-.461-.322-.747.322-1.476.389-1.924.315-1.53.286-1.9.17-.632-.012-.042-.14.018-1.434 1.967-2.18 2.945-1.726 1.845-.414.164-.717-.37.067-.662.401-.589 2.388-3.036 1.44-1.882.93-1.086-.006-.158h-.055L4.132 18.56l-1.13.146-.487-.456.061-.746.231-.243 1.908-1.312-.006.006z" />
  </svg>
)

const OpenCodeIcon = ({ className = "w-3.5 h-3.5 mr-1.5 shrink-0" }: { className?: string }) => (
  <svg className={className} viewBox="0 0 24 24" fill="currentColor" fillRule="evenodd" style={{ flex: 'none', lineHeight: 1 }}>
    <path d="M16 6H8v12h8V6zm4 16H4V2h16v20z" />
  </svg>
)

function getTabIcon(tabName: string) {
  const name = tabName.toLowerCase()
  if (name.includes('pwsh') || name.includes('powershell')) {
    return (
      <svg className="w-3.5 h-3.5 text-blue-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2.5">
        <path strokeLinecap="round" strokeLinejoin="round" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
    )
  }
  if (name.includes('opencode')) {
    return <OpenCodeIcon className="w-3.5 h-3.5 mr-1.5 shrink-0 text-amber-500" />
  }
  if (name.includes('cmd') || name.includes('prompt')) {
    return (
      <svg className="w-3.5 h-3.5 text-slate-400 shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2.5">
        <path strokeLinecap="round" strokeLinejoin="round" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
      </svg>
    )
  }
  if (name.includes('node') || name.includes('npm')) {
    return (
      <span className="text-emerald-500 text-xs shrink-0 font-bold">⬢</span>
    )
  }
  return <span className="text-slate-500 text-xs shrink-0">🐚</span>
}

function App() {
  const [config, setConfig] = useState<Config>({
    default_flags: '--dangerously-bypass-approvals-and-sandbox',
    append_mode: true,
    default_ai: 'codex',
    entries: [],
    shell: 'pwsh'
  })
  const [selectedIndex, setSelectedIndex] = useState<number>(-1)
  const [status, setStatus] = useState<string>('系统就绪')
  const [windows, setWindows] = useState<WindowInfo[]>([])
  const [selectedWindow, setSelectedWindow] = useState<number | null>(null)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [codexSessions, setCodexSessions] = useState<CodexSession[]>([])
  const [showWtTabsPreview, setShowWtTabsPreview] = useState(false)
  const [searchText, setSearchText] = useState('')

  // 弹窗与历史会话状态
  const [isEditModalOpen, setIsEditModalOpen] = useState(false)
  const [editIndex, setEditIndex] = useState<number>(-1)
  const [showSessionDropdown, setShowSessionDropdown] = useState(false)

  // 详细对话记录状态
  const [sessionMessages, setSessionMessages] = useState<CodexMessage[]>([])
  const [isHistoryModalOpen, setIsHistoryModalOpen] = useState(false)
  const [isLoadingHistory, setIsLoadingHistory] = useState(false)

  // 全局默认 AI 切换，并同步覆盖所有项目的 AI 设置
  const handleDefaultAiChange = (ai: 'codex' | 'claude' | 'opencode') => {
    const newEntries = config.entries.map(e => ({ ...e, ai }))
    const newConfig = {
      ...config,
      default_ai: ai,
      entries: newEntries
    }
    setConfig(newConfig)
    saveConfig(newConfig)
    setStatus(`已将全局及所有项目的默认 AI 切换为 ${ai === 'claude' ? 'Claude' : ai === 'opencode' ? 'OpenCode' : 'Codex'}`)
  }

  const currentEntry = editIndex >= 0 ? config.entries[editIndex] : null
  const entryDir = currentEntry?.dir
  const entryAi = currentEntry?.ai

  // 监听当前编辑的项目的 AI/dir 变化以加载 sessions (支持 codex, claude, opencode)
  useEffect(() => {
    if (entryDir && entryAi) {
      invoke<CodexSession[]>('get_sessions', { dir: entryDir, ai: entryAi })
        .then(sessions => {
          setCodexSessions(sessions)
        })
        .catch(err => {
          console.error('获取历史会话失败:', err)
          setCodexSessions([])
        })
    } else {
      setCodexSessions([])
    }
  }, [editIndex, entryDir, entryAi])

  // 计算过滤后的列表
  const filteredEntries = config.entries.map((entry, originalIndex) => ({
    entry,
    originalIndex
  })).filter(({ entry }) => {
    const query = searchText.toLowerCase().trim()
    if (!query) return true
    return (
      entry.name.toLowerCase().includes(query) ||
      entry.dir.toLowerCase().includes(query)
    )
  })

  // 全选/全不选当前过滤出的列表
  const handleSelectAll = (select: boolean) => {
    const newEntries = [...config.entries]
    filteredEntries.forEach(({ originalIndex }) => {
      newEntries[originalIndex].enabled = select
    })
    const newConfig = { ...config, entries: newEntries }
    setConfig(newConfig)
    saveConfig(newConfig)
    setStatus(select ? '已全选过滤后的项目' : '已取消全选过滤后的项目')
  }

  // 刷新窗口列表
  const refreshWindows = async () => {
    setIsRefreshing(true)
    try {
      const w = await invoke<WindowInfo[]>('get_wt_windows')
      setWindows(w)
      if (w.length > 0) {
        if (selectedWindow === null || !w.some(item => item.id === selectedWindow)) {
          setSelectedWindow(w[0].id)
        }
      } else {
        setSelectedWindow(null)
      }
      setStatus('终端列表已更新')
    } catch (e) {
      console.error('获取窗口失败:', e)
      setWindows([])
      setSelectedWindow(null)
      setStatus('获取终端列表失败')
    } finally {
      setIsRefreshing(false)
    }
  }

  // 加载配置
  useEffect(() => {
    invoke<Config>('load_config')
      .then(cfg => {
        setConfig(cfg)
        if (cfg.append_mode) {
          invoke<WindowInfo[]>('get_wt_windows')
            .then(w => {
              setWindows(w)
              if (w.length > 0) {
                setSelectedWindow(w[0].id)
              }
            })
            .catch(console.error)
        }
      })
      .catch(console.error)
  }, [])

  // 保存配置
  const saveConfig = (cfg: Config) => {
    invoke('save_config', { config: cfg }).catch(console.error)
  }

  // 添加条目并直接打开弹窗编辑
  const handleAdd = async () => {
    const folder = await invoke<string>('pick_folder')
    if (!folder) return

    const name = folder.split(/[\\/]/).pop() || ''
    const newEntry: Entry = {
      name,
      dir: folder,
      session: '',
      enabled: true,
      ai: config.default_ai
    }
    const newConfig = {
      ...config,
      entries: [...config.entries, newEntry]
    }
    setConfig(newConfig)
    saveConfig(newConfig)
    
    const newIdx = newConfig.entries.length - 1
    setSelectedIndex(newIdx)
    setEditIndex(newIdx)
    setIsEditModalOpen(true)
    setStatus('已添加新项目，请配置细节')
  }

  // 删除条目
  const handleRemove = (idx: number) => {
    const newEntries = config.entries.filter((_, i) => i !== idx)
    const newConfig = { ...config, entries: newEntries }
    setConfig(newConfig)
    saveConfig(newConfig)
    setSelectedIndex(-1)
    setEditIndex(-1)
    setStatus('项目已移除')
  }

  // 更新条目
  const updateEntry = (idx: number, updates: Partial<Entry>) => {
    const newEntries = [...config.entries]
    newEntries[idx] = { ...newEntries[idx], ...updates }
    const newConfig = { ...config, entries: newEntries }
    setConfig(newConfig)
    saveConfig(newConfig)
  }

  // 打开编辑配置弹窗
  const openEditModal = (idx: number) => {
    setEditIndex(idx)
    setIsEditModalOpen(true)
  }

  // 加载并查看详细会话历史
  const viewSessionHistory = async (sessId: string) => {
    if (!sessId || !currentEntry) return
    setIsLoadingHistory(true)
    setIsHistoryModalOpen(true)
    try {
      const msgs = await invoke<CodexMessage[]>('get_session_details', { 
        sessionId: sessId,
        ai: currentEntry.ai
      })
      setSessionMessages(msgs)
    } catch (e) {
      console.error('获取会话详情失败:', e)
      setStatus('无法获取该会话详情')
      setSessionMessages([])
    } finally {
      setIsLoadingHistory(false)
    }
  }

  // 转换会话
  const handleConvertSession = async () => {
    if (!currentEditingEntry) return
    const { session, dir, ai } = currentEditingEntry
    if (!session || !dir) {
      setStatus('会话 ID 或项目目录为空，无法转换')
      return
    }

    setStatus('正在进行会话格式转换与导入...')
    try {
      if (ai === 'claude') {
        const msg = await invoke<string>('convert_claude_to_codex', { 
          sessionId: session, 
          cwd: dir 
        })
        updateEntry(editIndex, { ai: 'codex' })
        setStatus(msg)
      } else if (ai === 'codex') {
        const msg = await invoke<string>('convert_codex_to_claude', { 
          sessionId: session, 
          cwd: dir 
        })
        updateEntry(editIndex, { ai: 'claude' })
        setStatus(msg)
      }
      setIsHistoryModalOpen(false)
      setSessionMessages([])
    } catch (e) {
      console.error('转换会话失败:', e)
      setStatus(`转换失败: ${e}`)
    }
  }

  // 启动
  const handleLaunch = async () => {
    setStatus('正在激活并分发会话...')
    try {
      const msg = await invoke<string>('launch_wt', { 
        config,
        targetWindow: selectedWindow 
      })
      setStatus(msg)
      setTimeout(() => {
        refreshWindows()
      }, 1200)
    } catch (e) {
      setStatus(`启动失败: ${e}`)
    }
  }

  const activeWindowInfo = windows.find(w => w.id === selectedWindow)
  const currentEditingEntry = editIndex >= 0 ? config.entries[editIndex] : null

  return (
    <div className="h-screen bg-[#080a0f] text-slate-100 p-5 flex flex-col font-sans select-none relative overflow-hidden">
      
      {/* 背景霓虹光圈 */}
      <div className="absolute top-[-100px] left-[-100px] w-[350px] h-[350px] bg-indigo-600/10 rounded-full blur-[100px] pointer-events-none" />
      <div className="absolute bottom-[-150px] right-[-100px] w-[450px] h-[450px] bg-purple-600/10 rounded-full blur-[120px] pointer-events-none" />

      {/* 顶部彩色装饰线条 */}
      <div className="absolute top-0 left-0 right-0 h-[2px] bg-gradient-to-r from-indigo-500 via-purple-500 to-emerald-500 opacity-60" />

      {/* 头部标题区 */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6 pb-5 border-b border-[#1b2233]">
        <div className="flex items-center gap-3.5">
          <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-indigo-500/80 via-purple-600/80 to-purple-800/80 flex items-center justify-center shadow-lg shadow-indigo-950/40 border border-indigo-400/20">
            <svg className="w-5 h-5 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth="2">
              <path strokeLinecap="round" strokeLinejoin="round" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
          </div>
          <div>
            <h1 className="text-lg font-extrabold tracking-tight text-slate-100">
              WTLinker
            </h1>
            <p className="text-[11px] text-slate-500 font-medium mt-0.5">
              一键在 Windows Terminal 指定窗口中开辟 AI 标签页
            </p>
          </div>
        </div>
        
        {/* 系统状态条 */}
        <div className="flex items-center self-start sm:self-center gap-2 px-3 py-1.5 rounded-xl bg-[#0e121b]/80 border border-[#1b2233] text-[11px]">
          <span className={`w-1.5 h-1.5 rounded-full ${status.includes('失败') ? 'bg-red-500 animate-pulse' : 'bg-emerald-400 animate-pulse'}`} />
          <span className="text-slate-400 font-mono font-medium">{status}</span>
        </div>
      </div>

      {/* 主面板两栏布局 */}
      <div className="grid grid-cols-12 gap-5 flex-1 min-h-0 relative z-10">
        
        {/* 左栏：项目配置列表 */}
        <div className="col-span-5 flex flex-col min-h-0">
          <div className="flex items-center justify-between mb-3 px-1">
            <div className="flex items-center gap-2">
              <span className="text-xs font-bold text-slate-400 uppercase tracking-wider">项目配置列表</span>
              <span className="px-1.5 py-0.5 text-[9px] font-mono rounded-md bg-[#0e121b] border border-[#1b2233] text-slate-400">
                {config.entries.length} Items
              </span>
            </div>
          </div>

          <div className="flex-1 bg-[#0f131b] border border-[#1b2233] rounded-2xl overflow-hidden flex flex-col shadow-xl shadow-black/30 min-h-0">
            {/* 搜索与全选工具栏 */}
            <div className="p-2.5 bg-[#0b0e16]/80 border-b border-[#1b2233] flex flex-col sm:flex-row gap-2 shrink-0">
              <div className="relative flex-1">
                <input
                  type="text"
                  value={searchText}
                  onChange={e => setSearchText(e.target.value)}
                  placeholder="搜索项目名称或工作目录..."
                  className="w-full h-8 rounded-lg bg-[#080b11] border border-[#1b2233] focus:border-indigo-500 pl-8 pr-8 text-xs text-slate-200 focus:outline-none transition-all placeholder:text-slate-655"
                />
                <span className="absolute left-2.5 top-1/2 -translate-y-1/2 text-slate-500 text-[11px]">
                  🔍
                </span>
                {searchText && (
                  <button
                    onClick={() => setSearchText('')}
                    className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-350 text-[10px]"
                  >
                    ✕
                  </button>
                )}
              </div>
              <div className="flex gap-1.5 shrink-0">
                <button
                  onClick={() => handleSelectAll(true)}
                  className="px-2.5 h-8 rounded-lg bg-[#121622] hover:bg-[#181d2c] border border-[#222a3d] hover:border-[#354366] text-slate-300 hover:text-slate-100 text-[10px] font-bold transition-all active:scale-95 flex items-center gap-1"
                  title="启用当前过滤出的所有项目"
                >
                  ☑️ 全选
                </button>
                <button
                  onClick={() => handleSelectAll(false)}
                  className="px-2.5 h-8 rounded-lg bg-[#121622] hover:bg-[#181d2c] border border-[#222a3d] hover:border-[#354366] text-slate-300 hover:text-slate-100 text-[10px] font-bold transition-all active:scale-95 flex items-center gap-1"
                  title="禁用当前过滤出的所有项目"
                >
                  ⬛ 全不选
                </button>
              </div>
            </div>

            <div className="flex-1 overflow-y-auto p-2.5 space-y-1.5 custom-scrollbar">
              {config.entries.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-slate-650 p-8 text-center">
                  <span className="text-3xl mb-3">📁</span>
                  <p className="text-xs font-semibold text-slate-500">暂无项目目录</p>
                  <p className="text-[10px] text-slate-600 mt-1 max-w-[200px]">点击下方添加按钮，选择本地工作目录导入您的第一个项目。</p>
                </div>
              ) : filteredEntries.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-slate-650 p-8 text-center">
                  <span className="text-2xl mb-2">🔍</span>
                  <p className="text-xs font-semibold text-slate-500">未找到匹配项目</p>
                  <p className="text-[10px] text-slate-600 mt-1">请尝试更换搜索关键字</p>
                </div>
              ) : (
                filteredEntries.map(({ entry, originalIndex: idx }) => (
                  <div
                    key={idx}
                    onClick={() => setSelectedIndex(idx)}
                    className={`group relative flex items-center justify-between gap-3.5 p-3.5 rounded-xl border transition-all duration-200 cursor-pointer ${
                      idx === selectedIndex
                        ? 'bg-[#151c2b] border-indigo-500/50 shadow-lg shadow-indigo-950/20 translate-x-1'
                        : 'bg-[#121622] border-[#1b2233]/40 hover:bg-[#181d2c] hover:border-[#222a3d] hover:translate-x-0.5'
                    }`}
                  >
                    <div className="flex items-center gap-3.5 min-w-0 flex-1">
                      <input
                        type="checkbox"
                        checked={entry.enabled}
                        onChange={e => updateEntry(idx, { enabled: e.target.checked })}
                        onClick={e => e.stopPropagation()}
                        className="w-4 h-4 rounded-md border-[#222a3d] bg-[#090b11] text-indigo-650 focus:ring-0 cursor-pointer transition-all shrink-0"
                      />
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center justify-between gap-2">
                          <span className={`font-semibold text-xs truncate transition-all ${entry.enabled ? 'text-slate-200' : 'text-slate-650 line-through'}`}>
                            {entry.name || '(未命名)'}
                          </span>
                          <div className="flex items-center gap-1.5 shrink-0">
                            {entry.session && (
                              <span className="px-1.5 py-0.5 rounded-md bg-indigo-950/50 border border-indigo-900/40 text-[9px] font-mono text-indigo-400">
                                ID: {entry.session.slice(0, 8)}
                              </span>
                            )}
                            <span
                              className={`px-1.5 py-0.5 rounded-md text-[9px] font-bold tracking-wide text-white uppercase ${
                                entry.ai === 'claude'
                                  ? 'bg-[#d97757] shadow-[0_0_8px_rgba(217,119,87,0.25)] border border-[#d97757]/20'
                                  : entry.ai === 'opencode'
                                  ? 'bg-[#f59e0b] shadow-[0_0_8px_rgba(245,158,11,0.25)] border border-[#f59e0b]/20'
                                  : 'bg-[#10a37f] shadow-[0_0_8px_rgba(16,163,127,0.25)] border border-[#10a37f]/20'
                              }`}
                            >
                              {entry.ai === 'claude' ? 'Claude' : entry.ai === 'opencode' ? 'OpenCode' : 'Codex'}
                            </span>
                          </div>
                        </div>
                        <div className="text-[10px] text-slate-555 truncate mt-1.5 font-mono">
                          {entry.dir}
                        </div>
                      </div>
                    </div>

                    {/* 卡片右侧 hover 显示的编辑齿轮按钮 */}
                    <button
                      onClick={(e) => {
                        e.stopPropagation()
                        openEditModal(idx)
                      }}
                      className="opacity-0 group-hover:opacity-100 p-1.5 rounded-lg bg-[#0e121b] border border-[#1b2233] text-slate-400 hover:text-slate-200 hover:border-[#354366] transition-all shrink-0 ml-1 focus:outline-none"
                      title="编辑项目配置"
                    >
                      ⚙️
                    </button>
                  </div>
                ))
              )}
            </div>

            {/* 左侧操作栏 */}
            <div className="p-3 bg-[#0c1017]/80 border-t border-[#1b2233] flex gap-2 shrink-0">
              <button
                onClick={handleAdd}
                className="flex-1 h-9 rounded-xl bg-indigo-650 hover:bg-indigo-500 text-white text-xs font-semibold transition-all active:scale-[0.98] flex items-center justify-center gap-1.5 shadow-md border border-indigo-400/10"
              >
                <span>➕</span>
                <span>添加项目</span>
              </button>
              <button
                onClick={() => handleRemove(selectedIndex)}
                disabled={selectedIndex < 0}
                className={`flex-1 h-9 rounded-xl text-xs font-semibold border transition-all active:scale-[0.98] ${
                  selectedIndex >= 0
                    ? 'bg-red-950/20 border-red-900/40 text-red-300 hover:bg-red-900/30 hover:border-red-500/50 hover:text-white'
                    : 'bg-[#090b11] border-transparent text-slate-705 cursor-not-allowed'
                }`}
              >
                删除选中
              </button>
            </div>
          </div>
        </div>

        {/* 右栏：全局设置与启动 - 高度自适应，不再显示大滚动条 */}
        <div className="col-span-7 flex flex-col min-h-0 justify-between gap-5">
          
          {/* 全局设置 */}
          <div className="bg-[#0f131b] border border-[#1b2233] rounded-2xl p-5 shadow-xl shadow-black/30 shrink-0">
            <h2 className="text-[10px] font-bold text-slate-400 uppercase tracking-widest mb-4 flex items-center gap-1.5 border-b border-[#1b2233]/40 pb-2">
              <span>⚙️</span> Global Settings / 全局配置
            </h2>

            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 mb-4">
              {/* 默认 AI */}
              <div className="sm:col-span-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase mb-1.5 block">默认 AI 程序</label>
                <div className="flex gap-1.5 bg-[#080b11] p-1 border border-[#1b2233] rounded-xl">
                  <button
                    onClick={() => handleDefaultAiChange('codex')}
                    className={`flex-1 h-7 rounded-lg font-bold text-[10px] flex items-center justify-center transition-all ${
                      config.default_ai === 'codex'
                        ? 'bg-[#10a37f] text-white shadow-md border border-[#10a37f]/20'
                        : 'text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    <CodexIcon />
                    <span>Codex</span>
                  </button>
                  <button
                    onClick={() => handleDefaultAiChange('claude')}
                    className={`flex-1 h-7 rounded-lg font-bold text-[10px] flex items-center justify-center transition-all ${
                      config.default_ai === 'claude'
                        ? 'bg-[#d97757] text-white shadow-md border border-[#d97757]/20'
                        : 'text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    <ClaudeIcon />
                    <span>Claude</span>
                  </button>
                  <button
                    onClick={() => handleDefaultAiChange('opencode')}
                    className={`flex-1 h-7 rounded-lg font-bold text-[10px] flex items-center justify-center transition-all ${
                      config.default_ai === 'opencode'
                        ? 'bg-[#f59e0b] text-white shadow-md border border-[#f59e0b]/20'
                        : 'text-slate-400 hover:text-slate-200'
                    }`}
                  >
                    <OpenCodeIcon />
                    <span>OpenCode</span>
                  </button>
                </div>
              </div>

              {/* Shell 环境 */}
              <div className="sm:col-span-1">
                <label className="text-[10px] font-bold text-slate-500 uppercase mb-1.5 block">运行 Shell 环境</label>
                <select
                  value={config.shell}
                  onChange={e => {
                    const newConfig = { ...config, shell: e.target.value as any }
                    setConfig(newConfig)
                    saveConfig(newConfig)
                  }}
                  className="w-full h-9 rounded-xl bg-[#090b11] border border-[#1b2233] hover:border-[#222a3d] focus:border-indigo-500 px-2.5 text-xs text-slate-300 focus:outline-none transition-all font-mono"
                >
                  <option value="pwsh">pwsh (PowerShell 7)</option>
                  <option value="powershell">powershell (PS 5)</option>
                  <option value="cmd">cmd (Command Prompt)</option>
                </select>
              </div>

              {/* 默认参数 */}
              <div className="sm:col-span-2">
                <label className="text-[10px] font-bold text-slate-500 uppercase mb-1.5 block">全局附加参数</label>
                <input
                  type="text"
                  value={config.default_flags}
                  onChange={e => {
                    const newConfig = { ...config, default_flags: e.target.value }
                    setConfig(newConfig)
                    saveConfig(newConfig)
                  }}
                  placeholder="如: --dangerously..."
                  className="w-full h-9 rounded-xl bg-[#090b11] border border-[#1b2233] hover:border-[#222a3d] focus:border-indigo-500 focus:shadow-[0_0_10px_rgba(99,102,241,0.1)] px-3 text-xs text-slate-200 focus:outline-none transition-all font-mono"
                />
              </div>
            </div>

            {/* 追加模式开关 */}
            <div className="flex items-center justify-between p-3.5 rounded-xl bg-[#090b11]/80 border border-[#1b2233] mb-4">
              <div className="flex items-center gap-3">
                <div className="relative flex items-center">
                  <input
                    type="checkbox"
                    id="appendMode"
                    checked={config.append_mode}
                    onChange={e => {
                      const newConfig = { ...config, append_mode: e.target.checked }
                      setConfig(newConfig)
                      saveConfig(newConfig)
                      if (e.target.checked) {
                        refreshWindows()
                      }
                    }}
                    className="w-4 h-4 rounded-md border-[#222a3d] bg-[#090b11] text-indigo-650 focus:ring-0 cursor-pointer"
                  />
                </div>
                <div>
                  <label htmlFor="appendMode" className="text-xs font-bold text-slate-350 cursor-pointer block">
                    追加到已有的 Windows Terminal 窗口
                  </label>
                  <span className="text-[10px] text-slate-500 font-medium">
                    在选中的 Terminal 窗口内新建标签页，保持您的多项目开发工作流一致
                  </span>
                </div>
              </div>
            </div>

            {/* 追加选项下的窗口和标签页选择 */}
            {config.append_mode && (
              <div className="mt-3 pt-3 border-t border-[#1b2233]/40 space-y-4">
                <div className="flex items-center gap-3 min-w-0">
                  <span className="text-xs font-semibold text-slate-400 shrink-0">目标终端窗口:</span>
                  <select
                    value={selectedWindow || ''}
                    onChange={e => setSelectedWindow(e.target.value ? Number(e.target.value) : null)}
                    className="flex-1 min-w-0 h-8 rounded-lg bg-[#080a0f] border border-[#222a3d] px-2.5 text-xs text-slate-300 focus:border-indigo-500 focus:outline-none font-mono"
                  >
                    <option value="">自动选择当前激活窗口</option>
                    {windows.map(w => {
                      const displayTitle = w.title 
                        ? (w.title.length > 50 ? w.title.slice(0, 47) + '...' : w.title)
                        : `Windows Terminal [${w.id}]`;
                      return (
                        <option key={w.id} value={w.id}>
                          "{displayTitle}" (PID: {w.pid})
                        </option>
                      );
                    })}
                  </select>
                  <button
                    onClick={refreshWindows}
                    disabled={isRefreshing}
                    className="h-8 px-3 rounded-lg border border-[#222a3d] text-xs text-slate-300 bg-[#080a0f] hover:bg-[#121622] hover:border-[#354366] flex items-center gap-1.5 transition-all disabled:opacity-50 font-semibold shadow-md active:scale-95"
                  >
                    <span className={isRefreshing ? 'animate-spin' : ''}>🔄</span>
                    <span>刷新</span>
                  </button>
                </div>

                {activeWindowInfo && (
                  <div className="flex justify-end pt-1">
                    <button
                      onClick={() => setShowWtTabsPreview(!showWtTabsPreview)}
                      className="text-[10px] text-indigo-400 hover:text-indigo-300 transition-all font-mono font-bold flex items-center gap-1.5 focus:outline-none"
                    >
                      <span>{showWtTabsPreview ? '▼ 折叠终端标签预览' : '▶ 展开终端标签预览'}</span>
                      <span className="px-1.5 py-0.5 rounded bg-indigo-950/50 border border-indigo-900/40 text-[9px] font-medium text-indigo-400 font-mono">
                        {activeWindowInfo.tabs.length} Tabs
                      </span>
                    </button>
                  </div>
                )}

                {/* 仿真 Windows Terminal 标签页预览栏 */}
                {activeWindowInfo && showWtTabsPreview && (
                  <div className="bg-[#0b0d12] border border-[#1b2233] rounded-xl overflow-hidden shadow-2xl transition-all duration-200">
                    <div className="flex items-center justify-between px-3 py-2 bg-[#0d1017] border-b border-[#181f2f]/80 select-none">
                      <div className="flex items-center gap-2 min-w-0 mr-4">
                        <span className="w-2.5 h-2.5 rounded-full bg-red-500/80 shrink-0" />
                        <span className="w-2.5 h-2.5 rounded-full bg-yellow-500/80 shrink-0" />
                        <span className="w-2.5 h-2.5 rounded-full bg-green-500/80 shrink-0" />
                        <span className="text-[10px] text-slate-400 font-bold font-mono ml-2 truncate" title={activeWindowInfo.title}>
                          {activeWindowInfo.title || 'Windows Terminal'}
                        </span>
                      </div>
                      <span className="text-[9px] font-mono text-slate-500 bg-[#090c12] px-2 py-0.5 rounded border border-[#1b2233]/20 shrink-0">
                        PID: {activeWindowInfo.pid}
                      </span>
                    </div>

                    <div className="px-2 pt-2 bg-[#090b10] flex items-end gap-1 overflow-x-auto custom-scrollbar">
                      {activeWindowInfo.tabs.length === 0 ? (
                        <div className="text-[10px] text-slate-650 italic px-3 py-2">
                          没有活动中的标签页，或该窗口无法读取
                        </div>
                      ) : (
                        activeWindowInfo.tabs.map((tabName, tIdx) => {
                          const isDummyActive = tIdx === 0;
                          return (
                            <div
                              key={tIdx}
                              className={`shrink-0 flex items-center gap-2 px-3 py-1.5 rounded-t-lg text-[10px] font-mono border-t border-x transition-all duration-150 ${
                                isDummyActive
                                  ? 'bg-[#151c2b] border-[#222a3d] text-slate-100 shadow-[inset_0_1px_0_rgba(255,255,255,0.05)]'
                                  : 'bg-[#0b0e16]/60 border-transparent text-slate-500 hover:bg-[#121622] hover:text-slate-300'
                              }`}
                            >
                              {getTabIcon(tabName)}
                              <span className="max-w-[110px] truncate font-medium">{tabName}</span>
                              <span className="text-[8px] text-slate-650 hover:text-red-400 cursor-pointer ml-1">✕</span>
                            </div>
                          )
                        })
                      )}
                    </div>
                  </div>
                )}
              </div>
            )}
          </div>

          {/* 启动栏 */}
          <div className="flex items-center justify-end gap-4 mt-auto shrink-0 pt-4">
            <button
              onClick={handleLaunch}
              disabled={config.entries.filter(e => e.enabled).length === 0}
              className="w-full sm:w-auto h-11 px-8 rounded-xl bg-gradient-to-r from-indigo-500 via-purple-600 to-indigo-650 text-white text-xs font-bold hover:from-indigo-400 hover:to-purple-500 shadow-lg shadow-indigo-600/10 active:scale-[0.98] transition-all disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2 border border-indigo-400/20"
            >
              <span>🚀</span>
              <span>分发并启动选中 AI 会话</span>
            </button>
          </div>
        </div>
      </div>

      {/* 配置编辑模态弹窗 (Modal) */}
      {isEditModalOpen && currentEditingEntry && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm">
          <div className="relative w-full max-w-lg bg-[#0f131b] border border-[#1b2233] rounded-2xl shadow-2xl p-6 overflow-visible flex flex-col gap-5 text-slate-200">
            
            {/* 弹窗头部 */}
            <div className="flex items-center justify-between pb-3 border-b border-[#1b2233]/60">
              <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                <span>⚙️</span> 编辑项目配置
              </h3>
              <button
                onClick={() => {
                  setIsEditModalOpen(false)
                  setShowSessionDropdown(false)
                }}
                className="text-slate-500 hover:text-slate-350 transition-colors text-lg"
              >
                ✕
              </button>
            </div>

            {/* 弹窗表单 */}
            <div className="space-y-4 text-left">
              {/* 项目名称 */}
              <div>
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wide mb-1.5 block">项目别名</label>
                <input
                  type="text"
                  value={currentEditingEntry.name}
                  onChange={e => updateEntry(editIndex, { name: e.target.value })}
                  className="w-full h-9 rounded-xl bg-[#080b11] border border-[#1b2233] focus:border-indigo-500 px-3 text-xs text-slate-200 focus:outline-none transition-all"
                  placeholder="项目显示别名"
                />
              </div>

              {/* 工作目录 */}
              <div>
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wide mb-1.5 block">工作目录</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    readOnly
                    value={currentEditingEntry.dir}
                    className="flex-1 h-9 rounded-xl bg-[#080b11]/50 border border-[#1b2233]/60 px-3 text-xs text-slate-400 focus:outline-none font-mono truncate"
                  />
                  <button
                    onClick={async () => {
                      const folder = await invoke<string>('pick_folder')
                      if (folder) {
                        const name = folder.split(/[\\/]/).pop() || ''
                        updateEntry(editIndex, { dir: folder, name: name })
                      }
                    }}
                    className="h-9 px-4 rounded-xl border border-[#222a3d] text-xs text-slate-300 hover:bg-[#121622] hover:border-[#354366] transition-all font-semibold shadow-md active:scale-95"
                  >
                    浏览
                  </button>
                </div>
              </div>

              {/* AI 程序选择 */}
              <div>
                <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wide mb-1.5 block">专属 AI 程序</label>
                <div className="flex gap-2 bg-[#080b11] p-1 border border-[#1b2233] rounded-xl">
                  <button
                    type="button"
                    onClick={() => updateEntry(editIndex, { ai: 'codex' })}
                    className={`flex-1 h-8 rounded-lg font-bold text-[10px] flex items-center justify-center gap-1.5 transition-all ${
                      currentEditingEntry.ai === 'codex'
                        ? 'bg-[#10a37f] text-white shadow-md border border-[#10a37f]/20'
                        : 'text-slate-400 hover:text-slate-250'
                    }`}
                  >
                    <CodexIcon />
                    <span>Codex</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => updateEntry(editIndex, { ai: 'claude' })}
                    className={`flex-1 h-8 rounded-lg font-bold text-[10px] flex items-center justify-center gap-1.5 transition-all ${
                      currentEditingEntry.ai === 'claude'
                        ? 'bg-[#d97757] text-white shadow-md border border-[#d97757]/20'
                        : 'text-slate-400 hover:text-slate-250'
                    }`}
                  >
                    <ClaudeIcon />
                    <span>Claude</span>
                  </button>
                  <button
                    type="button"
                    onClick={() => updateEntry(editIndex, { ai: 'opencode' })}
                    className={`flex-1 h-8 rounded-lg font-bold text-[10px] flex items-center justify-center gap-1.5 transition-all ${
                      currentEditingEntry.ai === 'opencode'
                        ? 'bg-[#f59e0b] text-white shadow-md border border-[#f59e0b]/20'
                        : 'text-slate-400 hover:text-slate-250'
                    }`}
                  >
                    <OpenCodeIcon />
                    <span>OpenCode</span>
                  </button>
                </div>
              </div>

              {/* 会话 ID (Resume) */}
              {currentEditingEntry && (
                <div>
                  <label className="text-[10px] font-bold text-slate-500 uppercase tracking-wide mb-1.5 block">
                    会话 ID (Resume)
                  </label>
                  
                  <div className="relative">
                    {/* 点击外面关闭的遮罩层 */}
                    {showSessionDropdown && (
                      <div
                        className="fixed inset-0 z-40 bg-transparent"
                        onClick={() => setShowSessionDropdown(false)}
                      />
                    )}

                    <div className="relative z-50 flex items-center gap-2">
                      <div className="relative flex-1">
                        <input
                          type="text"
                          value={currentEditingEntry.session}
                          onChange={e => updateEntry(editIndex, { session: e.target.value })}
                          onFocus={() => setShowSessionDropdown(true)}
                          placeholder="输入或选择历史会话 ID"
                          className="w-full h-9 rounded-xl bg-[#080b11] border border-[#1b2233] focus:border-indigo-500 pl-3 pr-10 text-xs text-slate-200 focus:outline-none transition-all font-mono"
                        />
                        <button
                          type="button"
                          onClick={() => setShowSessionDropdown(!showSessionDropdown)}
                          className="absolute right-2.5 top-1/2 -translate-y-1/2 text-slate-500 hover:text-slate-350 focus:outline-none"
                        >
                          <span className={`inline-block transition-transform duration-200 ${showSessionDropdown ? 'rotate-180' : ''}`}>
                            ▼
                          </span>
                        </button>
                      </div>
                      <button
                        type="button"
                        disabled={!currentEditingEntry.session}
                        onClick={() => viewSessionHistory(currentEditingEntry.session)}
                        className={`h-9 w-9 shrink-0 rounded-xl flex items-center justify-center border transition-all ${
                          currentEditingEntry.session
                            ? 'bg-[#121622] border-[#222a3d] text-slate-350 hover:bg-[#181d2c] hover:border-[#354366] hover:text-slate-100 active:scale-95'
                            : 'bg-[#080b11]/30 border-transparent text-slate-700 cursor-not-allowed'
                        }`}
                        title="查看会话详细聊天记录"
                      >
                        📖
                      </button>
                    </div>

                    {/* 下拉面板 */}
                    {showSessionDropdown && codexSessions.length > 0 && (
                      <div className="absolute z-[100] left-0 right-0 mt-1 max-h-60 overflow-y-auto bg-[#0d1017] border border-[#1b2233] rounded-xl shadow-2xl custom-scrollbar">
                        {codexSessions.map(sess => (
                          <div
                            key={sess.id}
                            onClick={() => {
                              updateEntry(editIndex, { session: sess.id })
                              setShowSessionDropdown(false)
                            }}
                            className="p-3 hover:bg-[#151c2b] border-b border-[#1b2233]/40 last:border-0 cursor-pointer transition-colors text-left"
                          >
                            <div className="flex justify-between items-center text-[11px] font-mono">
                              <span className="text-indigo-400 font-bold truncate max-w-[220px]" title={sess.id}>
                                {sess.id}
                              </span>
                              <span className="text-slate-500 font-medium text-[9px] shrink-0 ml-2">
                                {sess.timestamp}
                              </span>
                            </div>
                            {sess.preview && (
                              <div className="text-[10px] text-slate-400 mt-1.5 truncate">
                                💬 {sess.preview}
                              </div>
                            )}
                          </div>
                        ))}
                      </div>
                    )}

                    {showSessionDropdown && codexSessions.length === 0 && (
                      <div className="absolute z-[100] left-0 right-0 mt-1 p-3 bg-[#0d1017] border border-[#1b2233] rounded-xl text-center text-[10px] text-slate-500">
                        未检测到该项目的历史会话
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>

            {/* 弹窗底部 */}
            <div className="flex justify-end pt-3 border-t border-[#1b2233]/60">
              <button
                onClick={() => {
                  setIsEditModalOpen(false)
                  setShowSessionDropdown(false)
                }}
                className="h-9 px-6 rounded-xl bg-indigo-650 hover:bg-indigo-500 text-white text-xs font-bold transition-all active:scale-[0.98] border border-indigo-400/10"
              >
                保存并关闭
              </button>
            </div>

          </div>
        </div>
      )}

      {/* 历史记录对话流弹窗 */}
      {isHistoryModalOpen && (
        <div className="fixed inset-0 z-[60] flex items-center justify-center bg-black/85 backdrop-blur-md">
          <div className="relative w-full max-w-2xl h-[500px] bg-[#0e121b] border border-[#1b2233] rounded-2xl shadow-2xl p-5 flex flex-col gap-4 text-slate-200">
            
            {/* 头部 */}
            <div className="flex items-center justify-between pb-3 border-b border-[#1b2233]/65 shrink-0 text-left">
              <div>
                <h3 className="text-sm font-bold text-slate-100 flex items-center gap-2">
                  <span>💬</span> 会话详细聊天记录
                </h3>
                {currentEditingEntry && (
                  <p className="text-[10px] text-slate-500 font-mono mt-0.5 truncate max-w-[450px]">
                    ID: {currentEditingEntry.session}
                  </p>
                )}
              </div>
              <button
                onClick={() => {
                  setIsHistoryModalOpen(false)
                  setSessionMessages([])
                }}
                className="text-slate-500 hover:text-slate-350 transition-colors text-lg"
              >
                ✕
              </button>
            </div>

            {/* 对话气泡内容区域 */}
            <div className="flex-1 overflow-y-auto p-2 space-y-4 custom-scrollbar text-xs">
              {isLoadingHistory ? (
                <div className="h-full flex flex-col items-center justify-center text-slate-500 gap-2 font-mono">
                  <span className="animate-spin text-lg">⏳</span>
                  <span>正在读取并解析会话日志...</span>
                </div>
              ) : sessionMessages.length === 0 ? (
                <div className="h-full flex flex-col items-center justify-center text-slate-500 gap-2 font-mono">
                  <span>📭</span>
                  <span>暂无有效的真实对话记录</span>
                </div>
              ) : (
                sessionMessages.map((msg, mIdx) => {
                  const isUser = msg.role === 'user'
                  return (
                    <div
                      key={mIdx}
                      className={`flex flex-col ${isUser ? 'items-end' : 'items-start'} max-w-full`}
                    >
                      <div className="text-[9px] text-slate-500 font-bold mb-1 px-1 font-sans">
                        {isUser ? '👤 USER (您)' : '🤖 ASSISTANT (AI)'}
                      </div>
                      <div
                        className={`p-3 rounded-2xl max-w-[85%] border font-sans leading-relaxed break-words whitespace-pre-wrap text-left ${
                          isUser
                            ? 'bg-[#151c2b] border-indigo-900/60 text-slate-100 rounded-tr-none'
                            : 'bg-[#090b11] border-[#1b2233]/40 text-slate-300 rounded-tl-none'
                        }`}
                      >
                        {msg.content}
                      </div>
                    </div>
                  )
                })
              )}
            </div>

            {/* 底部按钮 */}
            <div className="flex justify-between items-center pt-3 border-t border-[#1b2233]/65 shrink-0">
              <div>
                {currentEditingEntry && (currentEditingEntry.ai === 'claude' || currentEditingEntry.ai === 'codex') && (
                  <button
                    onClick={handleConvertSession}
                    className="h-9 px-4 rounded-xl bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white text-xs font-bold transition-all active:scale-[0.98] border border-indigo-400/10 shadow-lg shadow-indigo-950/20"
                  >
                    {currentEditingEntry.ai === 'claude' ? '➡️ 导入并转换为 Codex 会话' : '➡️ 导入并转换为 Claude 会话'}
                  </button>
                )}
              </div>
              <button
                onClick={() => {
                  setIsHistoryModalOpen(false)
                  setSessionMessages([])
                }}
                className="h-9 px-6 rounded-xl bg-indigo-650 hover:bg-indigo-500 text-white text-xs font-bold transition-all active:scale-[0.98] border border-indigo-400/10"
              >
                关闭详情
              </button>
            </div>

          </div>
        </div>
      )}

    </div>
  )
}

export default App