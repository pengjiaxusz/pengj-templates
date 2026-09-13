import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Button,
  Input,
  Label,
  Checkbox,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Badge,
  Separator,
  ScrollArea,
} from "@chahu/cha-set";
import {
  Layers,
  FolderPlus,
  RefreshCw,
  FolderOpen,
  CheckCircle2,
  AlertCircle,
  FileCode,
  ArrowRight,
  Code2,
  Sparkles,
  Bot,
  Terminal,
  Boxes,
} from "lucide-react";

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
  needs_review: string[];
  unchanged: number;
}

function App() {
  return (
    <div className="min-h-screen flex flex-col bg-background text-foreground selection:bg-primary/10">
      {/* 顶栏 Header */}
      <header className="border-b border-border/80 bg-card/60 backdrop-blur-md px-6 py-4 sticky top-0 z-20 transition-all">
        <div className="max-w-7xl mx-auto flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="size-9 rounded-lg bg-primary/10 border border-primary/20 flex items-center justify-center text-primary shadow-xs">
              <Layers className="size-5" />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <h1 className="text-base font-semibold tracking-tight">pengj-templates</h1>
                <Badge variant="secondary" className="text-[11px] font-mono px-1.5 py-0">v0.29.1</Badge>
                <Badge variant="outline" className="text-[10px] text-muted-foreground">ChaSet UI</Badge>
              </div>
              <p className="text-xs text-muted-foreground mt-0.5">
                分层模板生成与同步更新工具 · 单一真相源驱动
              </p>
            </div>
          </div>
          <div className="flex items-center gap-2 text-xs text-muted-foreground">
            <span className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-muted/60 border border-border/60">
              <span className="size-1.5 rounded-full bg-emerald-500 animate-pulse" />
              Tauri 2 引擎就绪
            </span>
          </div>
        </div>
      </header>

      {/* 主工作区 */}
      <main className="flex-1 max-w-7xl w-full mx-auto p-6">
        <Tabs defaultValue="generate" className="w-full space-y-6">
          <div className="flex items-center justify-between border-b border-border pb-3">
            <TabsList className="bg-muted/60 p-1 rounded-lg border border-border/60">
              <TabsTrigger value="generate" className="gap-2 px-3 py-1.5 text-xs font-medium cursor-pointer">
                <FolderPlus className="size-3.5" />
                生成新项目
              </TabsTrigger>
              <TabsTrigger value="update" className="gap-2 px-3 py-1.5 text-xs font-medium cursor-pointer">
                <RefreshCw className="size-3.5" />
                同步模板更新
              </TabsTrigger>
            </TabsList>
            <div className="text-xs text-muted-foreground hidden sm:block">
              勾选所需层自由组合 · 模板更新一键无损同步
            </div>
          </div>

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

// ---------- 生成项目 ----------

function GenerateTab() {
  const [layers, setLayers] = useState<LayerInfo[]>([]);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [name, setName] = useState("");
  const [parentDir, setParentDir] = useState("");
  const [edition, setEdition] = useState("2024");
  const [channel, setChannel] = useState("stable");
  const [useSccache, setUseSccache] = useState(false);
  const [useLld, setUseLld] = useState(true);
  const [chinese, setChinese] = useState(false);
  const [skillLang, setSkillLang] = useState("zh");
  const [commitZh, setCommitZh] = useState(true);
  const [commitAndPush, setCommitAndPush] = useState(false);
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
          commit_and_push: commitAndPush,
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

  const hasRustLayer = selected.has("rust") || selected.has("rust-workspace");
  const hasAgentLayer = selected.has("agent");

  return (
    <div className="grid gap-6 lg:grid-cols-12 items-start">
      {/* 左侧配置面板 */}
      <div className="lg:col-span-7 space-y-6">
        <Card className="border-border/80 shadow-xs">
          <CardHeader className="border-b border-border/40 pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <Boxes className="size-4 text-primary" />
              项目基础配置
            </CardTitle>
            <CardDescription>指定新项目的工程命名与生成落脚点</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4 pt-4">
            <div className="space-y-1.5">
              <Label htmlFor="project-name" className="text-xs font-medium">项目名称</Label>
              <Input
                id="project-name"
                value={name}
                placeholder="例如: my-awesome-tool"
                onChange={(e) => setName(e.target.value)}
                className="font-mono text-sm"
              />
            </div>

            <div className="space-y-1.5">
              <Label className="text-xs font-medium">输出父目录</Label>
              <div className="flex gap-2">
                <Input
                  value={parentDir}
                  readOnly
                  placeholder="点击右侧按钮挑选生成目录…"
                  className="font-mono text-xs bg-muted/30"
                />
                <Button variant="outline" onClick={pickDir} className="shrink-0 gap-1.5 text-xs">
                  <FolderOpen className="size-3.5" />
                  浏览
                </Button>
              </div>
            </div>
          </CardContent>
        </Card>

        {/* 分层选择 */}
        <Card className="border-border/80 shadow-xs">
          <CardHeader className="border-b border-border/40 pb-3">
            <div className="flex items-center justify-between">
              <div>
                <CardTitle className="text-base flex items-center gap-2">
                  <Layers className="size-4 text-primary" />
                  选择分层组合
                </CardTitle>
                <CardDescription>按依赖拓扑合并叠加，后层覆盖前层</CardDescription>
              </div>
              <Badge variant="secondary" className="font-mono text-xs">
                已勾选 {selected.size} 层
              </Badge>
            </div>
          </CardHeader>
          <CardContent className="pt-4 space-y-2.5">
            {layers.map((layer) => {
              const isChecked = selected.has(layer.id);
              return (
                <div
                  key={layer.id}
                  onClick={() => toggle(layer.id)}
                  className={`group flex items-start gap-3 rounded-lg border p-3 cursor-pointer transition-all duration-quick select-none ${
                    isChecked
                      ? "border-primary/60 bg-primary/[0.03] shadow-xs"
                      : "border-border/80 hover:border-foreground/30 hover:bg-muted/30"
                  }`}
                >
                  <div className="pt-0.5" onClick={(e) => e.stopPropagation()}>
                    <Checkbox
                      id={`layer-${layer.id}`}
                      checked={isChecked}
                      onCheckedChange={() => toggle(layer.id)}
                    />
                  </div>
                  <div className="flex-1 space-y-1">
                    <div className="flex flex-wrap items-center gap-2">
                      <span className="text-sm font-medium tracking-tight group-hover:text-primary transition-colors">
                        {layer.name}
                      </span>
                      <Badge variant="secondary" className="font-mono text-[11px] px-1.5 py-0">
                        {layer.id}
                      </Badge>
                      <Badge variant="outline" className="text-[11px] px-1.5 py-0 text-muted-foreground">
                        {layer.file_count} 个文件
                      </Badge>
                    </div>
                    <p className="text-xs text-muted-foreground leading-relaxed">
                      {layer.description}
                    </p>
                    {layer.depends.length > 0 && (
                      <p className="text-[11px] text-muted-foreground/80 flex items-center gap-1 pt-0.5">
                        <ArrowRight className="size-2.5" />
                        前置依赖: <span className="font-mono">{layer.depends.join(", ")}</span>
                      </p>
                    )}
                  </div>
                </div>
              );
            })}
            {layers.length === 0 && (
              <p className="text-sm text-muted-foreground text-center py-6">
                暂未加载到可用分层模板
              </p>
            )}
          </CardContent>
        </Card>

        {/* 语言与高级选项 */}
        <Card className="border-border/80 shadow-xs">
          <CardHeader className="border-b border-border/40 pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <Code2 className="size-4 text-primary" />
              技术栈与规范定制
            </CardTitle>
            <CardDescription>精细调控工具链选项与 AI Agent 规范</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4 pt-4">
            {/* Rust 专属选项 */}
            <div className={`space-y-2 p-3 rounded-lg border transition-all ${
              hasRustLayer ? "bg-muted/20 border-border" : "opacity-60 bg-muted/10 border-dashed border-border"
            }`}>
              <div className="flex items-center justify-between">
                <Label className="text-xs font-semibold flex items-center gap-1.5">
                  <Terminal className="size-3.5 text-orange-500" />
                  Rust 工具链参数
                </Label>
                {!hasRustLayer && (
                  <span className="text-[11px] text-muted-foreground">选 rust 层时生效</span>
                )}
              </div>
              <div className="grid grid-cols-2 gap-3 pt-1">
                <div>
                  <span className="text-[11px] text-muted-foreground block mb-1">Edition 版本</span>
                  <select
                    value={edition}
                    onChange={(e) => setEdition(e.target.value)}
                    disabled={!hasRustLayer}
                    className="w-full h-8 rounded-md border border-input bg-background px-2.5 text-xs focus:ring-1 focus:ring-ring outline-none"
                  >
                    {["2015", "2018", "2021", "2024"].map((v) => (
                      <option key={v} value={v}>
                        Edition {v}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <span className="text-[11px] text-muted-foreground block mb-1">Toolchain 通道</span>
                  <select
                    value={channel}
                    onChange={(e) => setChannel(e.target.value)}
                    disabled={!hasRustLayer}
                    className="w-full h-8 rounded-md border border-input bg-background px-2.5 text-xs focus:ring-1 focus:ring-ring outline-none"
                  >
                    {["stable", "beta", "nightly"].map((v) => (
                      <option key={v} value={v}>
                        Channel: {v}
                      </option>
                    ))}
                  </select>
                </div>
              </div>
              <div className="flex items-center gap-5 pt-2 text-xs">
                <label className="flex items-center gap-2 cursor-pointer">
                  <Checkbox
                    checked={useSccache}
                    onCheckedChange={(v) => setUseSccache(!!v)}
                    disabled={!hasRustLayer}
                  />
                  <span>启用 sccache 编译缓存</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <Checkbox
                    checked={useLld}
                    onCheckedChange={(v) => setUseLld(!!v)}
                    disabled={!hasRustLayer}
                  />
                  <span>启用 lld 极速链接器</span>
                </label>
              </div>
            </div>

            {/* 中文编程通用开关 */}
            <div className="flex items-start gap-3 p-3 rounded-lg border border-border hover:bg-muted/20 transition-colors">
              <Checkbox
                checked={chinese}
                onCheckedChange={(v) => setChinese(!!v)}
                id="chinese-opt"
                className="mt-0.5"
              />
              <label htmlFor="chinese-opt" className="flex-1 cursor-pointer space-y-0.5">
                <div className="text-xs font-semibold flex items-center gap-1.5">
                  <Sparkles className="size-3.5 text-amber-500" />
                  中文编程范式 (Chinese Programming)
                </div>
                <p className="text-[11px] text-muted-foreground">
                  放行中文字面量与命名规范，在 AGENTS.md 注入中英锚点拼接与母语本土化标准
                </p>
              </label>
            </div>

            {/* Agent 技能细选 */}
            <div className={`space-y-3 p-3 rounded-lg border transition-all ${
              hasAgentLayer ? "bg-muted/20 border-border" : "opacity-60 bg-muted/10 border-dashed border-border"
            }`}>
              <div className="flex items-center justify-between">
                <Label className="text-xs font-semibold flex items-center gap-1.5">
                  <Bot className="size-3.5 text-blue-500" />
                  Agent 技能与门禁
                </Label>
                <span className="text-[11px] text-muted-foreground">
                  已选 {selectedSkills.size} / {skills.length} 个技能
                </span>
              </div>

              {skills.length > 0 && (
                <ScrollArea className="h-44 rounded-md border border-border/80 bg-background/50 p-2">
                  <div className="space-y-1.5 pr-2">
                    {skills.map((skill) => {
                      const isSkillChecked = selectedSkills.has(skill.name);
                      return (
                        <div
                          key={skill.name}
                          onClick={() => hasAgentLayer && toggleSkill(skill.name)}
                          className={`flex items-start gap-2.5 rounded-md p-2 text-xs cursor-pointer transition-colors ${
                            isSkillChecked ? "bg-primary/[0.04] border border-primary/20" : "hover:bg-muted/40 border border-transparent"
                          }`}
                        >
                          <Checkbox
                            id={`skill-${skill.name}`}
                            checked={isSkillChecked}
                            onCheckedChange={() => toggleSkill(skill.name)}
                            disabled={!hasAgentLayer}
                            className="mt-0.5"
                          />
                          <div className="flex-1">
                            <span className="font-medium font-mono text-[11px]">{skill.name}</span>
                            <p className="text-[11px] text-muted-foreground line-clamp-1">
                              {skill.description}
                            </p>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </ScrollArea>
              )}

              <div className="flex flex-wrap items-center gap-4 pt-1 text-xs">
                <div className="flex items-center gap-2">
                  <span className="text-[11px] text-muted-foreground">技能文档语言:</span>
                  <select
                    value={skillLang}
                    onChange={(e) => setSkillLang(e.target.value)}
                    disabled={!hasAgentLayer}
                    className="h-7 rounded border border-input bg-background px-2 text-xs outline-none"
                  >
                    <option value="zh">中文 (zh)</option>
                    <option value="en">English (en)</option>
                  </select>
                </div>
                <label className="flex items-center gap-2 cursor-pointer">
                  <Checkbox
                    checked={commitZh}
                    onCheckedChange={(v) => setCommitZh(!!v)}
                    disabled={!hasAgentLayer}
                  />
                  <span>提交信息使用中文</span>
                </label>
                <label className="flex items-center gap-2 cursor-pointer">
                  <Checkbox
                    checked={commitAndPush}
                    onCheckedChange={(v) => setCommitAndPush(!!v)}
                    disabled={!hasAgentLayer}
                  />
                  <span>完工回复前提交门禁 (commit & push)</span>
                </label>
              </div>
            </div>

            <Button
              onClick={generate}
              disabled={!canGenerate || busy}
              className="w-full gap-2 text-sm font-medium py-2.5 mt-2"
            >
              {busy ? (
                <>
                  <RefreshCw className="size-4 animate-spin" />
                  生成项目中…
                </>
              ) : (
                <>
                  <FolderPlus className="size-4" />
                  立即生成新项目
                </>
              )}
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* 右侧输出面板 */}
      <div className="lg:col-span-5 sticky top-24 space-y-6">
        <Card className="border-border/80 shadow-xs">
          <CardHeader className="border-b border-border/40 pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <FileCode className="size-4 text-primary" />
              生成产物与报告
            </CardTitle>
            <CardDescription>生成的文件清单与分层拓扑结果</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4 pt-4">
            {error && (
              <div className="rounded-lg border border-destructive/40 bg-destructive/10 p-3.5 text-xs text-destructive flex items-start gap-2.5">
                <AlertCircle className="size-4 shrink-0 mt-0.5" />
                <div className="space-y-1">
                  <span className="font-medium">生成失败</span>
                  <p className="text-[11px] leading-relaxed opacity-90">{error}</p>
                </div>
              </div>
            )}

            {report && (
              <div className="space-y-4">
                <div className="rounded-lg border border-emerald-500/30 bg-emerald-500/10 p-3 flex items-start gap-2.5 text-xs">
                  <CheckCircle2 className="size-4 text-emerald-500 shrink-0 mt-0.5" />
                  <div>
                    <span className="font-semibold text-emerald-600 dark:text-emerald-400">
                      项目生成成功！
                    </span>
                    <p className="text-[11px] text-muted-foreground mt-0.5 font-mono break-all">
                      {report.project_dir}
                    </p>
                  </div>
                </div>

                <div className="space-y-1.5 text-xs">
                  <div className="flex justify-between text-muted-foreground">
                    <span>分层渲染链路:</span>
                    <span className="font-mono text-foreground font-medium">
                      {report.layers.join(" → ")}
                    </span>
                  </div>
                  <div className="flex justify-between text-muted-foreground">
                    <span>产出文件总数:</span>
                    <span className="font-mono text-foreground font-medium">
                      {report.files.length} 个
                    </span>
                  </div>
                </div>

                <Separator />

                <div className="space-y-1.5">
                  <span className="text-xs font-medium text-muted-foreground block">
                    文件索引明细:
                  </span>
                  <ScrollArea className="h-80 rounded-lg border border-border bg-muted/20">
                    <ul className="p-3 space-y-1 font-mono text-[11px]">
                      {report.files.map((file) => (
                        <li key={file} className="text-muted-foreground hover:text-foreground flex items-center gap-2">
                          <span className="size-1 rounded-full bg-primary/40 shrink-0" />
                          <span className="break-all">{file}</span>
                        </li>
                      ))}
                    </ul>
                  </ScrollArea>
                </div>
              </div>
            )}

            {!report && !error && (
              <div className="text-center py-16 space-y-3">
                <div className="size-12 rounded-full bg-muted/60 border border-border/80 flex items-center justify-center mx-auto text-muted-foreground">
                  <Layers className="size-6 opacity-40" />
                </div>
                <div className="space-y-1">
                  <p className="text-xs font-medium text-foreground">等待生成参数配置</p>
                  <p className="text-[11px] text-muted-foreground max-w-xs mx-auto">
                    请在左侧指定项目名称、输出目录并选择模板分层后点击生成
                  </p>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

// ---------- 同步更新 ----------

function UpdateTab() {
  const [projectDir, setProjectDir] = useState("");
  const [busy, setBusy] = useState(false);
  const [report, setReport] = useState<UpdateReport | null>(null);
  const [error, setError] = useState("");
  const [workspaces, setWorkspaces] = useState<string[]>([]);
  const [selectedWorkspace, setSelectedWorkspace] = useState("");
  const [workspaceLoading, setWorkspaceLoading] = useState(false);

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
    <div className="grid gap-6 lg:grid-cols-12 items-start">
      {/* 目标项目配置 */}
      <div className="lg:col-span-6 space-y-6">
        <Card className="border-border/80 shadow-xs">
          <CardHeader className="border-b border-border/40 pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <RefreshCw className="size-4 text-primary" />
              同步目标仓库
            </CardTitle>
            <CardDescription>
              选择由 pengj-templates 生成并纳管的项目（含 .pengj-templates.json）
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4 pt-4">
            <div className="space-y-1.5">
              <Label className="text-xs font-medium">项目根目录</Label>
              <div className="flex gap-2">
                <Input
                  value={projectDir}
                  readOnly
                  placeholder="点击右侧选择存量项目目录…"
                  className="font-mono text-xs bg-muted/30"
                />
                <Button variant="outline" onClick={pickDir} className="shrink-0 gap-1.5 text-xs">
                  <FolderOpen className="size-3.5" />
                  选择
                </Button>
              </div>
            </div>

            {/* 工作空间感知提示 */}
            <div className="space-y-2 pt-1">
              {workspaceLoading && (
                <div className="flex items-center gap-2 text-xs text-muted-foreground py-2">
                  <RefreshCw className="size-3.5 animate-spin" />
                  正在深度探测 *.code-workspace 工作空间…
                </div>
              )}
              {!workspaceLoading && projectDir && workspaces.length > 0 && (
                <div className="rounded-lg border border-border/80 bg-muted/20 p-3 space-y-2">
                  <Label htmlFor="workspace-select" className="text-xs font-semibold flex items-center gap-1.5">
                    <Code2 className="size-3.5 text-primary" />
                    检测到 VS Code 工作空间文件
                  </Label>
                  <select
                    id="workspace-select"
                    value={selectedWorkspace}
                    onChange={(e) => setSelectedWorkspace(e.target.value)}
                    className="w-full h-8 rounded-md border border-input bg-background px-2.5 text-xs focus:ring-1 focus:ring-ring outline-none"
                  >
                    {workspaces.map((w) => (
                      <option key={w} value={w}>
                        {w}
                      </option>
                    ))}
                  </select>
                  <p className="text-[11px] text-muted-foreground">
                    同步时将根据模板最新规则全量更新所有工作空间内的 fileNesting 折叠项
                  </p>
                </div>
              )}
              {!workspaceLoading && projectDir && workspaces.length === 0 && (
                <div className="text-[11px] text-muted-foreground flex items-center gap-1.5 py-1">
                  <Info className="size-3.5" />
                  未发现 *.code-workspace 文件，将针对标准 .vscode/settings.json 实施更新
                </div>
              )}
            </div>

            <Button
              onClick={update}
              disabled={!projectDir || busy}
              className="w-full gap-2 text-sm font-medium py-2.5 mt-2"
            >
              {busy ? (
                <>
                  <RefreshCw className="size-4 animate-spin" />
                  正在比对并同步上游模板…
                </>
              ) : (
                <>
                  <RefreshCw className="size-4" />
                  开始同步模板更新
                </>
              )}
            </Button>
          </CardContent>
        </Card>
      </div>

      {/* 更新报告面板 */}
      <div className="lg:col-span-6 space-y-6">
        <Card className="border-border/80 shadow-xs">
          <CardHeader className="border-b border-border/40 pb-3">
            <CardTitle className="text-base flex items-center gap-2">
              <FileCode className="size-4 text-primary" />
              模板变更同步报告
            </CardTitle>
            <CardDescription>受管块原位替换、无损配置合并与过渡区审查</CardDescription>
          </CardHeader>
          <CardContent className="space-y-4 pt-4">
            {error && (
              <div className="rounded-lg border border-destructive/40 bg-destructive/10 p-3.5 text-xs text-destructive flex items-start gap-2.5">
                <AlertCircle className="size-4 shrink-0 mt-0.5" />
                <div className="space-y-1">
                  <span className="font-medium">同步中断</span>
                  <p className="text-[11px] leading-relaxed opacity-90">{error}</p>
                </div>
              </div>
            )}

            {report && (
              <div className="space-y-4">
                <div className="flex flex-wrap items-center justify-between gap-2 p-3 rounded-lg border border-border/80 bg-muted/20 text-xs">
                  <div>
                    <span className="text-muted-foreground">项目: </span>
                    <span className="font-semibold">{report.project_name}</span>
                  </div>
                  <div className="text-muted-foreground font-mono text-[11px]">
                    {report.layers.join(" → ")}
                  </div>
                </div>

                <div className="grid grid-cols-3 sm:grid-cols-6 gap-2 text-center">
                  <div className="p-2 rounded border border-border/60 bg-muted/10">
                    <span className="text-[10px] text-muted-foreground block">更新</span>
                    <span className="text-sm font-semibold font-mono text-primary">{report.updated.length}</span>
                  </div>
                  <div className="p-2 rounded border border-border/60 bg-muted/10">
                    <span className="text-[10px] text-muted-foreground block">新增</span>
                    <span className="text-sm font-semibold font-mono text-emerald-600 dark:text-emerald-400">{report.created.length}</span>
                  </div>
                  <div className="p-2 rounded border border-border/60 bg-muted/10">
                    <span className="text-[10px] text-muted-foreground block">待复核</span>
                    <span className="text-sm font-semibold font-mono text-amber-500">{report.needs_review.length}</span>
                  </div>
                  <div className="p-2 rounded border border-border/60 bg-muted/10">
                    <span className="text-[10px] text-muted-foreground block">冲突</span>
                    <span className="text-sm font-semibold font-mono text-destructive">{report.conflicted.length}</span>
                  </div>
                  <div className="p-2 rounded border border-border/60 bg-muted/10">
                    <span className="text-[10px] text-muted-foreground block">移除</span>
                    <span className="text-sm font-semibold font-mono text-muted-foreground">{report.removed.length}</span>
                  </div>
                  <div className="p-2 rounded border border-border/60 bg-muted/10">
                    <span className="text-[10px] text-muted-foreground block">未变</span>
                    <span className="text-sm font-semibold font-mono text-muted-foreground">{report.unchanged}</span>
                  </div>
                </div>

                <Separator />

                <ScrollArea className="h-72 rounded-lg border border-border/80 bg-muted/20">
                  <div className="p-3 space-y-2 text-xs">
                    {report.updated.map((f) => (
                      <div key={`u-${f}`} className="flex items-center gap-2 font-mono text-[11px]">
                        <Badge variant="default" className="text-[10px] px-1.5 py-0">更新</Badge>
                        <span className="text-foreground">{f}</span>
                      </div>
                    ))}
                    {report.created.map((f) => (
                      <div key={`c-${f}`} className="flex items-center gap-2 font-mono text-[11px]">
                        <Badge variant="secondary" className="text-[10px] px-1.5 py-0">新增</Badge>
                        <span className="text-foreground">{f}</span>
                      </div>
                    ))}
                    {report.conflicted.map((c) => (
                      <div
                        key={`x-${c.path}`}
                        className="rounded-md border border-destructive/40 bg-destructive/10 p-2 text-xs space-y-1"
                      >
                        <div className="flex items-center gap-1.5">
                          <Badge variant="destructive" className="text-[10px] px-1.5 py-0">冲突保护</Badge>
                          <span className="font-mono text-[11px] font-semibold">{c.path}</span>
                        </div>
                        <p className="text-[11px] text-muted-foreground pl-1">{c.reason}</p>
                      </div>
                    ))}
                    {report.needs_review.map((f) => (
                      <div
                        key={`n-${f}`}
                        className="rounded-md border border-amber-500/40 bg-amber-500/10 p-2 text-xs space-y-1"
                      >
                        <div className="flex items-center gap-1.5">
                          <Badge className="text-[10px] px-1.5 py-0 bg-amber-500 text-white">待复核</Badge>
                          <span className="font-mono text-[11px] font-semibold">{f}</span>
                        </div>
                      </div>
                    ))}
                    {report.removed.map((f) => (
                      <div key={`r-${f}`} className="flex items-center gap-2 font-mono text-[11px] text-muted-foreground">
                        <Badge variant="outline" className="text-[10px] px-1.5 py-0">移除</Badge>
                        <span>{f}</span>
                        <span className="text-[10px] text-muted-foreground/60">(上游已删，本地保留)</span>
                      </div>
                    ))}
                    {report.updated.length === 0 &&
                      report.created.length === 0 &&
                      report.conflicted.length === 0 &&
                      report.removed.length === 0 &&
                      report.needs_review.length === 0 && (
                        <div className="py-8 text-center text-muted-foreground space-y-1">
                          <CheckCircle2 className="size-6 text-emerald-500 mx-auto" />
                          <p className="text-xs font-medium">项目与最新模板完全一致</p>
                          <p className="text-[11px]">没有需要同步的变更</p>
                        </div>
                      )}
                  </div>
                </ScrollArea>
              </div>
            )}

            {!report && !error && (
              <div className="text-center py-16 space-y-3">
                <div className="size-12 rounded-full bg-muted/60 border border-border/80 flex items-center justify-center mx-auto text-muted-foreground">
                  <RefreshCw className="size-6 opacity-40" />
                </div>
                <div className="space-y-1">
                  <p className="text-xs font-medium text-foreground">等待选择目标项目</p>
                  <p className="text-[11px] text-muted-foreground max-w-xs mx-auto">
                    请在左侧指定已受管的项目目录后点击开始同步
                  </p>
                </div>
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function Info(props: React.SVGProps<SVGSVGElement>) {
  return (
    <svg
      {...props}
      xmlns="http://www.w3.org/2000/svg"
      width="24"
      height="24"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="12" cy="12" r="10" />
      <path d="M12 16v-4" />
      <path d="M12 8h.01" />
    </svg>
  );
}

export default App;
