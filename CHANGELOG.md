# Changelog

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
