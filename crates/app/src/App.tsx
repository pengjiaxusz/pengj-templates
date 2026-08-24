import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { ScrollArea } from "@/components/ui/scroll-area";

interface LayerInfo {
  id: string;
  name: string;
  description: string;
  depends: string[];
  file_count: number;
}

interface SkillInfo {
  name: string;
  description: string;
}

interface GenerateReport {
  project_dir: string;
  layers: string[];
  files: string[];
}

interface ConflictInfo {
  path: string;
  reason: string;
}

interface UpdateReport {
  project_name: string;
  layers: string[];
  updated: string[];
  created: string[];
  conflicted: ConflictInfo[];
  removed: string[];
  unchanged: number;
}

function App() {
  return (
    <div className="min-h-screen flex flex-col">
      <header className="border-b px-6 py-4">
        <h1 className="text-lg font-semibold">pengj-templates</h1>
        <p className="text-sm text-muted-foreground">
          分层模板生成与同步更新工具
        </p>
      </header>

      <main className="flex-1 p-6">
        <Tabs defaultValue="generate" className="w-full">
          <TabsList>
            <TabsTrigger value="generate">生成项目</TabsTrigger>
            <TabsTrigger value="update">同步更新</TabsTrigger>
          </TabsList>
          <TabsContent value="generate">
            <GenerateTab />
          </TabsContent>
          <TabsContent value="update">
            <UpdateTab />
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}

// ---------- 生成 ----------

function GenerateTab() {
  const [layers, setLayers] = useState<LayerInfo[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [name, setName] = useState("");
  const [parentDir, setParentDir] = useState("");
  const [edition, setEdition] = useState("2021");
  const [channel, setChannel] = useState("stable");
  const [useSccache, setUseSccache] = useState(true);
  const [useLld, setUseLld] = useState(true);
  const [chinese, setChinese] = useState(false);
  const [skillLang, setSkillLang] = useState("zh");
  const [commitZh, setCommitZh] = useState(true);
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [selectedSkills, setSelectedSkills] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<GenerateReport | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    invoke<LayerInfo[]>("cmd_list_layers")
      .then(setLayers)
      .catch((e) => setError(String(e)));
    invoke<SkillInfo[]>("cmd_list_skills")
      .then((list) => {
        setSkills(list);
        setSelectedSkills(new Set(list.map((s) => s.name)));
      })
      .catch((e) => setError(String(e)));
  }, []);

  const toggle = useCallback((id: string) => {
    setSelected((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }, []);

  const toggleSkill = useCallback((name: string) => {
    setSelectedSkills((prev) => {
      const next = new Set(prev);
      if (next.has(name)) next.delete(name);
      else next.add(name);
      return next;
    });
  }, []);

  const pickDir = async () => {
    const dir = await open({ directory: true, title: "选择输出目录" });
    if (typeof dir === "string") setParentDir(dir);
  };

  const generate = async () => {
    setBusy(true);
    setError("");
    setReport(null);
    try {
      const result = await invoke<GenerateReport>("cmd_create_project", {
        name,
        layers: [...selected],
        parentDir,
        options: {
          edition,
          channel,
          use_sccache: useSccache,
          use_lld: useLld,
          chinese_programming: chinese,
          skill_lang: skillLang,
          commit_zh: commitZh,
          skills: [...selectedSkills],
        },
      });
      setReport(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const canGenerate =
    name.trim() !== "" && parentDir !== "" && selected.size > 0;

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>配置</CardTitle>
          <CardDescription>选择层并指定输出位置</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <Label htmlFor="project-name">项目名</Label>
            <Input
              id="project-name"
              value={name}
              placeholder="例如 my-app"
              onChange={(e) => setName(e.target.value)}
            />
          </div>

          <div className="space-y-2">
            <Label>选择层</Label>
            <div className="space-y-2">
              {layers.map((layer) => (
                <div
                  key={layer.id}
                  className="flex items-start gap-3 rounded-md border p-3"
                >
                  <Checkbox
                    id={`layer-${layer.id}`}
                    checked={selected.has(layer.id)}
                    onCheckedChange={() => toggle(layer.id)}
                  />
                  <label
                    htmlFor={`layer-${layer.id}`}
                    className="flex-1 cursor-pointer space-y-1"
                  >
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{layer.name}</span>
                      <Badge variant="secondary">{layer.id}</Badge>
                      <Badge variant="outline">{layer.file_count} 个文件</Badge>
                    </div>
                    <p className="text-sm text-muted-foreground">
                      {layer.description}
                    </p>
                    {layer.depends.length > 0 && (
                      <p className="text-xs text-muted-foreground">
                        自动包含: {layer.depends.join(", ")}
                      </p>
                    )}
                  </label>
                </div>
              ))}
              {layers.length === 0 && (
                <p className="text-sm text-muted-foreground">加载层列表失败或没有可用层</p>
              )}
            </div>
          </div>

          <div className="space-y-2">
            <Label>输出目录</Label>
            <div className="flex gap-2">
              <Input
                value={parentDir}
                readOnly
                placeholder="点击右侧选择目录"
              />
              <Button variant="outline" onClick={pickDir}>
                选择
              </Button>
            </div>
          </div>

          <div className="space-y-2">
            <Label>Rust 选项（选 rust 层时生效）</Label>
            <div className="grid grid-cols-2 gap-2">
              <select
                value={edition}
                onChange={(e) => setEdition(e.target.value)}
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
              >
                {["2015", "2018", "2021", "2024"].map((v) => (
                  <option key={v} value={v}>
                    {v}
                  </option>
                ))}
              </select>
              <select
                value={channel}
                onChange={(e) => setChannel(e.target.value)}
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
              >
                {["stable", "beta", "nightly"].map((v) => (
                  <option key={v} value={v}>
                    {v}
                  </option>
                ))}
              </select>
              <div className="flex items-center gap-4">
                <label className="flex items-center gap-2">
                  <Checkbox
                    checked={useSccache}
                    onCheckedChange={(v) => setUseSccache(!!v)}
                  />
                  sccache
                </label>
                <label className="flex items-center gap-2">
                  <Checkbox
                    checked={useLld}
                    onCheckedChange={(v) => setUseLld(!!v)}
                  />
                  lld
                </label>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <Checkbox
              checked={chinese}
              onCheckedChange={(v) => setChinese(!!v)}
              id="chinese"
            />
            <Label htmlFor="chinese">中文编程</Label>
          </div>

          <div className="space-y-2">
            <Label>Agent 技能（选 agent 层时生效）</Label>
            {skills.length > 0 && (
              <div className="space-y-2">
                {skills.map((skill) => (
                  <div
                    key={skill.name}
                    className="flex items-start gap-3 rounded-md border p-3"
                  >
                    <Checkbox
                      id={`skill-${skill.name}`}
                      checked={selectedSkills.has(skill.name)}
                      onCheckedChange={() => toggleSkill(skill.name)}
                    />
                    <label
                      htmlFor={`skill-${skill.name}`}
                      className="flex-1 cursor-pointer space-y-1"
                    >
                      <span className="font-medium">{skill.name}</span>
                      <p className="text-sm text-muted-foreground">{skill.description}</p>
                    </label>
                  </div>
                ))}
              </div>
            )}
            <div className="flex items-center gap-4">
              <select
                value={skillLang}
                onChange={(e) => setSkillLang(e.target.value)}
                className="h-9 rounded-md border bg-transparent px-3 text-sm"
              >
                <option value="zh">技能用中文</option>
                <option value="en">技能用英文</option>
              </select>
              <label className="flex items-center gap-2">
                <Checkbox
                  checked={commitZh}
                  onCheckedChange={(v) => setCommitZh(!!v)}
                />
                提交信息用中文
              </label>
            </div>
          </div>
        </CardContent>
        <CardContent>
          <Button onClick={generate} disabled={!canGenerate || busy}>
            {busy ? "生成中…" : "生成项目"}
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>结果</CardTitle>
          <CardDescription>生成报告</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {error && (
            <p className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
              {error}
            </p>
          )}
          {report && (
            <>
              <p className="text-sm">
                已生成到{" "}
                <code className="rounded bg-muted px-1.5 py-0.5 text-xs">
                  {report.project_dir}
                </code>
              </p>
              <p className="text-sm">
                层顺序:{" "}
                <span className="text-muted-foreground">
                  {report.layers.join(" → ")}
                </span>
              </p>
              <Separator />
              <p className="text-sm text-muted-foreground">
                共 {report.files.length} 个文件：
              </p>
              <ScrollArea className="h-72 rounded-md border">
                <ul className="p-3 space-y-1 font-mono text-xs">
                  {report.files.map((f) => (
                    <li key={f}>{f}</li>
                  ))}
                </ul>
              </ScrollArea>
            </>
          )}
          {!report && !error && (
            <p className="text-sm text-muted-foreground">等待生成…</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

// ---------- 更新 ----------

function UpdateTab() {
  const [projectDir, setProjectDir] = useState("");
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<UpdateReport | null>(null);
  const [error, setError] = useState("");
  const [workspaces, setWorkspaces] = useState<string[]>([]);
  const [selectedWorkspace, setSelectedWorkspace] = useState("");
  const [workspaceLoading, setWorkspaceLoading] = useState(false);

  // 探测项目根目录下的 *.code-workspace 供用户确认（后端 update 会全量同步所有 workspace 的 fileNesting）
  useEffect(() => {
    let cancelled = false;
    if (!projectDir) {
      setWorkspaces([]);
      setSelectedWorkspace("");
      setWorkspaceLoading(false);
      return;
    }
    setWorkspaceLoading(true);
    setWorkspaces([]);
    setSelectedWorkspace("");
    invoke<string[]>("cmd_list_workspaces", { projectDir })
      .then((list) => {
        if (cancelled) return;
        setWorkspaces(list);
        if (list.length > 0) setSelectedWorkspace(list[0]);
      })
      .catch(() => {
        // 命令不存在或目录不可读时静默清空，不阻断更新
        if (cancelled) return;
        setWorkspaces([]);
        setSelectedWorkspace("");
      })
      .finally(() => {
        if (!cancelled) setWorkspaceLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [projectDir]);

  const pickDir = async () => {
    const dir = await open({ directory: true, title: "选择项目目录" });
    if (typeof dir === "string") setProjectDir(dir);
  };

  const update = async () => {
    setBusy(true);
    setError("");
    setReport(null);
    try {
      const result = await invoke<UpdateReport>("cmd_update_project", {
        projectDir,
      });
      setReport(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="grid gap-6 lg:grid-cols-2">
      <Card>
        <CardHeader>
          <CardTitle>目标项目</CardTitle>
          <CardDescription>
            选择由 pengj-templates 生成的项目目录（含 .pengj.json）
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex gap-2">
            <Input value={projectDir} readOnly placeholder="点击右侧选择项目目录" />
            <Button variant="outline" onClick={pickDir}>
              选择
            </Button>
          </div>

          <div className="space-y-2">
            {workspaceLoading && (
              <p className="text-sm text-muted-foreground">探测中...</p>
            )}
            {!workspaceLoading && projectDir && workspaces.length > 0 && (
              <>
                <Label htmlFor="workspace-select">
                  检测到工作空间文件：
                </Label>
                <select
                  id="workspace-select"
                  value={selectedWorkspace}
                  onChange={(e) => setSelectedWorkspace(e.target.value)}
                  className="h-9 w-full rounded-md border bg-transparent px-3 text-sm"
                >
                  {workspaces.map((w) => (
                    <option key={w} value={w}>
                      {w}
                    </option>
                  ))}
                </select>
                <p className="text-xs text-muted-foreground">
                  更新时会同步所有工作空间的 fileNesting，默认高亮首项
                </p>
              </>
            )}
            {!workspaceLoading && projectDir && workspaces.length === 0 && (
              <p className="text-sm text-muted-foreground">
                未检测到 .code-workspace，将仅更新 .vscode/settings.json
              </p>
            )}
          </div>

          {selectedWorkspace && (
            <p className="flex items-center gap-2 text-xs text-muted-foreground">
              将同步
              <Badge variant="outline">{selectedWorkspace}</Badge>
              等 {workspaces.length} 个工作空间的 fileNesting
            </p>
          )}

          <Button onClick={update} disabled={!projectDir || busy}>
            {busy ? "更新中…" : "同步模板更新"}
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>更新结果</CardTitle>
          <CardDescription>模板变更同步报告</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {error && (
            <p className="rounded-md border border-destructive/50 bg-destructive/10 p-3 text-sm text-destructive">
              {error}
            </p>
          )}
          {report && (
            <>
              <p className="text-sm">
                项目{" "}
                <span className="font-medium">{report.project_name}</span>
                <span className="text-muted-foreground">
                  {" "}
                  （层: {report.layers.join(" → ")}）
                </span>
              </p>
              <p className="text-sm text-muted-foreground">
                更新 {report.updated.length} · 新增 {report.created.length} ·
                冲突 {report.conflicted.length} · 移除 {report.removed.length} ·
                未变 {report.unchanged}
              </p>
              {workspaces.length > 0 && (
                <p className="text-sm text-muted-foreground">
                  已同步 {workspaces.length} 个工作空间的 fileNesting（
                  {workspaces.join("、")}）
                </p>
              )}
              <Separator />
              <ScrollArea className="h-72 rounded-md border">
                <div className="p-3 space-y-2 text-xs">
                  {report.updated.map((f) => (
                    <p key={`u-${f}`}>
                      <Badge className="mr-2">更新</Badge>
                      <span className="font-mono">{f}</span>
                    </p>
                  ))}
                  {report.created.map((f) => (
                    <p key={`c-${f}`}>
                      <Badge variant="secondary" className="mr-2">
                        新增
                      </Badge>
                      <span className="font-mono">{f}</span>
                    </p>
                  ))}
                  {report.conflicted.map((c) => (
                    <p
                      key={`x-${c.path}`}
                      className="rounded border border-destructive/50 bg-destructive/10 p-2"
                    >
                      <Badge variant="destructive" className="mr-2">
                        冲突
                      </Badge>
                      <span className="font-mono">{c.path}</span>
                      <span className="ml-2 text-muted-foreground">
                        {c.reason}
                      </span>
                    </p>
                  ))}
                  {report.removed.map((f) => (
                    <p key={`r-${f}`}>
                      <Badge variant="outline" className="mr-2">
                        移除
                      </Badge>
                      <span className="font-mono">{f}</span>
                      <span className="ml-2 text-muted-foreground">
                        （模板已删除，本地保留）
                      </span>
                    </p>
                  ))}
                  {report.updated.length === 0 &&
                    report.created.length === 0 &&
                    report.conflicted.length === 0 &&
                    report.removed.length === 0 && (
                      <p className="text-muted-foreground">
                        模板与项目完全同步，无需更新。
                      </p>
                    )}
                </div>
              </ScrollArea>
            </>
          )}
          {!report && !error && (
            <p className="text-sm text-muted-foreground">等待选择项目…</p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default App;
