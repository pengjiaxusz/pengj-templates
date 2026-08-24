import { existsSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { resolve } from 'node:path';

const dbPath = resolve('.codegraph', 'codegraph.db');
if (!existsSync(dbPath)) {
  console.log('[codegraph] 首次安装，正在初始化代码知识图谱索引...');
  try {
    execSync('npx codegraph query .', { stdio: 'inherit' });
  } catch (err) {
    console.warn('[codegraph] 索引初始化跳过或失败（可后续手动运行 npx codegraph query .）');
  }
}
