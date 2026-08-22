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
  },
};