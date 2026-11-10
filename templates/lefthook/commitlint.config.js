import base from './commitlint.base.js';

export default {
  ...base,
  rules: {
    ...base.rules,
    // {% if options["skill_lang"] == "en" %}Extend the project-specific scope whitelist here (e.g. ['core', 'cli', 'app']){% else %}在此处扩展项目特有的 scope 白名单（例如 ['core', 'cli', 'app']）{% endif %}
    // {% if options["skill_lang"] == "en" %}This file is protected by update_ignore and is never overwritten by template updates{% else %}本文件受 update_ignore 保护，模板更新时不会被覆盖{% endif %}
  },
};
