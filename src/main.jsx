import Desktop from './Desktop.jsx';
import React from 'react';
import {createRoot} from 'react-dom/client';
import App from './App.jsx';
import './styles.less';
const floating=new URLSearchParams(location.search).get('window')==='pet';
if(floating)document.body.classList.add('floating');
createRoot (document.getElementById ('root')).render (
  <React.StrictMode>{floating?<Desktop floating/>:<App/>}</React.StrictMode>
);

