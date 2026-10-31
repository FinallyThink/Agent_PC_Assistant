import {useEffect, useRef, useState} from 'react';
import {invoke, isTauri} from '@tauri-apps/api/core';
function validAddress(address){try{const url=new URL(address);return url.protocol==='http:'&&['127.0.0.1','[::1]'].includes(url.hostname)&&!url.username&&!url.password&&url.pathname==='/'&&!url.search&&!url.hash;}catch{return false;}}
const storageKey = 'agent-pc-assistant.connections.v1';
export default function Connections(){
 const [connections,setConnections]=useState(()=>{
  try { const value=JSON.parse(localStorage.getItem(storageKey)||'[]'); return Array.isArray(value)?value.filter(x=>x&&typeof x.id==='string'&&typeof x.name==='string'&&typeof x.address==='string'&&validAddress(x.address)).slice(0,20):[]; }catch{return [];}
 });
 const [name,setName]=useState(''); const [address,setAddress]=useState('http://127.0.0.1:9881');
 const [feedback,setFeedback]=useState(''); const [results,setResults]=useState({});
 const active=useRef(new Map());
 useEffect(()=>()=>active.current.clear(),[]);
 function save(next){try{localStorage.setItem(storageKey,JSON.stringify(next));setConnections(next);setFeedback('已保存到此设备');return true;}catch{setFeedback('保存失败，请检查本地存储；输入已保留');return false;}}
 function add(event){event.preventDefault();
  if(!name.trim())return setFeedback('请填写连接名称');
  if(connections.length>=20)return setFeedback('最多保存 20 个连接');
  try{const url=new URL(address);if(url.protocol!=='http:'||!['127.0.0.1','[::1]'].includes(url.hostname)||url.username||url.password||url.pathname!=='/'||url.search||url.hash)throw Error();}catch{return setFeedback('请输入本机 HTTP 地址，不包含路径或凭据');}
  if(connections.some(x=>new URL(x.address).origin===new URL(address).origin))return setFeedback('该地址已经存在');
  if(save([...connections,{id:crypto.randomUUID(),name:name.trim(),address:new URL(address).origin}]))setName('');
 }
 async function check(connection){
  const request=crypto.randomUUID();active.current.set(connection.id,request);
  setResults(previous=>({...previous,[connection.id]:{status:'checking',message:'正在检查…'}}));
  try{
   if(!isTauri())throw Error('请在 Tauri 桌面应用中测试连接，浏览器预览不会访问本地服务');
   const result=await invoke('check_connection',{address:connection.address});
   if(active.current.get(connection.id)===request)setResults(previous=>({...previous,[connection.id]:result}));
  }catch(error){if(active.current.get(connection.id)===request)setResults(previous=>({...previous,[connection.id]:{status:'error',message:String(error)}}));}
  finally{if(active.current.get(connection.id)===request)active.current.delete(connection.id);}
 }
 const labels={online:'在线',offline:'离线',unknown:'状态未知',error:'检查失败',checking:'检查中'};
 return <><section><h2>连接本地服务</h2><p>通过桌面后端检查 GET /health。当前仅支持无需鉴权的健康检查，业务能力调用尚未接入。</p><form onSubmit={add} className="connection-form"><label>连接名称<input maxLength={60} value={name} onChange={e=>setName(e.target.value)} placeholder="例如：本地语音服务" required/></label><label>服务地址<input value={address} onChange={e=>setAddress(e.target.value)} required spellCheck={false}/></label><button type="submit">保存连接</button></form><p role="status">{feedback}</p></section><div className="connection-list">{connections.length===0?<section><h3>还没有连接</h3><p>添加本机服务后可测试可达性。界面不会自动启动模型或服务。</p></section>:connections.map(connection=><section key={connection.id}><span className="tag">{labels[results[connection.id]?.status]||'未检查'}</span><h3>{connection.name}</h3><code>{connection.address}</code><p role="status">{results[connection.id]?.message||'尚未发起健康检查'}</p><div className="actions"><button disabled={results[connection.id]?.status==='checking'} onClick={()=>check(connection)}>测试连接</button><button onClick={()=>{if(save(connections.filter(x=>x.id!==connection.id)))active.current.delete(connection.id);}}>删除</button></div></section>)}</div></>;
}

