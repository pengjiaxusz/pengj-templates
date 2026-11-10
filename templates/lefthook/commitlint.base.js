const subjectLanguagePlugin = {
  rules: {
    'subject-language': ({ scope, subject, raw }, _when = 'always', lang = 'zh') => {
      // {% if options["skill_lang"] == "en" %}Exempt: git revert commits and the release scope (e.g. main){% else %}豁免：git revert 提交 或 release scope (如 main){% endif %}
      if (raw && (raw.startsWith('revert:') || raw.startsWith('Revert "') || scope === 'main')) {
        return [true];
      }
      if (!subject) {
        return [false, '{% if options["skill_lang"] == "en" %}the commit subject cannot be empty{% else %}提交标题 (subject) 不能为空 / commit subject cannot be empty{% endif %}'];
      }

      const trimmedSubject = subject.trim();

      if (lang === 'zh') {
        // {% if options["skill_lang"] == "en" %}Chinese mode: require at least 2 CJK characters so pure-English subjects are rejected{% else %}中文模式：要求至少包含 2 个汉字，防止写成纯英文提交{% endif %}
        const chineseMatches = trimmedSubject.match(/[\p{Unified_Ideograph}]/gu) || [];
        const minChineseChars = 2;
        if (chineseMatches.length < minChineseChars) {
          return [
            false,
            `{% if options["skill_lang"] == "en" %}Commit subject must be written in Chinese (detected ${chineseMatches.length} CJK characters, at least ${minChineseChars} required).\n{% else %}提交标题必须使用中文撰写（当前检测到汉字数: ${chineseMatches.length}，要求至少 ${minChineseChars} 个汉字）。\n{% endif %}` +
              `  {% if options["skill_lang"] == "en" %}Invalid example: feat(cli): add new flags\n{% else %}错误示例: feat(cli): add new flags\n{% endif %}` +
              `  {% if options["skill_lang"] == "en" %}Valid example: feat(cli): [Chinese subject with at least two CJK characters]{% else %}正确示例: feat(cli): 新增参数解析支持{% endif %}`,
          ];
        }
      } else if (lang === 'en') {
        // {% if options["skill_lang"] == "en" %}English mode:{% else %}英文模式：{% endif %}
        // {% if options["skill_lang"] == "en" %}Step a: strip literal contents inside single quotes, double quotes and backticks (e.g. support 'name' alias){% else %}步骤 a: 剥离单引号/双引号/反引号中的字面量内容（如 support '名称' alias）{% endif %}
        const strippedSubject = trimmedSubject.replace(/(['"`])[\s\S]*?\1/g, ' ').trim();
        const strippedChinese = strippedSubject.match(/[\p{Unified_Ideograph}]/gu) || [];

        // {% if options["skill_lang"] == "en" %}If no CJK characters remain, they were all quoted literals or proper nouns, so accept the subject{% else %}若剥离引号后无汉字，说明汉字均为被引用的字面量/专有名词，直接放行{% endif %}
        if (strippedChinese.length === 0) {
          return [true];
        }

        // {% if options["skill_lang"] == "en" %}Step b: reject subjects that start with a CJK character{% else %}步骤 b: 检查首字符是否为汉字（如 "修复...", "为..." 开头判定为中文句式）{% endif %}
        const startsWithChinese = /^[\p{Unified_Ideograph}]/u.test(strippedSubject);
        // {% if options["skill_lang"] == "en" %}Step c: tolerate a few unquoted CJK characters (e.g. a product name, at most 4){% else %}步骤 c: 允许小额无引号汉字引用容差（如 add 简体中文 support，允许 <= 4 个汉字）{% endif %}
        const maxUnquotedChineseChars = 4;

        if (startsWithChinese || strippedChinese.length > maxUnquotedChineseChars) {
          return [
            false,
            `Commit subject must be in English (detected ${strippedChinese.length} Chinese characters: "${strippedChinese.join('')}").\n` +
              `  {% if options["skill_lang"] == "en" %}If you need to reference a Chinese term or literal, wrap it in quotes (e.g. 'some-term').\n{% else %}If referencing Chinese terms/literals, please wrap them in quotes (e.g. '中文' or \`中文\`).\n{% endif %}` +
              `  Example: feat(cli): add new flags\n` +
              `  {% if options["skill_lang"] == "en" %}Example with literals: fix(core): support the legacy alias property{% else %}Example with literals: fix(core): support '名称' alias property{% endif %}`,
          ];
        }
      }
      return [true];
    },
  },
};

export default {
  extends: ['@commitlint/config-conventional'],
  plugins: [subjectLanguagePlugin],
  rules: {
    'body-max-line-length': [0], // {% if options["skill_lang"] == "en" %}no line-length limit for the body{% else %}禁用正文行长度限制{% endif %}
    'subject-case': [0, 'always'], // {% if options["skill_lang"] == "en" %}no case check on the subject{% else %}禁用 subject 大小写检查（允许中文标题）{% endif %}
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
    'subject-language': [2, 'always', '{% if options["commit_zh"] is undefined or options["commit_zh"] %}zh{% else %}en{% endif %}'],
  },
};
