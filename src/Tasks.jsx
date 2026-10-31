import {useState,useRef,useEffect} from 'react';
import {invoke,isTauri} from '@tauri-apps/api/core';
const labels={running:'执行中',succeeded:'已完成',failed:'失败',unknown:'结果未知',detached:'已停止等待'};
export default function Tasks(){
 const [connections]=useState(()=>{try{const list=JSON.parse(localStorage.getItem('agent-pc-assistant.connections.v1')||'[]');return Array.isArray(list)?list.filter(x=>x&&typeof x.id==='string'&&typeof x.address==='string'):[];}catch{return [];}});
 const [target,setTarget]=useState(connections[0]?.id||'');const [capability,setCapability]=useState('service.health');
 const [tasks,setTasks]=useState([]);const [error,setError]=useState('');const active=useRef(new Set());const pending=useRef(0);
 useEffect(()=>()=>active.current.clear(),[]);
 function update(id,result){setTasks(previous=>previous.map(task=>task.id===id?{...task,...result}:task));}
 async function run(event){event.preventDefault();setError('');
  const connection=connections.find(x=>x.id===target);
  if(!connection)return setError('请先添加并选择连接');
  if(!isTauri())return setError('请在 Tauri 桌面应用中执行，浏览器预览不模拟任务');
  if(pending.current>=4)return setError('最多同时执行 4 个任务');
  const id=crypto.randomUUID();active.current.add(id);pending.current+=1;
  setTasks(previous=>[{id,connection:connection.name,address:connection.address,capability,status:'running',time:new Date().toLocaleTimeString()},...previous].slice(0,50));
  try{const result=await invoke('call_capability',{address:connection.address,capability});if(active.current.has(id))update(id,result);}
  catch(e){if(active.current.has(id))update(id,{status:'failed',message:String(e)});}
  finally{active.current.delete(id);pending.current-=1;}
 }
 return <><section><h2>只读能力调用</h2><p>显式绑定本机连接。支持健康信息与能力清单 JSON 读取，5 秒超时，不自动重试。最多保留本页的 50 条记录，离开页面后清空。</p><form onSubmit={run} className="connection-form"><label>目标连接<select value={target} onChange={e=>setTarget(e.target.value)}><option value="">选择连接</option>{connections.map(x=><option key={x.id} value={x.id}>{x.name} · {x.address}</option>)}</select></label><label>能力<select value={capability} onChange={e=>setCapability(e.target.value)}><option value="service.health">健康信息 · GET /health</option><option value="service.capabilities">能力清单 · GET /capabilities</option></select></label><button disabled={!target}>执行读取</button></form><p role="alert">{error}</p></section><div className="connection-list">{tasks.length===0?<section><p>暂无任务。先选择连接，再执行读取。</p></section>:tasks.map(task=><section key={task.id}><span className="tag">{labels[task.status]}</span><h3>{task.capability}</h3><small>{task.connection} · {task.address} · {task.time}</small><p role="status">{task.message||'正在等待服务响应'}</p>{task.status==='running'&&<button onClick={()=>{active.current.delete(task.id);update(task.id,{status:'detached',message:'仅停止界面等待，未确认远端停止；迟到结果不会更新此记录。'});}}>停止等待</button>}{task.data!==undefined&&task.data!==null&&<pre>{JSON.stringify(task.data,null,2)}</pre>}</section>)}</div></>;
}

