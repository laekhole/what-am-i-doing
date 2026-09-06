// Windows regression: sleeping stubs only. Build tests/fixtures/process_stub.rs as
// .tools/agent-detection-check/helper.exe, then run this from the repository root.

const fs=require('fs'),path=require('path'),cp=require('child_process'),assert=require('assert');
const root=path.resolve('.tools/agent-detection-check'),home=path.join(root,'isolated-home');fs.mkdirSync(home,{recursive:true});
const env={...process.env,HOME:home,USERPROFILE:home,WAID_ADAPTERS:path.join(root,'no-adapters'),PYTHONPATH:root};delete env.CODEX_HOME;delete env.COPILOT_HOME;
const core=path.resolve('target/x86_64-pc-windows-gnu/release/waid.exe');
const children=[],cases=[];
async function launch(exe,args,agent) {
 const child=cp.spawn(exe,args,{cwd:root,env,windowsHide:true,stdio:['ignore','pipe','pipe']});children.push(child);
 const pid=await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('fixture timeout '+agent)),10000);child.stdout.once('data',b=>{clearTimeout(timer);resolve(Number(b.toString().trim()))});child.once('error',reject);child.once('exit',code=>{if(code)reject(Error('fixture exit '+code))});});
 cases.push({pid,agent});
}
(async()=>{try {
 for (const [pkg,agent] of [['@google/gemini-cli','gemini'],['@github/copilot','copilot'],['opencode-ai','opencode']]) {
  const script=path.join(root,'node_modules',pkg,'index.js');fs.mkdirSync(path.dirname(script),{recursive:true});fs.writeFileSync(script,'console.log(process.pid);setInterval(()=>{},1000);');await launch(process.execPath,['--no-warnings',script],agent);
 }
 fs.mkdirSync(path.join(root,'aider'),{recursive:true});fs.writeFileSync(path.join(root,'aider/__init__.py'),'# Local fixture package; shadows any installed aider.\n');fs.writeFileSync(path.join(root,'aider/__main__.py'),'import os,time\nprint(os.getpid(),flush=True)\ntime.sleep(120)\n');await launch('python.exe',['-m','aider'],'aider');
 for(const agent of ['goose','cursor','opencode']) { const exe=path.join(root,agent==='cursor'?'cursor-agent.exe':agent+'.exe');fs.copyFileSync(path.join(root,'helper.exe'),exe);await launch(exe,[],agent); }
 const negative=path.join(root,'server.js');fs.writeFileSync(negative,'console.log(process.pid);setInterval(()=>{},1000);');await launch(process.execPath,[negative,'@github/copilot'],null);
 const start=Date.now();let final;
 for(let i=0;i<30;i++) {
   const result=cp.spawnSync(core,['--json'],{env,windowsHide:true,encoding:'utf8'});assert.equal(result.status,0,result.stderr);
   const data=JSON.parse(result.stdout);const rows=data.sessions;
   for(const fixture of cases) {
      const row=rows.find(r=>r.pid===fixture.pid);
      if(fixture.agent===null) assert.equal(row,undefined,'incidental mention');
      else {assert(row,'missing '+fixture.agent);assert.equal(row.agent.name,fixture.agent);assert.equal(row.status.state,'unknown');}
   }
   final=rows.filter(r=>cases.some(f=>f.pid===r.pid)).map(r=>({pid:r.pid,agent:r.agent.name,state:r.status.state}));
 }
 console.log(JSON.stringify({fixtures:cases.length,scans:30,totalMs:Date.now()-start,verified:final},null,2));
}finally{ for(const child of children){try{child.kill();}catch{}} }})().catch(e=>{console.error(e);process.exitCode=1;});
