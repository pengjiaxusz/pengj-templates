# Changelog

## [0.84.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.83.0...v0.84.0) (2026-11-22)


### Features

* **core,templates:** 自动探测仓库变量、新增无交互 doctor 引导脚本并升级 CONTRIBUTING 模板 ([756ddbe](https://github.com/pengjiaxusz/pengj-templates/commit/756ddbe492aa7e3bd308ec25295a3b626d12bc2c))
* **templates,core:** 新增双语托管块 CONTRIBUTING.md 模板与置顶合并支持 ([d1ccb4b](https://github.com/pengjiaxusz/pengj-templates/commit/d1ccb4b678c84511f13fe565ef736b5628f3f15c))

## [0.83.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.82.0...v0.83.0) (2026-11-22)


### Features

* **core:** Linux 下 doctor 自检补充 ld.lld 检查与安装提示 ([25ce076](https://github.com/pengjiaxusz/pengj-templates/commit/25ce07690d23277ab3d071e68789171eb1a36561))


### Bug Fixes

* **ci:** Linux CLI 发布产物改用 musl 静态链接以兼容低版本 glibc ([5507c4d](https://github.com/pengjiaxusz/pengj-templates/commit/5507c4df2d4c73b3d9907abda614e458b98050c3))
* **templates:** 优化 just 配方在 Linux 下的回退与平台分支兼容性 ([8812b50](https://github.com/pengjiaxusz/pengj-templates/commit/8812b50b4720071c85a54d35599e166d1bd862c2))

## [0.82.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.81.1...v0.82.0) (2026-11-22)


### Features

* **cli:** 新增 doctor 子命令支持按层自检、本地修复、双语与 JSON 输出 ([d635934](https://github.com/pengjiaxusz/pengj-templates/commit/d6359346a12130056b90bfd436ccd6cad6fba822))
* **core:** 新增双语项目环境自检核心模块与子仓库双语体检接口 ([c618245](https://github.com/pengjiaxusz/pengj-templates/commit/c6182450962362eb130876afb672dc73033db975))
* **gate:** 新增目录隔离且结构化返回的钩子安装与竞品检测接口 ([9f480cf](https://github.com/pengjiaxusz/pengj-templates/commit/9f480cf2fcd758693cb36b47ef15c8ccf0219da6))

## [0.81.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.81.0...v0.81.1) (2026-11-22)


### Bug Fixes

* **core:** 符号链接子模块不再让 repo init 失败 ([bc3b5fc](https://github.com/pengjiaxusz/pengj-templates/commit/bc3b5fc96e548b670952673e1c421953231bc82c))

## [0.81.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.80.1...v0.81.0) (2026-11-22)


### ⚠ BREAKING CHANGES

* **templates:** 把 issue-sync 并入 patch-handoff，随基线技能一起下发
* **templates:** 撤掉 bque 层 just 配方，重型命令改直调原生 bque

### Features

* **cli:** update 支持 --commit/--push 一条命令收口模板同步 ([1a7e4c3](https://github.com/pengjiaxusz/pengj-templates/commit/1a7e4c32f4bb84630821c42adec76357de90660f))
* **cli:** 新增 self update，用命令替代 install 脚本升级自己 ([8f26304](https://github.com/pengjiaxusz/pengj-templates/commit/8f26304cefe67d2e0d38a1294b6fd88f4054e927))
* **core:** 将 patch-handoff 设为默认基线技能 ([0755f46](https://github.com/pengjiaxusz/pengj-templates/commit/0755f464c480f5babf2798795d5ec14aa8a1ddb6))
* **templates:** patch-handoff 作者归一改为自动解析仓库拥有者 ([5e8e5de](https://github.com/pengjiaxusz/pengj-templates/commit/5e8e5de9895abbdb2d44c08205ac4c84e5a38f38))
* **templates:** 把 issue-sync 并入 patch-handoff，随基线技能一起下发 ([3cda7cb](https://github.com/pengjiaxusz/pengj-templates/commit/3cda7cb761cceda51afb0f97a9bc26db862c8ac3))
* **templates:** 撤掉 bque 层 just 配方，重型命令改直调原生 bque ([3f410b0](https://github.com/pengjiaxusz/pengj-templates/commit/3f410b0438a44daaf1f6a9a4e39c44c58d9bfb85))
* **templates:** 新增 write-just 技能随 just 层自动下发 ([c625689](https://github.com/pengjiaxusz/pengj-templates/commit/c625689fd82eb9f7d742c53adee39f4e51c69676))


### Bug Fixes

* **cli:** safe_relative 限定为 windows/test 作用域，修复 Linux CI dead-code 拦截 ([afd6489](https://github.com/pengjiaxusz/pengj-templates/commit/afd64893f07c0d9fafe7e8850f7b726b538bf13c))
* **templates:** write-just 校验脚本识别 just q-run/q-bench 包装 ([1b3d70c](https://github.com/pengjiaxusz/pengj-templates/commit/1b3d70c9b748a9ec57550115360d34b98a472adc))


### Reverts

* 撤回 update 的 --commit/--push，收尾职责交回 ps1 ([fb2043c](https://github.com/pengjiaxusz/pengj-templates/commit/fb2043c51d9e091b1499a234e6346411d7d7a709))

## [0.80.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.80.0...v0.80.1) (2026-11-21)


### Bug Fixes

* **ci:** 兜住失效 base 与反引号命令替换 ([af07f5c](https://github.com/pengjiaxusz/pengj-templates/commit/af07f5c95b5234124c4a80f8411e82992b1fed58))

## [0.80.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.79.0...v0.80.0) (2026-11-21)


### Features

* **templates:** 新增 patch-handoff 技能验收离线补丁交接包 ([d1018c6](https://github.com/pengjiaxusz/pengj-templates/commit/d1018c6da1e9f6022f13a20f392f5e1dfe1a54fb))

## [0.79.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.78.0...v0.79.0) (2026-11-21)


### Features

* **templates:** 新增 issue-sync 技能搬运 GitHub Issues ([44c42e8](https://github.com/pengjiaxusz/pengj-templates/commit/44c42e861c9b0ed9147d040563367595f0313ad2))

## [0.78.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.77.1...v0.78.0) (2026-11-21)


### Features

* **gate:** 新增 pre-push 钩子，补上批量落补丁漏掉的检查 ([f2594bf](https://github.com/pengjiaxusz/pengj-templates/commit/f2594bfc6bb21777a0ea36ee20cb95d8cd23f4df))

## [0.77.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.77.0...v0.77.1) (2026-11-19)


### Bug Fixes

* **core:** 模板根指错时拒绝清空受管文件 ([79f8ce3](https://github.com/pengjiaxusz/pengj-templates/commit/79f8ce3ed5590eaf0efb942f640a162cf64d95d4))

## [0.77.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.76.1...v0.77.0) (2026-11-19)


### Features

* **templates:** 精简 AGENTS.md 结构并收编注释与代码语言规范 ([c593b55](https://github.com/pengjiaxusz/pengj-templates/commit/c593b55e995337f3b2727a9f586b6109e2752960))

## [0.76.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.76.0...v0.76.1) (2026-11-18)


### Bug Fixes

* **core:** 子模块形态下主工作树取检出目录而非 git 目录 ([00c3550](https://github.com/pengjiaxusz/pengj-templates/commit/00c3550590a1bdfd38d36704bcb4165505a2fed0))

## [0.76.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.75.1...v0.76.0) (2026-11-18)


### ⚠ BREAKING CHANGES

* **core:** target-dir 不再指向平台数据目录，--json 的 project_key 改为 main_worktree

### Features

* **core:** 共享编译目录改为主工作树的 target ([820058d](https://github.com/pengjiaxusz/pengj-templates/commit/820058d8c46d2ad0f3270c6489546ac0e391daf2))

## [0.75.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.75.0...v0.75.1) (2026-11-18)


### Bug Fixes

* **core:** 共享目录覆盖值比较两侧同时规范化 ([ecbf49f](https://github.com/pengjiaxusz/pengj-templates/commit/ecbf49f8ba932c44e22d5f2ad02802a285d25c4b))

## [0.75.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.74.0...v0.75.0) (2026-11-18)


### Features

* **cli:** 新增子仓库与工作树子命令 ([4c19c26](https://github.com/pengjiaxusz/pengj-templates/commit/4c19c26dd06750e30a5a632c009fe613e6b85e54))
* **core:** 新增子仓库登记表与共享编译目录核心模块 ([e6e8ab7](https://github.com/pengjiaxusz/pengj-templates/commit/e6e8ab726dc9a6396e77bda804162968cb17c3b3))
* **templates:** 新增子仓库与工作树两个可独立勾选的层 ([74bcbc1](https://github.com/pengjiaxusz/pengj-templates/commit/74bcbc1fd48b5fa9097afed19c6382b92c7e4386))

## [0.74.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.73.0...v0.74.0) (2026-11-17)


### ⚠ BREAKING CHANGES

* 存量项目不再被识别。把 .pengj-templates.json、.gate.toml、 scripts/、just/ 搬进 .pengj-templates/（分别改名为 manifest.json、gate.toml、 scripts/、just/）后重跑 update 即可，报错信息里带同样的清单。

### Features

* 模板产物收敛到 .pengj-templates/ ([3c04121](https://github.com/pengjiaxusz/pengj-templates/commit/3c041214ce7c5ef61f9ed72e3b880262a261eafb))

## [0.73.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.72.0...v0.73.0) (2026-11-17)


### Features

* **agent:** 补齐 workbuddy 链接并把 CLI 缺失判为错误 ([007ed7c](https://github.com/pengjiaxusz/pengj-templates/commit/007ed7c9c28a0a3cd8ae1b3eaeca8f0aebdbb7e0))

## [0.72.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.71.0...v0.72.0) (2026-11-17)


### Features

* **cli:** 新增 CLI 自身更新检查 ([db8240e](https://github.com/pengjiaxusz/pengj-templates/commit/db8240ef7713e5d587cd1cde661c3d1069b69590))
* **core:** update_ignore 说明块随模板刷新并投递迁移提示 ([18b6d8e](https://github.com/pengjiaxusz/pengj-templates/commit/18b6d8e52db0d6c05d73a4f4c1e553ec17fa763e))
* **templates:** gate 配置改为「托管说明块 + 纯键值」 ([7008fc9](https://github.com/pengjiaxusz/pengj-templates/commit/7008fc93b06b4ebc9655827731b3e7acf34ae825))

## [0.71.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.70.0...v0.71.0) (2026-11-17)


### Features

* **ci:** 服务端门禁检出子模块 ([6d35e9a](https://github.com/pengjiaxusz/pengj-templates/commit/6d35e9a419a71337392eff0007b50a9613306a8d))

## [0.70.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.69.0...v0.70.0) (2026-11-17)


### Features

* **agent:** 合入 mattpocock 工程与效率技能集 ([b374cf0](https://github.com/pengjiaxusz/pengj-templates/commit/b374cf0109d93966c7b7ccd14258144706ce9e33))

## [0.69.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.68.0...v0.69.0) (2026-11-17)


### Features

* **core:** 新增 repo 初始化能力并接入 doctor ([c1b0e3b](https://github.com/pengjiaxusz/pengj-templates/commit/c1b0e3bcb35d94cf5de9d94ae727e00c3eed5d41))

## [0.68.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.67.0...v0.68.0) (2026-11-17)


### Features

* **core:** 修复 doctor.d 文档语言并新增双向文案门禁 ([3e0ef5a](https://github.com/pengjiaxusz/pengj-templates/commit/3e0ef5ae61a1890c6668887454ab451ec59a105a))

## [0.67.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.66.0...v0.67.0) (2026-11-17)


### ⚠ BREAKING CHANGES

* **gate:** commit.scopes 空列表从放行所有改为拒绝所有，老项目需设 ["*"] 恢复旧行为

### Features

* **gate:** scope 白名单改为拒绝优先的三态策略 ([991a777](https://github.com/pengjiaxusz/pengj-templates/commit/991a77799cf543eea7811ee478ee9aac6b6f1579))

## [0.66.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.65.3...v0.66.0) (2026-11-17)


### Features

* **cli:** 一键安装脚本按系统语言显示中英文 ([faea56b](https://github.com/pengjiaxusz/pengj-templates/commit/faea56bae8a82e07cc96883b033878fad82e945d))
* **templates:** doctor 按语言轴双语文案并移除 legacy 检测 ([9c8c8c4](https://github.com/pengjiaxusz/pengj-templates/commit/9c8c8c4b192449ac5dc4ff92b370345e5dd84897))

## [0.65.3](https://github.com/pengjiaxusz/pengj-templates/compare/v0.65.2...v0.65.3) (2026-11-17)


### Bug Fixes

* **core:** 保留 update_ignore 文件的托管账本记录 ([fd0346a](https://github.com/pengjiaxusz/pengj-templates/commit/fd0346a6d31df653e4b6c8346ebd53e2556e644b))

## [0.65.2](https://github.com/pengjiaxusz/pengj-templates/compare/v0.65.1...v0.65.2) (2026-11-17)


### Bug Fixes

* **core:** update 补写缺失的 update_ignore 文件 ([32ab832](https://github.com/pengjiaxusz/pengj-templates/commit/32ab832ab70dd8e3533091b808eab342d222e007))

## [0.65.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.65.0...v0.65.1) (2026-11-17)


### Bug Fixes

* **templates:** 环境自检按实际钩子状态判定激活结果 ([e3167b2](https://github.com/pengjiaxusz/pengj-templates/commit/e3167b2a99936973865cbbf897ce8ae10b907d65))

## [0.65.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.64.0...v0.65.0) (2026-11-17)


### Features

* **templates:** 新增统一 doctor 并按项目激活门禁钩子 ([045d800](https://github.com/pengjiaxusz/pengj-templates/commit/045d8007965bb1276ca81087751fbfee8e5a3690))

## [0.64.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.63.1...v0.64.0) (2026-11-17)


### Features

* **gate:** 没装 CLI 时的四道防线兜底 ([de7fe6b](https://github.com/pengjiaxusz/pengj-templates/commit/de7fe6b85bee76774b558dc70fe3b9e94e74020b))

## [0.63.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.63.0...v0.63.1) (2026-11-17)


### Bug Fixes

* **gate:** 英文输出改用 ASCII 标点,避免 Windows 控制台乱码 ([3b84003](https://github.com/pengjiaxusz/pengj-templates/commit/3b840037cb5b5e889d2e707e6c8b0bec48b248e3))

## [0.63.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.62.4...v0.63.0) (2026-11-17)


### Features

* **gate:** 全局钩子自动分发 + 服务端 CI 门禁层 ([0f156b0](https://github.com/pengjiaxusz/pengj-templates/commit/0f156b02d1dbdeb200b6b3a20b9bc0cd9bdc014a))

## [0.62.4](https://github.com/pengjiaxusz/pengj-templates/compare/v0.62.3...v0.62.4) (2026-11-17)


### Bug Fixes

* **core:** 受管块注释跟随 skill_lang,消除引擎层英文残留 ([052465b](https://github.com/pengjiaxusz/pengj-templates/commit/052465b5ef5b8245eb5d1519058d3df0c1d846b5))

## [0.62.3](https://github.com/pengjiaxusz/pengj-templates/compare/v0.62.2...v0.62.3) (2026-11-17)


### Bug Fixes

* **templates:** 补齐模板层漏掉的 skill_lang 双语分支 ([f75ca89](https://github.com/pengjiaxusz/pengj-templates/commit/f75ca891decd0ec8d295612e8389ea51f4590db8))

## [0.62.2](https://github.com/pengjiaxusz/pengj-templates/compare/v0.62.1...v0.62.2) (2026-11-17)


### Bug Fixes

* **core:** 技能路径断言改用平台原生分隔符 ([391de7a](https://github.com/pengjiaxusz/pengj-templates/commit/391de7aaa85b59b145f7743b894a8d7a36e8289b))

## [0.62.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.62.0...v0.62.1) (2026-11-17)


### Bug Fixes

* **main:** 仓库自身 cargo 配置同步改走 cc -fuse-ld=lld ([923a21f](https://github.com/pengjiaxusz/pengj-templates/commit/923a21fd998040f44cbbfa9ced5ed52f6979cd98))
* **templates:** Linux 链接改走编译器驱动修复 lld 找不到系统库 ([3e70459](https://github.com/pengjiaxusz/pengj-templates/commit/3e70459fd960fe43a34d38508061803a6ce9bbc6))

## [0.62.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.61.0-beta.1...v0.62.0) (2026-11-17)


### Features

* **ci:** 补齐 CLI 一键安装脚本与 bundle 打包 ([e938046](https://github.com/pengjiaxusz/pengj-templates/commit/e93804613d36cfa096cf04b5a8afa2a2bd9d17d8))
* **core:** 扩展模板根定位链支持标准安装布局 ([4f80da8](https://github.com/pengjiaxusz/pengj-templates/commit/4f80da888d7527e796890192acdb28780aaf1260))


### Bug Fixes

* **ci:** Linux runner 安装 lld 修复链接器缺失 ([4759d3c](https://github.com/pengjiaxusz/pengj-templates/commit/4759d3c759d09e146fdc9767d4c1e866af21fc66))
* **main:** 修复版本锚点回退并切到稳定版发版 ([e56b3f3](https://github.com/pengjiaxusz/pengj-templates/commit/e56b3f38fc94cdad707afced73833d844fac9442))

## [0.61.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.60.0-beta.1...v0.61.0-beta.1) (2026-11-16)


### Features

* **core:** 安装钩子时自动记录可执行位 ([6ea0538](https://github.com/pengjiaxusz/pengj-templates/commit/6ea053856080a320e038a6dce516ea9a551d71c0))
* **core:** 新增 Rust 提交门禁工具 ([d0a709c](https://github.com/pengjiaxusz/pengj-templates/commit/d0a709c234be1edd425d79530fa0a27bf19a5529))
* **core:** 钩子安装增加防覆盖检查 ([8e51985](https://github.com/pengjiaxusz/pengj-templates/commit/8e519859722c8071d81f0a0c8ce8f8abae45b566))
* **core:** 验证门禁仍然生效 ([808b9bd](https://github.com/pengjiaxusz/pengj-templates/commit/808b9bdbae749740410b04da09476459a381d75a))
* **templates:** lefthook 层迁移为 gate 提交门禁 ([405a9df](https://github.com/pengjiaxusz/pengj-templates/commit/405a9df7ede209cc8f8fddaf95fe96d26370cfd0))


### Bug Fixes

* **core:** 记录钩子脚本的可执行位 ([ff6c1e7](https://github.com/pengjiaxusz/pengj-templates/commit/ff6c1e73e10104b41298d88b76571ec8c221a660))

## [0.60.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.59.0-beta.1...v0.60.0-beta.1) (2026-11-15)


### Features

* **templates:** 中文模式只判定面向人的文案 ([3b7caff](https://github.com/pengjiaxusz/pengj-templates/commit/3b7caffdabb0ae944b851dd2aec5b074ee63c0ff))

## [0.59.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.58.0-beta.1...v0.59.0-beta.1) (2026-11-15)


### Features

* **templates:** 全量审计改为从仓库根扫描以适配任意布局 ([b19bb4a](https://github.com/pengjiaxusz/pengj-templates/commit/b19bb4ae9645287fc0e6f537da098baf69206677))

## [0.58.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.57.3-beta.1...v0.58.0-beta.1) (2026-11-15)


### Features

* **templates:** 代码语言守卫改为非英文占比并随 lefthook 层生成 ([59ea7e6](https://github.com/pengjiaxusz/pengj-templates/commit/59ea7e655a8a4630c6bfbee7faec3b1a316ce5af))

## [0.57.3-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.57.2-beta.1...v0.57.3-beta.1) (2026-11-15)


### Bug Fixes

* **core:** 补齐被删除的受管文件并在审计中统计缺失 ([8fe1b08](https://github.com/pengjiaxusz/pengj-templates/commit/8fe1b0844b0c76362a3f464f8e277d3f819381e4))

## [0.57.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.57.1-beta.1...v0.57.2-beta.1) (2026-11-15)


### Bug Fixes

* **templates:** 模板源文件固定 LF，避免 CRLF 被原样渲染到下游 ([b7e2897](https://github.com/pengjiaxusz/pengj-templates/commit/b7e28979244cd423d25982da2bade8c92da7b947))

## [0.57.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.57.0-beta.1...v0.57.1-beta.1) (2026-11-15)


### Bug Fixes

* **templates:** 托管技能资产固定 LF，消除更新时的换行抖动 ([d758679](https://github.com/pengjiaxusz/pengj-templates/commit/d758679ddbe584ddd0c73c0f34b4c40ef36aad46))

## [0.57.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.56.0-beta.1...v0.57.0-beta.1) (2026-11-15)


### Features

* **core:** 自动清理模板中已删除的技能名 ([8456c15](https://github.com/pengjiaxusz/pengj-templates/commit/8456c15437cca98cacfa4fd8de14ca454bf4037d))

## [0.56.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.55.2-beta.1...v0.56.0-beta.1) (2026-11-15)


### Features

* **agent:** 以通用 write-docs 技能取代 arch-align ([8ef1118](https://github.com/pengjiaxusz/pengj-templates/commit/8ef11180c0fe2628f4b094873bc33c58657ffa7f))

## [0.55.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.55.1-beta.1...v0.55.2-beta.1) (2026-11-14)


### Bug Fixes

* **templates:** 移除 export-ignore，发布包保留仓库规则与 AI 上下文 ([41e27f4](https://github.com/pengjiaxusz/pengj-templates/commit/41e27f48eac987e524d48e534c97d3b803f3f902))

## [0.55.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.55.0-beta.1...v0.55.1-beta.1) (2026-11-13)


### Bug Fixes

* **templates:** 修复提交门禁静默失效并让模板同步自动提交下游项目 ([c49421b](https://github.com/pengjiaxusz/pengj-templates/commit/c49421b41306bfe5279a1f3b9bdfa2e527cf64cd))

## [0.55.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.54.0-beta.1...v0.55.0-beta.1) (2026-11-11)


### Features

* **templates:** 统一启用 pre-commit 自动格式化并解耦 fmt 与 fmt-check ([cadb6b9](https://github.com/pengjiaxusz/pengj-templates/commit/cadb6b9e737b37b47f240cc2d1e3cdfe2a5ee063))

## [0.54.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.53.0-beta.1...v0.54.0-beta.1) (2026-11-11)


### Features

* **agent:** 将三元锚定与代价矩阵等架构模式反哺至 arch-align 技能 ([69559a7](https://github.com/pengjiaxusz/pengj-templates/commit/69559a7d0e63e9017bf85b69ac6fb1ae3e252072))

## [0.53.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.52.1-beta.1...v0.53.0-beta.1) (2026-11-10)


### Features

* **core:** 新增语言变体文件按语言轴二选一 ([d133f70](https://github.com/pengjiaxusz/pengj-templates/commit/d133f70a0997cf8ba542b74b0fc71ad7ccaa9a6c))

## [0.52.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.52.0-beta.1...v0.52.1-beta.1) (2026-11-10)


### Bug Fixes

* **templates:** 按语言轴双语渲染 just 与 lefthook 模板 ([34dd084](https://github.com/pengjiaxusz/pengj-templates/commit/34dd08449045770f014d0cd8acc82142e1286b86))

## [0.52.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.51.0-beta.1...v0.52.0-beta.1) (2026-11-09)


### Features

* **cli:** 新增 add-layer 命令与配置提示输出 ([aa9b369](https://github.com/pengjiaxusz/pengj-templates/commit/aa9b369d979f55bf7475f7c4f84eeccb987a9426))
* **core:** 新增增量加层与排队技能联动 ([4e3b8ea](https://github.com/pengjiaxusz/pengj-templates/commit/4e3b8ea09a22eb437e9ffae0680e3b29efecc25c))
* **templates:** 新增 just 层并重构排队与语言配方装配 ([6b2704e](https://github.com/pengjiaxusz/pengj-templates/commit/6b2704ea33e8998ea7d0f382d2e430043e2e34b5))

## [0.51.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.9-beta.1...v0.51.0-beta.1) (2026-11-08)


### Features

* **templates:** 全面英文化 agent 技能模板与脚本并支持更新 frontmatter 元数据 ([a5229f0](https://github.com/pengjiaxusz/pengj-templates/commit/a5229f03bd4ce37405be05f9dd5def1f7e28c7d6))

## [0.50.9-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.8-beta.1...v0.50.9-beta.1) (2026-11-07)


### Bug Fixes

* **agent:** 严禁分叉分支部分重置并为同步恢复增加跳过选项 ([19d4bc1](https://github.com/pengjiaxusz/pengj-templates/commit/19d4bc1eeecdbb6463f3f4e3568ab573791b1a11))

## [0.50.8-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.7-beta.1...v0.50.8-beta.1) (2026-11-06)


### Bug Fixes

* **agent:** 避免长行超长导致 Windows 命令行溢出报错 ([ca11976](https://github.com/pengjiaxusz/pengj-templates/commit/ca119768349c1bf1bdc1a43cda32c13a6006c17e))

## [0.50.7-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.6-beta.1...v0.50.7-beta.1) (2026-11-06)


### Bug Fixes

* **agent:** 补充 continue-sync 缺失的 Get-RemoteNames 与 Test-ValidBranchName 函数 ([64900b1](https://github.com/pengjiaxusz/pengj-templates/commit/64900b14d19d2600830ac278f583024a05b30f52))

## [0.50.6-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.5-beta.1...v0.50.6-beta.1) (2026-11-06)


### Bug Fixes

* **agent:** 补充 continue-sync 双分支 Tree-Diff Guard 上下文漂移与后继容错 ([98f926e](https://github.com/pengjiaxusz/pengj-templates/commit/98f926e7342f7a01b9e48c8bf0fdb9cc632089ba))

## [0.50.5-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.4-beta.1...v0.50.5-beta.1) (2026-11-06)


### Bug Fixes

* **agent:** 增强 Tree-Diff Guard 审计支持集成分支全局保全与后继提交吸收容错 ([65b40de](https://github.com/pengjiaxusz/pengj-templates/commit/65b40de3bff095010b190d4acb8e1388f7e6c9dc))

## [0.50.4-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.3-beta.1...v0.50.4-beta.1) (2026-11-06)


### Bug Fixes

* **agent:** 增强 Test-ContentPreserved 校验支持早前提交已包含改动的容错 ([ecc813d](https://github.com/pengjiaxusz/pengj-templates/commit/ecc813d416c32542fd75f83bde34bd6ddd4ad916))

## [0.50.3-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.2-beta.1...v0.50.3-beta.1) (2026-11-06)


### Bug Fixes

* **agent:** 修复分支枚举消歧短名导致的递归嵌套分支与 remote 同名污染 ([e202037](https://github.com/pengjiaxusz/pengj-templates/commit/e20203706de4af84ce0ff22714918ad3c52da7a4))

## [0.50.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.1-beta.1...v0.50.2-beta.1) (2026-11-05)


### Bug Fixes

* **agent:** 补充 continue-sync 遍历有快照源分支时的补丁上下文漂移容错 ([8d95cf2](https://github.com/pengjiaxusz/pengj-templates/commit/8d95cf2a592a2d5366b6c72c5e42e21bd64a00ae))

## [0.50.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.50.0-beta.1...v0.50.1-beta.1) (2026-11-04)


### Bug Fixes

* **agent:** 修复纯删除提交在内容保全校验中误判 diff 头部导致校验失败 ([69c4663](https://github.com/pengjiaxusz/pengj-templates/commit/69c4663b3ff943393d306672339d7d4eff633002))

## [0.50.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.49.0-beta.1...v0.50.0-beta.1) (2026-11-03)


### Features

* **agent:** mcp-bootstrap 支持多宿主构建与换源重装 ([18c2f61](https://github.com/pengjiaxusz/pengj-templates/commit/18c2f61326a363758b0b30e88f9e3d39adbb3d79))

## [0.49.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.48.0-beta.1...v0.49.0-beta.1) (2026-11-03)


### Features

* **core:** 基线技能随 agent 层始终生成 ([ad0600c](https://github.com/pengjiaxusz/pengj-templates/commit/ad0600c0928b93604d8cfaaf3f1b0c10b0484e04))
* **templates:** mcp-bootstrap 兼容无 MCP 声明项目并双语化专属区 ([6447caf](https://github.com/pengjiaxusz/pengj-templates/commit/6447caf024ddc22b93c5d5ff9a98585e892a30f7))

## [0.48.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.47.0-beta.1...v0.48.0-beta.1) (2026-11-03)


### Features

* **templates:** 新增 mcp-bootstrap 技能（项目内 MCP 安装与握手校验） ([069bb58](https://github.com/pengjiaxusz/pengj-templates/commit/069bb58e5a1ec0584502227f20754a9be8e50bd2))

## [0.47.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.46.0-beta.1...v0.47.0-beta.1) (2026-11-03)


### Features

* **mcp:** 附属产物改用托管源码缓存并支持自动浅克隆 ([483f8b2](https://github.com/pengjiaxusz/pengj-templates/commit/483f8b2632d50611c37a182d429053d5fecc9296))

## [0.46.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.45.2-beta.1...v0.46.0-beta.1) (2026-11-03)


### Features

* **cli:** 新增 mcp 子命令串联注册与巡检 ([8bfbb1e](https://github.com/pengjiaxusz/pengj-templates/commit/8bfbb1ee35dfcd3df2409c0c65acaa2082c1b8f1))
* **mcp:** 新增 MCP 安装与巡检 crate（配方目录 / 五客户端适配 / 握手校验） ([b54b7e2](https://github.com/pengjiaxusz/pengj-templates/commit/b54b7e22f6c4b286dbb90286cb0b913b131b6620))

## [0.45.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.45.1-beta.1...v0.45.2-beta.1) (2026-11-02)


### Bug Fixes

* **templates:** 增强分支拓扑与净贡献巡检的补丁落地上下文漂移判定 ([e16eca5](https://github.com/pengjiaxusz/pengj-templates/commit/e16eca55fabf7cf83b64b89507c6e676a6301a07))

## [0.45.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.45.0-beta.1...v0.45.1-beta.1) (2026-11-02)


### Bug Fixes

* **templates:** 增强 Tree-Diff Guard 补丁上下文容错与冲突分支保全度判定 ([8e94fc0](https://github.com/pengjiaxusz/pengj-templates/commit/8e94fc00eaafb1bb9448368e79f47799a23e0c86))

## [0.45.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.44.0-beta.1...v0.45.0-beta.1) (2026-10-30)


### Features

* **templates:** 全面升级为原生 bque CLI 调度并精简 justfile ([5b93248](https://github.com/pengjiaxusz/pengj-templates/commit/5b932484e13a6404ba2326a528909232434c203c))

## [0.44.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.43.0-beta.1...v0.44.0-beta.1) (2026-10-30)


### Features

* **templates:** 统一引导通过 bque-runner 调度并更新使用矩阵 ([2b8af46](https://github.com/pengjiaxusz/pengj-templates/commit/2b8af466c05d6398e7f7b08be7be7d8ea4df6233))

## [0.43.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.42.0-beta.1...v0.43.0-beta.1) (2026-10-30)


### Features

* **templates:** 强化裸跑原生构建命令硬性禁令，清理 just 别名与非必要 gate ([4e44c3f](https://github.com/pengjiaxusz/pengj-templates/commit/4e44c3f67fcd576fe7e08ab230ac9cda8263a990))

## [0.42.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.41.0-beta.1...v0.42.0-beta.1) (2026-10-30)


### Features

* **templates:** 统一 justfile 接入 gate 与 bench 任务并纳管前端构建 ([f508cb1](https://github.com/pengjiaxusz/pengj-templates/commit/f508cb172f38e910bfa8448cca0b4ce0c5da527e))

## [0.41.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.40.1-beta.1...v0.41.0-beta.1) (2026-10-30)


### Features

* **templates:** 缺失 bque 时提示一键安装命令并支持默认路径回退 ([8e4c946](https://github.com/pengjiaxusz/pengj-templates/commit/8e4c946ce8bbfd3af9b6d19f08a1043efe81ab6c))

## [0.40.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.40.0-beta.1...v0.40.1-beta.1) (2026-10-30)


### Bug Fixes

* **agent:** 为 bque 技能文档添加管道卫生反例豁免注释 ([810c770](https://github.com/pengjiaxusz/pengj-templates/commit/810c7702bb6e2ef0d6b15444cb925a1b0d4bf212))

## [0.40.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.39.0-beta.1...v0.40.0-beta.1) (2026-10-30)


### Features

* **agent:** 新增 bque 全局编译排队与纯净跑分技能并精简 AGENTS.md ([dcd7864](https://github.com/pengjiaxusz/pengj-templates/commit/dcd786454a8cb60151a19b11228a05cef61577d0))

## [0.39.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.38.1-beta.1...v0.39.0-beta.1) (2026-10-30)


### Features

* **templates:** 新增 bque 模板层及编译排队与纯净跑分支持 ([7f2608b](https://github.com/pengjiaxusz/pengj-templates/commit/7f2608b893805cff0fcbbcb6e65e50c57e8b6530))

## [0.38.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.38.0-beta.1...v0.38.1-beta.1) (2026-10-29)


### Bug Fixes

* **templates:** 将 Cargo.toml 纳入 rust 层 update_ignore ([ad63fa6](https://github.com/pengjiaxusz/pengj-templates/commit/ad63fa638947a1ff3d7476762550f2e3ded41ccc))

## [0.38.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.37.2-beta.1...v0.38.0-beta.1) (2026-10-24)


### Features

* **core:** 支持模板废弃受管文件的安全物理移除与技能选项平滑迁移 ([02abfb0](https://github.com/pengjiaxusz/pengj-templates/commit/02abfb098cafa37a6c889a85f27266f4d7d32e7a))

## [0.37.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.37.1-beta.1...v0.37.2-beta.1) (2026-10-13)


### Bug Fixes

* **agent:** branch-sync Tree-Diff Guard 增加补丁上下文漂移时的增删行保全容错 ([63a7f4f](https://github.com/pengjiaxusz/pengj-templates/commit/63a7f4ff5f44cbc18d3bd464b6d7a335d4da2de9))

## [0.37.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.37.0-beta.1...v0.37.1-beta.1) (2026-10-11)


### Bug Fixes

* **agent:** 同步分支技能防漏提交改造，远端真值发现与分叉隔离 ([e205681](https://github.com/pengjiaxusz/pengj-templates/commit/e20568136c36274d2c1554ccfb1eb5d7c8daaaa6))

## [0.37.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.36.3-beta.1...v0.37.0-beta.1) (2026-10-08)


### Features

* **agent:** branch-sync 上收沙箱适配与强推复核经验并新增深入参考 ([95a5ea2](https://github.com/pengjiaxusz/pengj-templates/commit/95a5ea2cb42b87c5cd59ffe61a3741dc1b790fa8))
* **agent:** subrepo-sync 上收依赖指针先后判定与陈旧产物陷阱 ([78c463c](https://github.com/pengjiaxusz/pengj-templates/commit/78c463c5e58ee9558179be43afd688e9c7ad0e0f))

## [0.36.3-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.36.2-beta.1...v0.36.3-beta.1) (2026-10-07)


### Bug Fixes

* **agent:** 切换 Invoke-Git 至 ReadToEndAsync 彻底消除线程上下文异常与管道死锁 ([014b7f6](https://github.com/pengjiaxusz/pengj-templates/commit/014b7f6947e6bb09c6c024eb5c355ef857c2affc))

## [0.36.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.36.1-beta.1...v0.36.2-beta.1) (2026-10-07)


### Bug Fixes

* **agent:** 增强 branch-sync 异步管道与超时机制并杜绝无头编辑器阻塞 ([48b9276](https://github.com/pengjiaxusz/pengj-templates/commit/48b9276e8e37607937873efa53503ff377f0c55a))

## [0.36.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.36.0-beta.1...v0.36.1-beta.1) (2026-10-06)


### Bug Fixes

* **agent:** 移除 continue-sync 管道截断以满足 ps-pipeline-hygiene ([c3f37ed](https://github.com/pengjiaxusz/pengj-templates/commit/c3f37ede9bdb44f9fc61b6648436a4a8993940b0))

## [0.36.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.35.0-beta.1...v0.36.0-beta.1) (2026-10-06)


### Features

* **agent:** branch-sync支持默认全分支1-Shot同步与动态集成分支推导 ([906a947](https://github.com/pengjiaxusz/pengj-templates/commit/906a94719756061211ee12f5020f1f1378b37a48))

## [0.35.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.34.0-beta.1...v0.35.0-beta.1) (2026-09-23)


### Features

* **agent:** 增强分支同步防漏防覆盖机制并支持单调用闭环 ([1a853a9](https://github.com/pengjiaxusz/pengj-templates/commit/1a853a95e2cee1cc602302bc7d8089dbe92c91e3))

## [0.34.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.33.0-beta.1...v0.34.0-beta.1) (2026-09-22)


### Features

* **agent:** 引入 template-sync 极速流水线脚本与使用指南 ([38f4216](https://github.com/pengjiaxusz/pengj-templates/commit/38f4216e57b2e70a223492fc66c75824c1ae9a24))
* **cli:** 支持批量项目路径与技能同步命令行参数 ([6c6588f](https://github.com/pengjiaxusz/pengj-templates/commit/6c6588f071a7beca94ad8f869db38eeebc4d1cf2))
* **core:** 支持受管技能资产更新对齐选项与单测 ([1854306](https://github.com/pengjiaxusz/pengj-templates/commit/18543065d44526f3ed0b90e245eda4e392f601cd))

## [0.33.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.32.0-beta.1...v0.33.0-beta.1) (2026-09-22)


### Features

* **core:** 优化模板更新状态判定与审计巡检逻辑 ([583de3d](https://github.com/pengjiaxusz/pengj-templates/commit/583de3d43a1889b4e6fad949a2aaba1db5819d43))
* **templates:** 完善分层种子保护与技能模板空白符修剪 ([a1f6f07](https://github.com/pengjiaxusz/pengj-templates/commit/a1f6f074dc0e125b200fed4ca48a4bd76df99088))

## [0.32.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.31.0-beta.1...v0.32.0-beta.1) (2026-09-21)


### Features

* **agent:** 分支同步通用化集成分支推导与未追踪路径正则忽略 ([27e34e0](https://github.com/pengjiaxusz/pengj-templates/commit/27e34e0c0e744a52cae84092324b8b53c994eceb))
* **core:** 新增项目模板漂移巡检分析引擎与 CLI audit 命令 ([789e184](https://github.com/pengjiaxusz/pengj-templates/commit/789e184edc7974d8144d94a267d3641c91b40386))
* **templates:** 扩充模板巡检工作流并在 AGENTS.md 注入防篡改红线 ([ff86bc7](https://github.com/pengjiaxusz/pengj-templates/commit/ff86bc7283eb3427ea0241bde763d0e300887cf8))

## [0.31.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.30.0-beta.1...v0.31.0-beta.1) (2026-09-13)


### Features

* **app:** 引入 cha-set 子模块作为 UI 库并升级前端界面 ([61879b2](https://github.com/pengjiaxusz/pengj-templates/commit/61879b2a7cef49666c1682d9e6647ec5e4dd8ebd))

## [0.30.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.29.1-beta.1...v0.30.0-beta.1) (2026-09-13)


### Features

* **template-sync:** 强化新项目纳管自动诊断推荐方案与用户咨询流程 ([723e829](https://github.com/pengjiaxusz/pengj-templates/commit/723e82911cc2fd6fe89dacc272a733711c2248c5))

## [0.29.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.29.0-beta.1...v0.29.1-beta.1) (2026-09-13)


### Performance Improvements

* **agent:** 优化分支同步流程减少工具调用与消耗 ([c335b69](https://github.com/pengjiaxusz/pengj-templates/commit/c335b69d27663f6732b7324b51f02f4a0eb88938))

## [0.29.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.28.0-beta.1...v0.29.0-beta.1) (2026-09-12)


### Features

* **agent:** 新增 subrepo-sync 通用子仓库同步与升级技能 ([d77c3ba](https://github.com/pengjiaxusz/pengj-templates/commit/d77c3ba912daf5775bde0435b059a6303340dd51))

## [0.28.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.27.0-beta.1...v0.28.0-beta.1) (2026-09-08)


### Features

* **templates:** 增强提交语言门禁并支持字面量豁免与小额容差 ([4c146fc](https://github.com/pengjiaxusz/pengj-templates/commit/4c146fcb3bc3faf7d03e6add6bceec65fd0e02ba))

## [0.27.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.26.0-beta.1...v0.27.0-beta.1) (2026-09-07)


### Features

* **templates:** 增加提交信息语言校验门禁并强化规范约束 ([cb3295e](https://github.com/pengjiaxusz/pengj-templates/commit/cb3295eac158c157f75fc3803c633e20aab1642c))

## [0.26.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.25.0-beta.1...v0.26.0-beta.1) (2026-09-03)


### Features

* **agent:** 强化回复前提交门禁与规划模式约束并补齐触发词 ([0eb0366](https://github.com/pengjiaxusz/pengj-templates/commit/0eb0366cd1f7abc163ade11b283c4aeefaa316e4))
* **core:** 规范类文件无托管块时采用置顶放置策略 ([6a0ad59](https://github.com/pengjiaxusz/pengj-templates/commit/6a0ad59c91da517eb4ce5e6080a12b98f8137636))

## [0.25.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.24.0-beta.1...v0.25.0-beta.1) (2026-09-01)


### Features

* **agent:** 新增 template-sync 模板同步与项目纳管技能 ([e68c1bc](https://github.com/pengjiaxusz/pengj-templates/commit/e68c1bcb2874b1b0ce3ef3116c7797ec13645a6a))

## [0.24.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.23.0-beta.1...v0.24.0-beta.1) (2026-09-01)


### Features

* **agent:** 增加阶段性完工即提交硬性约束与触发时机规范 ([b189c87](https://github.com/pengjiaxusz/pengj-templates/commit/b189c8793083f2f9b17d0f9ce32d4e08c77ec876))

## [0.23.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.22.0-beta.1...v0.23.0-beta.1) (2026-08-31)


### Features

* **templates:** 丰富并通用化 Rust 中文编程与本土化命名规范 ([2c5afae](https://github.com/pengjiaxusz/pengj-templates/commit/2c5afae0618a0f8905272e9a2978eb4b3bcc8046))

## [0.22.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.21.5-beta.1...v0.22.0-beta.1) (2026-08-31)


### Features

* **templates:** 精简 AGENTS.md 规范、移除 codegraph 并支持自动提交推送选项 ([a7fe68a](https://github.com/pengjiaxusz/pengj-templates/commit/a7fe68a689c08c1c6f1f13fc1a1908c730bff103))

## [0.21.5-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.21.4-beta.1...v0.21.5-beta.1) (2026-08-28)


### Bug Fixes

* **templates:** 将 .gitattributes 模板注释翻译为英文 ([29a333a](https://github.com/pengjiaxusz/pengj-templates/commit/29a333aedc2f3c0a1be0155bfe6c30fc52174076))

## [0.21.4-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.21.3-beta.1...v0.21.4-beta.1) (2026-08-28)


### Bug Fixes

* **agent:** 英文项目不再出现中文 ([fc9f9ec](https://github.com/pengjiaxusz/pengj-templates/commit/fc9f9ec6ec4f0dfb6057ff66e7cdbbb1e1776dbd))

## [0.21.3-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.21.2-beta.1...v0.21.3-beta.1) (2026-08-28)


### Bug Fixes

* **agent:** commit 模板项目专属区默认留空 ([8b3da59](https://github.com/pengjiaxusz/pengj-templates/commit/8b3da59d115b5cc94fb550fce5087cfc37f4c6c8))

## [0.21.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.21.1-beta.1...v0.21.2-beta.1) (2026-08-28)


### Bug Fixes

* **agent:** 修正架构检查条件以兼容默认全量技能 ([119f23d](https://github.com/pengjiaxusz/pengj-templates/commit/119f23d27083fd545f22e172820b76205ec16da3))

## [0.21.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.21.0-beta.1...v0.21.1-beta.1) (2026-08-28)


### Bug Fixes

* **agent:** 对齐 commit 技能架构检查与模板一致 ([6be8d64](https://github.com/pengjiaxusz/pengj-templates/commit/6be8d64f22eb82fe30bea362c8f05f49d4b41eb2))
* **agent:** 将架构检查移入托管块以支持模板更新传播 ([7ced6ca](https://github.com/pengjiaxusz/pengj-templates/commit/7ced6ca6c1d6ea6e31e9f9386ae48f96c003b86a))

## [0.21.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.20.2-beta.1...v0.21.0-beta.1) (2026-08-28)


### Features

* **agent:** 增加架构文档提交前检查并落地主索引 ([0035f40](https://github.com/pengjiaxusz/pengj-templates/commit/0035f40c77c9f24cf9f15efcbffb988eda17e7df))

## [0.20.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.20.1-beta.1...v0.20.2-beta.1) (2026-08-28)


### Bug Fixes

* **core:** 修正累加文件托管块结构为单块块内标记 ([315c6c9](https://github.com/pengjiaxusz/pengj-templates/commit/315c6c95be4724653987dfdd23c5bb28b5e89c6e))

## [0.20.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.20.0-beta.1...v0.20.1-beta.1) (2026-08-27)


### Bug Fixes

* **core:** 修复 adopt 时 .vscode/settings.json 未自动合并 fileNesting ([83892f9](https://github.com/pengjiaxusz/pengj-templates/commit/83892f97f4eb60bf745596ff1ba5b05bd88e66cf))

## [0.20.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.19.0-beta.1...v0.20.0-beta.1) (2026-08-27)


### Features

* **agent:** 新增 branch-sync 技能，支持无 worktree 与合后分支同步 ([a0672f1](https://github.com/pengjiaxusz/pengj-templates/commit/a0672f1f29d078b42ad86ea9c9a4a45598bab408))

## [0.19.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.18.1-beta.1...v0.19.0-beta.1) (2026-08-27)


### Features

* **agent:** 新增 write-a-skill 技能，符合双语与托管块规范 ([ca2d968](https://github.com/pengjiaxusz/pengj-templates/commit/ca2d96848db64fb92102d727a43b055508b9f779))

## [0.18.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.18.0-beta.1...v0.18.1-beta.1) (2026-08-25)


### Bug Fixes

* **agent:** fileNesting 折叠规则改为 commitlint.*，纳入 commitlint.base.js ([bb6641a](https://github.com/pengjiaxusz/pengj-templates/commit/bb6641a3b99762a127359c1f74dfaf90e096a881))

## [0.18.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.17.1-beta.1...v0.18.0-beta.1) (2026-08-25)


### Features

* **core:** 技能接管以模板整页为准，description 一并覆盖 ([4c49c85](https://github.com/pengjiaxusz/pengj-templates/commit/4c49c852b341f0804c4c47b41388849aff5e605f))

## [0.17.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.17.0-beta.1...v0.17.1-beta.1) (2026-08-25)


### Bug Fixes

* **core:** 技能接管剥离渲染页自身 frontmatter，避免双 frontmatter ([e6e7e75](https://github.com/pengjiaxusz/pengj-templates/commit/e6e7e75dfd12b7f0726d0008fbd950d6b5b9977d))

## [0.17.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.16.0-beta.1...v0.17.0-beta.1) (2026-08-25)


### Features

* **core:** 存量技能自动接管，原文下移纳管过渡区 ([3e2aeb6](https://github.com/pengjiaxusz/pengj-templates/commit/3e2aeb6a52687980a6c93425cb4cc444c5c4aaef))

## [0.16.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.15.0-beta.1...v0.16.0-beta.1) (2026-08-25)


### Features

* **agent:** commit 技能框架去脚本化，检查定义归项目专属区 ([b1f382a](https://github.com/pengjiaxusz/pengj-templates/commit/b1f382ae7fb9ec9fe023cdddbe33eb56e906935b))

## [0.15.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.14.0-beta.1...v0.15.0-beta.1) (2026-08-25)


### Features

* **core:** 固化技能扩展规范，存量全自定义技能不接管 ([10d88db](https://github.com/pengjiaxusz/pengj-templates/commit/10d88db9cadef10b032c1aa38948e204f08973dd))

## [0.14.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.13.1-beta.1...v0.14.0-beta.1) (2026-08-25)


### Features

* **app:** 生成页默认 edition 与 CLI 对齐为 2024 ([3633522](https://github.com/pengjiaxusz/pengj-templates/commit/36335223996548e727311dc7c50fcae8473e57cd))
* **cli:** create 默认 Rust edition 调整为 2024 ([4afeba3](https://github.com/pengjiaxusz/pengj-templates/commit/4afeba3a3be725e739f831cc27073605ddf35243))
* **core:** 纳管与更新合并语义增强 ([526f651](https://github.com/pengjiaxusz/pengj-templates/commit/526f651d5fff1958aa28d5a7ac1f0803f73cb6b2))

## [0.13.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.13.0-beta.1...v0.13.1-beta.1) (2026-08-25)


### Bug Fixes

* **app:** 修复 tauri dev 启动时 beforeDevCommand 路径错误 ([6095d39](https://github.com/pengjiaxusz/pengj-templates/commit/6095d39139cf3507a5dcf7f2e7910563aaec9f3f))

## [0.13.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.12.0-beta.1...v0.13.0-beta.1) (2026-08-25)


### Features

* **core:** 受管块合并引擎与 TOML 结构化合并，JSON 并集用户优先 ([3bb0b48](https://github.com/pengjiaxusz/pengj-templates/commit/3bb0b4884a56b8d7089f74f07924e5377f6d2050))

## [0.12.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.11.0-beta.1...v0.12.0-beta.1) (2026-08-24)


### Features

* **cli:** 支持 adopt 存量项目纳管命令与 Tauri 接口 ([44af406](https://github.com/pengjiaxusz/pengj-templates/commit/44af40627299e191f938f99ae855219db7bd7fd6))
* **core:** 支持模板锚点局部合并与存量项目纳管 ([5b67ed8](https://github.com/pengjiaxusz/pengj-templates/commit/5b67ed88eb88564c0b6af6eafef7712a357cf736))
* **templates:** 新增 rust-workspace 与 codegraph 模板层并支持插槽保护与 Commitlint 继承 ([5c894b7](https://github.com/pengjiaxusz/pengj-templates/commit/5c894b7c42d92471608a8d59954ce175cd3b3bf3))

## [0.11.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.10.0-beta.1...v0.11.0-beta.1) (2026-08-24)


### Features

* **agent:** 为当前项目安装渲染版 arch-align 架构对齐技能 ([fb5dade](https://github.com/pengjiaxusz/pengj-templates/commit/fb5dade20edd8ef67f6af4d1bfbe10b2c973180f))


### Bug Fixes

* **cli:** 统一 CLI 二进制与命令名称为 pengj-templates-cli ([74cce8d](https://github.com/pengjiaxusz/pengj-templates/commit/74cce8d54d934cac6fef19b237c69d8f28184691))

## [0.10.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.9.1-beta.1...v0.10.0-beta.1) (2026-08-24)


### Features

* **agent:** 新增 arch-align 架构对齐技能并支持中英双语 ([41fff78](https://github.com/pengjiaxusz/pengj-templates/commit/41fff78a9ebe7a595dcfdd66f15a6d8364aa76bc))

## [0.9.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.9.0-beta.1...v0.9.1-beta.1) (2026-08-24)


### Bug Fixes

* **core:** 规范 Manifest 文件名为 .pengj-templates.json ([708b19f](https://github.com/pengjiaxusz/pengj-templates/commit/708b19fc1783e14bc3e36af6b796cc5ec60e90d4))

## [0.9.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.8.1-beta.1...v0.9.0-beta.1) (2026-08-24)


### Features

* **core:** 支持按层累加拼接 .gitattributes ([b2c51aa](https://github.com/pengjiaxusz/pengj-templates/commit/b2c51aa7606581131ff0b16ce3b0d7a6d38ec986))

## [0.8.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.8.0-beta.1...v0.8.1-beta.1) (2026-08-24)


### Bug Fixes

* **core:** 增量合并 .vscode/settings.json 保留用户自定义 ([25cdec2](https://github.com/pengjiaxusz/pengj-templates/commit/25cdec2fc24da73b169f7eb442f20c0eb71969b1))

## [0.8.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.7.0-beta.1...v0.8.0-beta.1) (2026-08-24)


### Features

* 支持 VS Code 文件嵌套与工作空间双轨同步 ([21e05bb](https://github.com/pengjiaxusz/pengj-templates/commit/21e05bb29f980cdf16c83c8f8f2998e72e1d0dad))

## [0.7.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.6.3-beta.1...v0.7.0-beta.1) (2026-08-24)


### Features

* **agent:** 新增 caveman/grill-me 技能并动态渲染技能清单 ([e019bda](https://github.com/pengjiaxusz/pengj-templates/commit/e019bda231084fbb4dafdd1ab03071aa7e3010d7))
* **app:** GUI 技能勾选生成 ([bf9fcc8](https://github.com/pengjiaxusz/pengj-templates/commit/bf9fcc8b8db9c27a8a1dc799619f15925af90182))
* **cli:** create 支持 --skills 选择技能 ([daca2ef](https://github.com/pengjiaxusz/pengj-templates/commit/daca2efefb1d584100d412954988d114fffd7305))
* **core:** 支持技能发现与按选择过滤 ([e39e2c8](https://github.com/pengjiaxusz/pengj-templates/commit/e39e2c82dcf8fe84a04e653621877ac90809c0b4))

## [0.6.3-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.6.2-beta.1...v0.6.3-beta.1) (2026-08-23)


### Bug Fixes

* 更新 commitlint 配置，添加 scope-enum 规则 ([aea7395](https://github.com/pengjiaxusz/pengj-templates/commit/aea7395f952405edefc460287b13076735cb6eb0))

## [0.6.2-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.6.1-beta.1...v0.6.2-beta.1) (2026-08-23)


### Bug Fixes

* pre-release 版本自动标记 GitHub Prerelease 徽章 ([4dadeca](https://github.com/pengjiaxusz/pengj-templates/commit/4dadeca714af0cafb85abbf91145888f7e68d05c))

## [0.6.1-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.6.0-beta.1...v0.6.1-beta.1) (2026-08-23)


### Bug Fixes

* Windows 构建剥离 pre-release 版本号后缀 ([34d6398](https://github.com/pengjiaxusz/pengj-templates/commit/34d639806a07104628753bde727b95526c2fb954))

## [0.6.0-beta.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.5.0...v0.6.0-beta.1) (2026-08-23)


### Documentation

* **agent:** commit 技能支持切发 beta/preview/rc ([6fad804](https://github.com/pengjiaxusz/pengj-templates/commit/6fad8045867430825092e71892ee112a02c217f1))

## [0.5.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.4.1...v0.5.0) (2026-08-23)


### Features

* 全自动合并 Release PR 与 beta/preview 分阶段说明 ([198cb33](https://github.com/pengjiaxusz/pengj-templates/commit/198cb33e011e6cc1314777adbf1106d4badad082))


### Bug Fixes

* auto-merge 改为步骤内检测 Release PR 而非 job 级 if ([b8c6bd2](https://github.com/pengjiaxusz/pengj-templates/commit/b8c6bd205e39a7147e90d8942e46bc9eb4932757))

## [0.4.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.4.0...v0.4.1) (2026-08-23)


### Bug Fixes

* 修正 sync-lockfile 的 cargo update 参数重复 ([b67adc0](https://github.com/pengjiaxusz/pengj-templates/commit/b67adc06184b1e081e2f83c1cfa3b641dd08fb0a))

## [0.4.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.3.0...v0.4.0) (2026-08-23)


### Features

* 支持 Win/Linux/macOS arm64 并统一多架构产物命名 ([e4493ca](https://github.com/pengjiaxusz/pengj-templates/commit/e4493caecba86eb2f05c1aae25c7773e1e6ae6b7))


### Bug Fixes

* macOS 便携包动态定位并打包 .app ([efe016d](https://github.com/pengjiaxusz/pengj-templates/commit/efe016d6906a692bf793d4665743b4d607ee6728))
* 修正多架构便携打包步骤路径变量 ([216d66b](https://github.com/pengjiaxusz/pengj-templates/commit/216d66b49dd29a2c58b33ca016f225a37f4946e5))

## [0.3.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.7...v0.3.0) (2026-08-23)


### Features

* 增加 Linux/macOS GUI 构建发布 ([0c9d2be](https://github.com/pengjiaxusz/pengj-templates/commit/0c9d2be90906efdc285fae06afa4e7e37d16ac26))

## [0.2.7](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.6...v0.2.7) (2026-08-23)


### Bug Fixes

* GUI 命名为 pengj-templates 而非 pengj-app ([6b3848a](https://github.com/pengjiaxusz/pengj-templates/commit/6b3848a93332c4bfc653d85d179407367535900a))

## [0.2.6](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.5...v0.2.6) (2026-08-23)


### Bug Fixes

* Windows 便携包动态定位 GUI 可执行文件 ([c300d91](https://github.com/pengjiaxusz/pengj-templates/commit/c300d91e0daba063f3c8ea1452661e7c69fb3558))

## [0.2.5](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.4...v0.2.5) (2026-08-23)


### Bug Fixes

* Windows 便携打包 PowerScript 变量缺 \$ 前缀 ([0999e9c](https://github.com/pengjiaxusz/pengj-templates/commit/0999e9c1f1ef1a6707908e6f4a71be12ebdea022))

## [0.2.4](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.3...v0.2.4) (2026-08-23)


### Bug Fixes

* 发布产物改为便携版 zip（GUI+CLI，CLI 命名 pengj-templates-cli） ([3e0a061](https://github.com/pengjiaxusz/pengj-templates/commit/3e0a061d4ec8e4595b2324091b7dd9033868f1b9))

## [0.2.3](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.2...v0.2.3) (2026-08-23)


### Bug Fixes

* tauri 构建/开发命令改为 workspace 感知 ([b661efb](https://github.com/pengjiaxusz/pengj-templates/commit/b661efb0e7b0b5eb45ae4664d1fe4272a2c5fbfe))

## [0.2.2](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.1...v0.2.2) (2026-08-23)


### Bug Fixes

* release GUI 构建升级到 Node 24 ([f343341](https://github.com/pengjiaxusz/pengj-templates/commit/f3433412429590c8de6e7a60e244884bfe12d782))

## [0.2.1](https://github.com/pengjiaxusz/pengj-templates/compare/v0.2.0...v0.2.1) (2026-08-23)


### Bug Fixes

* 修复 release 构建流程（pnpm 版本冲突 + windows tar 打包） ([d758ff6](https://github.com/pengjiaxusz/pengj-templates/commit/d758ff696b60559257b8fe9cee4bd133a9cdaa67))

## [0.2.0](https://github.com/pengjiaxusz/pengj-templates/compare/v0.1.0...v0.2.0) (2026-08-23)


### Features

* list-layers 支持 --json 输出 ([747397b](https://github.com/pengjiaxusz/pengj-templates/commit/747397b71d51c26f09273fc70413b57cf0c7c8a9))
* 完整的分层模板生成与同步更新工具 ([c6d0eb6](https://github.com/pengjiaxusz/pengj-templates/commit/c6d0eb6d37398096e9ea869dcf411841b9d72fbf))


### Bug Fixes

* 修复 clippy 报错使 CI 通过 ([aa6c177](https://github.com/pengjiaxusz/pengj-templates/commit/aa6c17735e4f90bd3e5f3544f35afea2dac12e0f))
