export default {
  extends: ['@commitlint/config-conventional'],
  rules: {
    'body-max-line-length': [0], // 禁用正文行长度限制
    'subject-case': [0, 'always'], // 禁用 subject 大小写检查（允许中文标题）
    'type-enum': [
      2,
      'always',
      [
        'feat',
        'fix',
        'docs',
        'style',
        'refactor',
        'perf',
        'test',
        'build',
        'ci',
        'chore',
        'revert',
      ],
    ],
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
    ],
  },
};
