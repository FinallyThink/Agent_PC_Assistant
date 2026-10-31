import {useEffect,useState} from 'react';
const base='/characters/deepseek/';
const animations={wave:{frames:['wave-1.png','wave-2.png','wave-3.png','wave-4.png'],fps:5},blink:{frames:['blink/open.png','blink/half.png','blink/closed.png','blink/closed.png','blink/half.png','blink/open.png'],fps:12}};
export default function Character({onDrag}){
 const [action,setAction]=useState('idle');const [frame,setFrame]=useState('avatar.png');const [error,setError]=useState(false);
 useEffect(()=>{const timer=setInterval(()=>setAction(current=>current==='idle'?'blink':current),6000);return()=>clearInterval(timer);},[]);
 useEffect(()=>{setError(false);const animation=animations[action];if(!animation){setFrame(action==='think'?'think.png':action==='confused'?'confused.png':'avatar.png');return;}
  let index=0;setFrame(animation.frames[0]);const timer=setInterval(()=>{index+=1;if(index>=animation.frames.length){clearInterval(timer);setAction('idle');}else setFrame(animation.frames[index]);},1000/animation.fps);return()=>clearInterval(timer);
 },[action]);
 return <div className="character"><button className="character-drag" aria-label="澜鲸，按住拖动" onPointerDown={e=>{if(e.button===0&&onDrag)onDrag();}}><img src={base+frame} alt="澜鲸角色" draggable={false} onError={()=>setError(true)}/></button>{error&&<p role="alert">角色资源加载失败</p>}<div className="character-actions">{[['idle','待机'],['wave','挥手'],['think','思考'],['confused','疑惑']].map(([id,label])=><button key={id} onClick={()=>setAction(id)}>{label}</button>)}</div></div>;
}
