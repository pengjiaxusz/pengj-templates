# Changelog

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
