// Source-built self-hosting candidate. Runtime acceptance is separate from configuration tests.
import {randomBytes} from 'node:crypto';
import {mkdir,readFile,writeFile,statfs} from 'node:fs/promises';
import {dirname,resolve,join} from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {totalmem,freemem,platform} from 'node:os';

const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const configPath=join(root,'.tacitus','stack','config.json');
const secret=()=>randomBytes(32).toString('hex');
export function createConfig(mode) {
  return {version:1,mode,project:'tacitus-policy',apiPort:8080,frontendPort:3000,
    ollamaImage:'ollama/ollama:0.40.3',chatModel:'qwen3:4b',embeddingModel:'embeddinggemma:latest',
    postgresPassword:secret(),jwtSecret:secret(),adminPassword:secret(),secretsKey:secret()};
}
function validate(c) {
  const fields=Object.keys(createConfig('local'));
  if(!c || Object.keys(c).some(k=>!fields.includes(k)) || c.version!==1 || !['local','cloud'].includes(c.mode)) throw new Error('Unsupported stack configuration');
  if(!/^[a-z][a-z0-9-]{2,40}$/.test(c.project)) throw new Error('Invalid project name');
  if(!/^ollama\/ollama:\d+\.\d+\.\d+$/.test(c.ollamaImage)) throw new Error('Use an explicit official Ollama version');
  for(const model of [c.chatModel,c.embeddingModel]) if(typeof model!=='string'||!/^[-a-zA-Z0-9_.:]{1,120}$/.test(model)||/cloud/i.test(model)) throw new Error('Local model tag required');
  for(const port of [c.apiPort,c.frontendPort]) if(!Number.isInteger(port)||port<1024||port>65535||[5432,11434].includes(port)) throw new Error('Invalid published port');
  if(c.apiPort===c.frontendPort) throw new Error('Ports must differ');
  for(const key of ['postgresPassword','jwtSecret','adminPassword','secretsKey']) if(!/^[a-f0-9]{64}$/.test(c[key])) throw new Error(`Generated 256-bit ${key} required`);
}
export function composeSpec(c) {
  validate(c);
  const apiEnv={DATABASE_URL:`postgres://edgequake:${c.postgresPassword}@postgres:5432/edgequake`,
    EDGEQUAKE_HOST:'0.0.0.0',EDGEQUAKE_PORT:'8080',EDGEQUAKE_SCHEMA_GATE:'fail',
    EDGEQUAKE_LLM_PROVIDER:'ollama',EDGEQUAKE_LLM_MODEL:c.chatModel,
    EDGEQUAKE_EMBEDDING_PROVIDER:'ollama',EDGEQUAKE_EMBEDDING_MODEL:c.embeddingModel,
    OLLAMA_HOST:'http://ollama:11434',OLLAMA_MODEL:c.chatModel,OLLAMA_EMBEDDING_MODEL:c.embeddingModel,
    OLLAMA_NO_CLOUD:'1',OLLAMA_CONTEXT_LENGTH:'8192',
    EDGEQUAKE_VISION_PROVIDER:'ollama',EDGEQUAKE_VISION_MODEL:c.chatModel,
    EDGEQUAKE_DECISION_ENABLED:'0',EDGEQUAKE_ALLOW_MOCK_PROVIDER:'0',
    EDGEQUAKE_AUTH_ENABLED:'true',EDGEQUAKE_ALLOW_ANONYMOUS:'false',ALLOW_REGISTRATION:'false',
    EDGEQUAKE_DEV_MODE:'false',EDGEQUAKE_STRICT_TENANT_BIND:'true',EDGEQUAKE_PG_RLS_ENABLED:'true',
    EDGEQUAKE_BOOTSTRAP_ADMIN_USERNAME:'admin',EDGEQUAKE_BOOTSTRAP_ADMIN_EMAIL:'admin@localhost',
    EDGEQUAKE_BOOTSTRAP_ADMIN_PASSWORD:c.adminPassword,JWT_SECRET:c.jwtSecret,EDGEQUAKE_SECRETS_KEY:c.secretsKey,
    EDGEQUAKE_CORS_ORIGINS:`http://localhost:${c.frontendPort},http://127.0.0.1:${c.frontendPort}`,
    EDGEQUAKE_REQUIRE_DELETE_ALL_CONFIRM:'true',RUST_LOG:'info'};
  const apiBuild={context:root,dockerfile:'edgequake/docker/Dockerfile',args:{ENABLE_OTEL:'false'}};
  const runtime={restart:'unless-stopped',networks:['private']};
  return {name:c.project,services:{
    postgres:{...runtime,image:`${c.project}-postgres:checkout`,
      build:{context:root,dockerfile:'edgequake/docker/Dockerfile.postgres.unified',args:{PG_MAJOR:'16',PGVECTOR_VERSION:'v0.8.5',AGE_GIT_REF:'PG16/v1.6.0-rc0',AGE_EXPECTED_VERSION:'1.6.0'}},
      environment:{POSTGRES_USER:'edgequake',POSTGRES_DB:'edgequake',POSTGRES_PASSWORD:c.postgresPassword},
      command:['postgres','-c','shared_preload_libraries=age'],volumes:['pg-data:/var/lib/postgresql/data'],shm_size:'256m',
      healthcheck:{test:['CMD-SHELL','pg_isready -U edgequake -d edgequake'],interval:'10s',timeout:'5s',retries:12}},
    ollama:{...runtime,image:c.ollamaImage,environment:{OLLAMA_HOST:'0.0.0.0:11434',OLLAMA_NO_CLOUD:'1',OLLAMA_CONTEXT_LENGTH:'8192',OLLAMA_NUM_PARALLEL:'1',OLLAMA_MAX_LOADED_MODELS:'1'},
      volumes:['model-data:/root/.ollama'],healthcheck:{test:['CMD','ollama','list'],interval:'10s',timeout:'5s',retries:12}},
    'model-download':{image:c.ollamaImage,profiles:['download'],networks:['download'],restart:'no',volumes:['model-data:/root/.ollama'],
      environment:{OLLAMA_NO_CLOUD:'1'},entrypoint:['/bin/sh','-ec'],
      command:['ollama serve & pid=$!; trap "kill $pid" EXIT; i=0; until ollama list >/dev/null 2>&1; do i=$((i+1)); [ "$i" -lt 60 ] || exit 1; sleep 1; done; ollama pull "$1"; ollama pull "$2"','download',c.chatModel,c.embeddingModel]},
    migrate:{image:`${c.project}-api:checkout`,build:apiBuild,networks:['private'],restart:'no',environment:{DATABASE_URL:apiEnv.DATABASE_URL,EDGEQUAKE_MIGRATE_CLI:'1'},command:['migrate'],depends_on:{postgres:{condition:'service_healthy'}}},
    api:{...runtime,image:`${c.project}-api:checkout`,build:apiBuild,environment:apiEnv,
      ports:[`127.0.0.1:${c.apiPort}:8080`],depends_on:{postgres:{condition:'service_healthy'},ollama:{condition:'service_healthy'},migrate:{condition:'service_completed_successfully'}},
      healthcheck:{test:['CMD','edgequake','healthcheck'],interval:'20s',timeout:'10s',retries:12,start_period:'30s'}},
    frontend:{restart:'unless-stopped',networks:['web'],image:`${c.project}-frontend:checkout`,
      build:{context:root,dockerfile:'edgequake_webui/Dockerfile',args:{NEXT_PUBLIC_API_URL:`http://localhost:${c.apiPort}`}},
      ports:[`127.0.0.1:${c.frontendPort}:3000`],environment:{NODE_ENV:'production',NEXT_TELEMETRY_DISABLED:'1',EDGEQUAKE_API_URL:`http://localhost:${c.apiPort}`,NEXT_PUBLIC_AUTH_ENABLED:'true',NEXT_PUBLIC_DISABLE_DEMO_LOGIN:'true'},
      depends_on:{api:{condition:'service_healthy'}}},
  },volumes:{'pg-data':{},'model-data':{}},networks:{private:{internal:true},web:{},download:{}}};
}
export function childEnvironment(ambient=process.env) {
  const env={};
  for(const key of ['PATH','Path','SystemRoot','SYSTEMROOT','WINDIR','TEMP','TMP','HOME','USERPROFILE','DOCKER_CONFIG','XDG_RUNTIME_DIR']) if(ambient[key]) env[key]=ambient[key];
  env.COMPOSE_DISABLE_ENV_FILE='1'; return env;
}
export async function initConfig(path,mode) {
  const c=createConfig(mode); validate(c);
  await mkdir(dirname(path),{recursive:true,mode:0o700});
  await writeFile(path,JSON.stringify(c,null,2)+'\n',{flag:'wx',mode:0o600});
}
export function commandArgs(command) {
  const commands={build:['build','postgres','api','frontend'],download:['--profile','download','run','--rm','model-download'],
    up:['up','-d','--no-build','--pull','never'],down:['down'],status:['ps'],restart:['restart','postgres','ollama','api','frontend']};
  if(!commands[command]) throw new Error('Unsupported command'); return commands[command];
}
function docker(args,stdio='inherit') {
  const r=spawnSync('docker',args,{cwd:root,env:childEnvironment(),stdio,encoding:'utf8'});
  if(r.error) throw new Error('Docker is unavailable; install on a suitable host. No services started.');
  if(r.status!==0) throw new Error(`Docker command failed (exit ${r.status})`); return r;
}
async function main(args) {
  const command=args[0]??'doctor';
  if(command==='init') {
    if(args.length>2) throw new Error('Usage: init local|cloud');
    await initConfig(configPath,args[1]??'local');
    console.log('Saved private .tacitus/stack/config.json. It contains the admin password; do not share or commit it. No software installed.'); return;
  }
  if(command==='doctor') {
    const disk=await statfs(root); const hasDocker=spawnSync('docker',['compose','version'],{env:childEnvironment(),stdio:'ignore'});
    const report={platform:platform(),ramGiB:+(totalmem()/2**30).toFixed(1),freeRamGiB:+(freemem()/2**30).toFixed(1),freeDiskGiB:+(Number(disk.bavail)*Number(disk.bsize)/2**30).toFixed(1),dockerCompose:!hasDocker.error&&hasDocker.status===0,
      recommendation:'Use a separate local host or cloud VM with 32 GiB RAM and 100 GiB free disk for source builds; GPU optional.',runtimeAcceptance:'NOT RUN'};
    console.log(JSON.stringify(report,null,2)); if(!report.dockerCompose||report.freeDiskGiB<40||report.freeRamGiB<8) process.exitCode=2; return;
  }
  const c=JSON.parse(await readFile(configPath,'utf8')); const spec=composeSpec(c);
  const path=join(dirname(configPath),'compose.json'); await writeFile(path,JSON.stringify(spec,null,2)+'\n',{mode:0o600});
  const base=['--context','default','compose','--env-file',join(dirname(configPath),'empty.env'),'-p',c.project,'-f',path];
  await writeFile(join(dirname(configPath),'empty.env'),'',{mode:0o600});
  if(command==='check') {docker([...base,'config','--quiet']);console.log('Compose configuration valid. Build/inference/persistence remain unverified.');return;}
  const operations=commandArgs(command);
  if(command==='build'||command==='download') console.log('This explicit command downloads software or models on this host.');
  docker([...base,...operations]);
  if(command==='up') console.log(`Candidate startup requested. Verify http://localhost:${c.frontendPort} and http://localhost:${c.apiPort}/health. This is not policy-workflow acceptance.`);
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)) main(process.argv.slice(2)).catch(e=>{console.error(e.message);process.exitCode=2;});
