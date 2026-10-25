// Executed unchanged beta TSX with a labeled recording React/DOM/IPC harness.
// This checks real parser, reconciliation and callback code, not desktop rendering.
const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { load, ts } = require('./ts-loader.cjs');
const base = path.resolve(__dirname,'../fixtures/beta/src/renderer/src/raycast-api');
const plain = v => JSON.parse(JSON.stringify(v));
let current;
function frame() {
  const f={slots:[],index:0,effects:[]};
  f.render=(fn,props)=>{current=f;f.index=0;f.effects=[];return fn(props);};
  return f;
}
function slot(initial) {const i=current.index++;if (!(i in current.slots)) current.slots[i]=typeof initial==='function'?initial():initial;return i;}
const react={
  useState(initial){const f=current,i=slot(initial);return [f.slots[i],v=>{f.slots[i]=typeof v==='function'?v(f.slots[i]):v;}];},
  useRef(initial){const i=slot({current:initial});return current.slots[i];},
  useMemo(fn){return fn();},useCallback(fn){return fn;},
  useEffect(fn){current.effects.push(fn);},
  useImperativeHandle(ref,fn){if(ref)ref.current=fn();},
  forwardRef(fn){return props=>fn(props,props.ref);},
  createContext(value){return {Provider:'recording-provider',value};},
  useContext(ctx){return ctx.value;},
};
const jsx=(type,props)=>({type,props:props??{}});
const jsxRuntime={jsx,jsxs:jsx,Fragment:'fragment'};
const effects=[];
const listeners=new Map();
const window={
  addEventListener(type,fn){if(!listeners.has(type))listeners.set(type,[]);listeners.get(type).push(fn);},
  removeEventListener(type,fn){listeners.set(type,(listeners.get(type)??[]).filter(v=>v!==fn));},
  dispatchEvent(event){effects.push(['dispatch',event.type]);},
  tezbar:new Proxy({}, {get:(_,name)=>(...args)=>{effects.push([name,...args]);return Promise.resolve({ok:true});}}),
};
const primitive=()=>null;
const primitives={Hint:primitive,HintBar:primitive,Kbd:primitive,ViewHeader:primitive,cx:(...a)=>a.join(' ')};
const cache={};
function module(name) {
  if(cache[name])return cache[name];
  const imports={react,'react/jsx-runtime':jsxRuntime,'../../ui/primitives':primitives,'../../ui/Markdown':{Markdown:primitive},'../../colorConverter':{buildColorFormatRows:()=>[],buildColorConversionResults:()=>[]}};
  const deps={
    index:['action-runtime','action-runtime-registry','detail-runtime','form-runtime','grid-runtime','list-runtime','hooks'],
    'action-runtime':['action-runtime-overlay'],
    'form-runtime':['detail-runtime'],
    'list-runtime':['detail-runtime'],
    'grid-runtime':['grid-content'],hooks:['ipc-bridge'],
  };
  for(const dep of deps[name]??[])imports['./'+dep]=module(dep);
  const extensions={ 'form-runtime':'\nexport { collectFormItems };', 'list-runtime':'\nexport { parseListRows };', 'detail-runtime':'\nexport { markdownFromNode };' };
  const file=fs.existsSync(path.join(base,name+'.tsx'))?name+'.tsx':name+'.ts';
  return cache[name]=load(path.join(base,file),imports,extensions[name]??'',{window,CustomEvent:class{constructor(type){this.type=type;}}});
}
function nodes(tree) {
  if(!tree||typeof tree!=='object')return [];
  return [tree,...[tree.props?.children].flat(Infinity).flatMap(nodes)];
}
const view=JSON.parse(fs.readFileSync(path.resolve(__dirname,'../reports/render.json'),'utf8'));
const detail=module('detail-runtime'),surface=module('index'),form=module('form-runtime'),list=module('list-runtime'),bridge=module('ipc-bridge');
function surfaceProps(extra={}) {
  return {...view,onBack:()=>effects.push(['back']),onSearchTextChanged:()=>{},onLoadMore:()=>{},onInvokeAction:(actionId,formValues)=>bridge.invokeExtensionAction({sessionId:view.sessionId,actionId,...(formValues?{formValues}:{})}),...extra};
}
test('actual Detail reads adapter markdown and exposes callbacks without URL link bypass',()=>{
  assert.match(detail.markdownFromNode(view.root),/Link copied to clipboard/);
  const calls=[];const tree=frame().render(detail.DetailRuntime,{root:view.root,title:view.title,onBack:()=>calls.push('back'),onRunPrimaryAction:()=>calls.push('primary'),onOpenActions:()=>calls.push('actions')});
  const buttons=nodes(tree).filter(n=>n.type==='button');buttons.forEach(n=>n.props.onClick());
  assert.deepEqual(calls,['back','primary','actions']);assert.equal(nodes(tree).filter(n=>n.type==='a').length,0);
  const markdown=nodes(tree).find(n=>n.type===primitive&&n.props.text);assert.equal(markdown.props.text,view.root.props.markdown);
});
test('actual Surface primary and Back route through real callback/IPC code',async()=>{
  effects.length=0;const tree=frame().render(surface.ExtensionRuntimeSurface,surfaceProps());
  const node=nodes(tree).find(n=>n.type===detail.DetailRuntime);assert.ok(node);
  node.props.onRunPrimaryAction();node.props.onBack();
  assert.deepEqual(plain(effects),[['extensionInvokeAction',{sessionId:view.sessionId,actionId:view.actions[0].id}],['back']]);
  assert.equal(surface.backActionId(view.root),null);
});
test('actual Surface Detail keyboard Enter invokes first action; extension advertises no shortcuts',()=>{
  effects.length=0;listeners.clear();const f=frame();f.render(surface.ExtensionRuntimeSurface,surfaceProps());
  f.effects.forEach(fn=>fn());
  for(const fn of listeners.get('keydown')??[])fn({key:'Enter',preventDefault(){}});
  assert.equal(effects.filter(v=>v[0]==='extensionInvokeAction').length,1);
  assert.equal(effects.find(v=>v[0]==='extensionInvokeAction')[1].actionId,view.actions[0].id);
  assert.ok(view.actions.every(a=>!a.shortcut));
});
test('actual action overlay callbacks route selected adapter capability',()=>{
  effects.length=0;const f=frame();let tree=f.render(surface.ExtensionRuntimeSurface,surfaceProps());
  nodes(tree).find(n=>n.type===detail.DetailRuntime).props.onOpenActions();
  tree=f.render(surface.ExtensionRuntimeSurface,surfaceProps());
  const overlay=nodes(tree).find(n=>n.type===module('action-runtime-overlay').ActionPanelOverlay);assert.ok(overlay);
  overlay.props.onExecute(view.actions[1]);assert.equal(effects[0][1].actionId,view.actions[1].id);
});
test('actual Form parser sees no invented source fields; List parser sees no invented rows',()=>{
  assert.deepEqual(plain(form.collectFormItems(view.root)),[]);
  assert.deepEqual(plain(list.parseListRows(view.root)),[]);
  const rows=list.parseListRows({type:'List',children:[{type:'List.Item',props:{id:'row',title:'Recorded',actionIds:[view.actions[0].id]}}]});
  assert.equal(rows[0].id,'row');assert.equal(rows[0].actionIds[0],view.actions[0].id);
});
test('actual generic Form reconciliation preserves omitted/uncontrolled newer drafts',()=>{
  const items=form.collectFormItems({type:'Form',children:[{type:'Form.TextField',props:{id:'first',value:'',actionId:'change'}},{type:'Form.TextField',props:{id:'second',value:''}}]});
  const fields=items.map(i=>i.field),next=structuredClone(fields);delete next[0].value;
  assert.deepEqual(plain(form.reconcileFormValues(next,{first:'queued new',second:'local draft'},fields)),{first:'queued new',second:'local draft'});
  const controlled=structuredClone(fields);controlled[0].value='ack';
  assert.deepEqual(plain(form.reconcileFormValues(controlled,{first:'queued new',second:'local draft'},fields)),{first:'ack',second:'local draft'});
});
test('actual generic Form callback uses value and submit sends complete map',()=>{
  effects.length=0;const root={type:'Form',children:[{type:'Form.TextField',props:{id:'first',value:'a',actionId:'change'}},{type:'Form.TextField',props:{id:'second',value:'b'}}]};
  const f=frame();const props={root,title:'Fixture',onBack:()=>{},onSubmitForm:values=>effects.push(['submit',plain(values)]),onChangeField:(id,value)=>effects.push(['change',id,value]),onInvokeAction:()=>{},onOpenActions:()=>{}};
  let tree=f.render(form.FormRuntime,props);
  const input=nodes(tree).find(n=>n.type==='input'&&n.props.value==='a');assert.ok(input);input.props.onChange({target:{value:'new'}});
  tree=f.render(form.FormRuntime,props);tree.props.onSubmit({preventDefault(){}});
  assert.deepEqual(effects,[['change','change','new'],['submit',{first:'new',second:'b'}]]);
  const sf=frame();const st=sf.render(surface.ExtensionRuntimeSurface,surfaceProps({root,actions:[{id:'submit',kind:'submit-form'}]}));
  nodes(st).find(n=>n.type===form.FormRuntime).props.onChangeField('change','queued');
  assert.deepEqual(plain(effects.at(-1)),['extensionInvokeAction',{sessionId:view.sessionId,actionId:'change',formValues:{value:'queued'}}]);
});
test('actual beta launch bridge preserves argumentValues; no OS service called',async()=>{
  effects.length=0;const payload={extensionId:view.extensionId,commandName:view.commandName,argumentValues:{word:'APPLE'}};
  await bridge.runExtensionCommand(payload);assert.deepEqual(effects,[['extensionRunCommand',payload]]);
});
test('actual descriptor typechecks against pinned beta types and rejects unsupported action kind',()=>{
  const root=path.resolve(__dirname,'..');
  const report=path.join(root,'reports/descriptor-contract.ts');
  const library=path.join(root,'fixtures/libraries/typescript/lib/lib.es5.d.ts');
  const source=`import type { ExtensionRunCommandResult, ExtensionRuntimeAction } from '../fixtures/beta/src/shared/extensionRuntime';\nconst view: ExtensionRunCommandResult = ${JSON.stringify(view)};\n`;
  function diagnostics(text) {
    fs.writeFileSync(report,text);
    const program=ts.createProgram([report,library],{strict:true,noEmit:true,target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS,noLib:true});
    return ts.getPreEmitDiagnostics(program);
  }
  assert.deepEqual(diagnostics(source).map(d=>ts.flattenDiagnosticMessageText(d.messageText,'\n')),[]);
  const rejected=diagnostics(source+'const bad: ExtensionRuntimeAction = {id:"x",title:"x",kind:"callback"};');
  assert.equal(rejected.length,1);assert.equal(rejected[0].code,2322);
  fs.writeFileSync(report,source);
});
