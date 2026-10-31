import Character from './Character.jsx';
import {useState,useEffect} from 'react';
import {invoke,isTauri} from '@tauri-apps/api/core';
export default function Desktop({floating=false}){
 const [state,setState]=useState(null);const [error,setError]=useState('');
 async function command(action,value){try{if(!isTauri())throw Error('桌面交互需要运行 Tauri 桌面版');setState(await invoke('desktop_control',{action,value}));setError('');}catch(e){setError(String(e));}}
 useEffect(()=>{command('state');},[]);
 if(floating)return <div className="pet-stage"><Character onDrag={()=>command('drag')}/><div className="pet-tools"><button onClick={()=>command('console')}>控制台</button><button onClick={()=>command('hide')}>收起</button></div><p role="alert">{error}</p></div>;
 return <section><h2>桌面悬浮窗 · 澜鲸</h2><div className="character-preview"><Character/></div><p>按住澜鲸立绘可拖动窗口。关闭窗口会收起到托盘，托盘菜单可恢复窗口或退出应用。</p><div className="actions"><button onClick={()=>command('show')}>显示悬浮窗</button><button onClick={()=>command('hide')}>收起到托盘</button><button onClick={()=>command('state')}>刷新状态</button></div>{state&&<><p>窗口：{state.visible?'显示':'隐藏'} · 置顶：{state.always_on_top?'开':'关'} · 穿透：{state.click_through?'开':'关'}</p><div className="actions"><button onClick={()=>command('top',state.always_on_top?0:1)}>{state.always_on_top?'取消置顶':'设为置顶'}</button><button onClick={()=>command('through',state.click_through?0:1)}>{state.click_through?'关闭鼠标穿透':'开启鼠标穿透'}</button></div><label className="scale-label">窗口缩放 {state.scale}%<input aria-label="窗口缩放" type="range" min="60" max="160" step="10" value={state.scale} onChange={e=>command('scale',Number(e.target.value))}/></label></>}<p>开启穿透后，悬浮窗不接收鼠标。可在此关闭，或使用托盘“恢复悬浮窗”恢复操作。已迁入澜鲸 PNG 立绘，支持挥手、思考、疑惑与自动眨眼；并非 Cubism 模型。</p><p role="alert">{error}</p></section>;
}

