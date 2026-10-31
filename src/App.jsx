import Chat from './Chat.jsx';
import Voice from './Voice.jsx';
import Desktop from './Desktop.jsx';
import Tasks from './Tasks.jsx';
import Connections from './Connections.jsx';
import {useState} from 'react';
const items = [
 ['聊天与上下文','迁移 DeepSeek 对话、历史、取消；为待处理消息设置上限。','旧版问题'],
 ['语音输入与输出','保留系统语音、本地 GPU 音色与 F8 输入；模型环境独立验证。','待迁移'],
 ['桌面交互','迁移透明悬浮、拖动缩放、置顶、穿透和托盘恢复。','待迁移'],
 ['屏幕陪伴','提供暂停、忙碌跳过与内存截图；保留隐私说明。','待迁移'],
 ['本地能力网关','按能力注册和路由 HTTP、MCP、CLI、IPC，绑定目标实例。','新需求'],
 ['任务管理','支持进度、取消与超时未知状态，隔离迟到响应。','新需求'],
 ['设置与恢复','保存失败保留输入，明确保存结果，重启恢复配置。','旧版问题'],
 ['角色与动作','迁移现有非大肥鱼角色、动作表情与待机恢复。','待迁移'],
];
export default function App(){
 const [page,setPage]=useState('开发总览');
 const [filter,setFilter]=useState('全部');
 return <div className="shell"><aside><div className="brand">A<span>Agent PC Assistant</span></div><small>个人桌面助手 / 新版工作台</small><nav>{['开发总览','连接管理','桌面交互','聊天','语音输入与输出','任务管理','能力清单','架构说明'].map(x=><button className={page===x?'active':''} key={x} onClick={()=>setPage(x)}>{x}</button>)}</nav><div className="aside-note">TAURI 2 · REACT<br/>独立开发环境</div></aside><main><header><div><small>WORKSPACE / SECONDARY DEVELOPMENT</small><h1>{page}</h1></div><span className="badge">开发中 · 本地 HTTP 已接入</span></header>{page==='聊天'?<Chat/>:page==='语音输入与输出'?<Voice/>:page==='桌面交互'?<Desktop/>:page==='任务管理'?<Tasks/>:page==='连接管理'?<Connections/>:page==='架构说明'?<section><h2>面向能力的本地接口</h2><p>React 控制台 → Tauri 命令白名单 → 本地能力网关 → 协议适配器 → 本地服务</p><p>凭据与执行权限由后端管理。旧版 Electron preload 与 IPC 需要逐项迁移，不能直接运行。</p><p>超时不等于远端停止；写操作必须先查询结果，避免盲目重试。</p></section>:<><section className="intro"><small>新版开发资料 · 2026.11.01</small><h2>让陪伴与执行，在一个工作台协作。</h2><p>以现有能力为基线，逐项修复旧问题、迁移桌面交互，并建立多本地接口接入。</p><p>当前为开发范围与验收入口，以下能力均尚未完成接入。</p></section><div className="filters">{['全部','旧版问题','新需求','待迁移'].map(x=><button className={filter===x?'active':''} onClick={()=>setFilter(x)} key={x}>{x}</button>)}</div><div className="grid">{items.filter(x=>filter==='全部'||x[2]===filter).map(([name,description,status])=><section key={name}><span className="tag">{status}</span><h3>{name}</h3><p>{description}</p><small>待实现 · 待验收</small></section>)}</div></>}<footer>开发范围已排除大肥鱼形象制作与接入。语音服务按独立能力评估。</footer></main></div>
}





