import {useEffect,useRef,useState} from 'react';
import {Menu} from '@tauri-apps/api/menu';
import {isTauri} from '@tauri-apps/api/core';
const base='/characters/deepseek/';
const animations={wave:{frames:['wave-1.png','wave-2.png','wave-3.png','wave-4.png'],fps:5},blink:{frames:['blink/open.png','blink/half.png','blink/closed.png','blink/closed.png','blink/half.png','blink/open.png'],fps:12}};
export default function Character({onDrag,onConsole,onHide}){
 const menuRef=useRef(null);const callbacks=useRef({onConsole,onHide});callbacks.current={onConsole,onHide};
 const [menuError,setMenuError]=useState('');
 const [action,setAction]=useState('idle');const [frame,setFrame]=useState('avatar.png');const [error,setError]=useState(false);
 useEffect(()=>{const timer=setInterval(()=>setAction(current=>current==='idle'?'blink':current),6000);return()=>clearInterval(timer);},[]);
 useEffect(()=>{setError(false);const animation=animations[action];if(!animation){setFrame(action==='think'?'think.png':action==='confused'?'confused.png':'avatar.png');return;}
  let index=0;setFrame(animation.frames[0]);const timer=setInterval(()=>{index+=1;if(index>=animation.frames.length){clearInterval(timer);setAction('idle');}else setFrame(animation.frames[index]);},1000/animation.fps);return()=>clearInterval(timer);
 },[action]);
 useEffect(()=>{
  if(!isTauri())return;
  let disposed=false;let resource;
  Menu.new({items:[{text:'动作',items:[['idle','待机'],['wave','挥手'],['think','思考'],['confused','疑惑']].map(([id,text])=>({id,text,action:()=>setAction(id)}))},...(onConsole?[{id:'console',text:'控制台',action:()=>callbacks.current.onConsole?.()}]:[]),...(onHide?[{id:'hide',text:'收起',action:()=>callbacks.current.onHide?.()}]:[])]}).then(menu=>{resource=menu;if(disposed)menu.close();else menuRef.current=menu;}).catch(e=>{if(!disposed)setMenuError(String(e));});
  return()=>{disposed=true;menuRef.current=null;resource?.close().catch(()=>{});};
 },[Boolean(onConsole),Boolean(onHide)]);
 async function openMenu(event){event.preventDefault();event.stopPropagation();if(!isTauri()){setMenuError('请在桌面版右键测试模型菜单');return;}try{if(!menuRef.current)throw Error('菜单正在准备，请重试');await menuRef.current.popup();setMenuError('');}catch(e){setMenuError(String(e));}}
 return <div className="character"><button className="character-drag" aria-label="澜鲸，左键拖动，右键打开菜单" onContextMenu={openMenu} onKeyDown={e=>{if(e.key==='ContextMenu'||(e.shiftKey&&e.key==='F10'))openMenu(e);}} onPointerDown={e=>{if(e.button===0&&onDrag)onDrag();}}><img src={base+frame} alt="澜鲸角色" draggable={false} onError={()=>setError(true)}/></button>{error&&<p role="alert">角色资源加载失败</p>}{menuError&&<p role="alert">{menuError}</p>}</div>;
}

