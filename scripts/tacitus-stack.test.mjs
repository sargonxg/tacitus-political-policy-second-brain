import test from "node:test";
import assert from "node:assert/strict";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  createConfig,
  composeSpec,
  initConfig,
  childEnvironment,
  commandArgs,
} from "./tacitus-stack.mjs";

test("self-hosted local and cloud configurations use source builds and isolated runtime", () => {
  for (const mode of ["local", "cloud"]) {
    const config = createConfig(mode);
    const spec = composeSpec(config);
    for (const name of ["api", "frontend", "postgres"])
      assert.ok(spec.services[name].build);
    for (const name of ["api", "postgres", "ollama", "migrate"])
      assert.deepEqual(spec.services[name].networks, ["private"]);
    assert.equal(spec.networks.private.internal, true);
    assert.equal(spec.services.ollama.environment.OLLAMA_NO_CLOUD, "1");
    assert.equal(spec.services.api.environment.EDGEQUAKE_AUTH_ENABLED, "true");
    assert.equal(spec.services.api.environment.TACITUS_LOCAL_ONLY, "1");
    assert.equal(
      spec.services.api.environment.EDGEQUAKE_EMBEDDING_DIMENSION,
      "768",
    );
    assert.equal(
      spec.services.api.environment.EDGEQUAKE_VISION_PROVIDER,
      "none",
    );
    assert.equal(
      spec.services.api.environment.EDGEQUAKE_ALLOW_ANONYMOUS,
      "false",
    );
    assert.equal(spec.services.api.environment.EDGEQUAKE_DEV_MODE, "false");
    assert.equal(
      spec.services.api.environment.EDGEQUAKE_ALLOW_MOCK_PROVIDER,
      "0",
    );
    assert.equal(
      spec.services.api.environment.OLLAMA_HOST,
      "http://ollama:11434",
    );
    for (const service of Object.values(spec.services)) {
      assert.equal(service.env_file, undefined);
      assert.equal(service.network_mode, undefined);
      assert.equal(service.privileged, undefined);
      for (const key of Object.keys(service.environment ?? {}))
        assert.ok(!/OPENAI|ANTHROPIC|GEMINI|MISTRAL|LANGFUSE/.test(key));
      for (const port of service.ports ?? [])
        assert.match(port, /^127\.0\.0\.1:/);
    }
    assert.equal(spec.services.postgres.ports, undefined);
    assert.equal(spec.services.ollama.ports, undefined);
    assert.ok(spec.volumes["pg-data"]);
    assert.ok(spec.volumes["model-data"]);
    assert.ok(spec.services.api.depends_on.migrate);
  }
});

test("ambient credentials, proxies and Docker remote contexts are not inherited", () => {
  const env = childEnvironment({
    PATH: "path",
    SystemRoot: "windows",
    OPENAI_API_KEY: "test-secret",
    HTTP_PROXY: "bad",
    DOCKER_HOST: "remote",
    COMPOSE_FILE: "other",
    FOUNDRY_TOKEN: "private",
  });
  assert.deepEqual(env, {
    PATH: "path",
    SystemRoot: "windows",
    COMPOSE_DISABLE_ENV_FILE: "1",
  });
});

test("initial model download is a separate networked one-shot using the runtime volume", () => {
  const spec = composeSpec(createConfig("cloud"));
  const download = spec.services["model-download"];
  assert.deepEqual(download.profiles, ["download"]);
  assert.deepEqual(download.networks, ["download"]);
  assert.deepEqual(download.volumes, spec.services.ollama.volumes);
  assert.ok(!spec.services.api.depends_on["model-download"]);
  assert.equal(spec.networks.download.internal, undefined);
});

test("Compose does not expand the downloader shell variables on the host", () => {
  const command = composeSpec(createConfig("cloud")).services["model-download"]
    .command[0];
  assert.ok(
    !command.replaceAll("$$", "").includes("$"),
    "shell variables must use Compose dollar escaping",
  );
  const shell = command.replaceAll("$$", "$");
  assert.ok(shell.includes("pid=$!"));
  assert.ok(shell.includes("kill $pid"));
  assert.ok(shell.includes('ollama pull "$1"; ollama pull "$2"'));
});

test("configuration rejects arbitrary image endpoints, shell model names and weak secrets", () => {
  for (const change of [
    { ollamaImage: "evil.example/image:latest" },
    { ollamaImage: "ollama/ollama:latest" },
    { chatModel: "x;curl evil" },
    { embeddingModel: "x-cloud" },
    { mode: "public" },
    { postgresPassword: "weak" },
    { apiPort: 11434 },
    { frontendPort: 8080 },
  ])
    assert.throws(() => composeSpec({ ...createConfig("local"), ...change }));
});

test("init never replaces saved credentials or configuration", async () => {
  const dir = await mkdtemp(join(tmpdir(), "tacitus-stack-"));
  try {
    const path = join(dir, "config.json");
    await initConfig(path, "local");
    const original = await readFile(path, "utf8");
    await assert.rejects(initConfig(path, "cloud"), /EEXIST/);
    assert.equal(await readFile(path, "utf8"), original);
    assert.notEqual(
      createConfig("local").jwtSecret,
      JSON.parse(original).jwtSecret,
    );
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test("commands cannot delete volumes, print secrets, auto-download or publish", () => {
  assert.deepEqual(commandArgs("down"), ["down"]);
  assert.deepEqual(commandArgs("up"), [
    "up",
    "-d",
    "--no-build",
    "--pull",
    "never",
  ]);
  assert.deepEqual(commandArgs("build"), [
    "build",
    "postgres",
    "api",
    "frontend",
  ]);
  assert.throws(() => commandArgs("push"));
  assert.throws(() => commandArgs("config"));
});
