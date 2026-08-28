import base from './commitlint.base.js';

const pengjUserConfig = {
  rules: {
    'scope-enum': [
      2,
      'always',
      [
        'agent', // 智能体技能与 .agents 层（含 templates/agent）
        'app', // Tauri GUI（crates/app）
        'cli', // CLI（crates/cli）
        'core', // Rust core 共享库（crates/core）
        'ci', // GitHub Actions 工作流与发布自动化（.github/workflows）
        'templates', // 模板层内容（templates/ 下各层）
        'main', // release-please 自动生成的 release 提交，保持历史兼容
      ],
    ]
  }
};

export default {
  ...base,
  ...pengjUserConfig,
  rules: {
    ...base.rules,
    ...pengjUserConfig.rules,
  },
};
